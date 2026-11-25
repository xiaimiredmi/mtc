//! The C API: everything a game engine needs to read and write `.mtc`, and nothing
//! else.
//!
//! Three concerns shape this file. Buffers cross the boundary as `(pointer, length)`
//! pairs the caller frees, so nothing is kept alive on this side; a panic must not
//! unwind into C, so every entry point is wrapped; and the error message is per
//! thread, because decoding on a worker pool is the normal case.

use std::cell::RefCell;
use std::ffi::CString;
use std::mem;
use std::os::raw::{c_char, c_int};

use mtc::container::{self, AqMode, Image, Options};

/// Every buffer we hand out has this much room in front of it, holding the total
/// allocation length so `mtc_free` needs no size argument from the caller.
const HEADER: usize = 16;

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::new("").unwrap());
}

fn set_error(text: &str) {
    let clean = text.replace('\0', " ");
    let value = CString::new(clean).unwrap_or_else(|_| CString::new("error").unwrap());
    LAST_ERROR.with(|slot| *slot.borrow_mut() = value);
}

fn code_of(message: &str) -> c_int {
    let lower = message.to_ascii_lowercase();
    if lower.contains("truncated") {
        -3
    } else if lower.contains("larger than") || lower.contains("too large") {
        -4
    } else if lower.contains("not an mtc") || lower.contains("version") || lower.contains("mode") {
        -2
    } else {
        -6
    }
}

/// Copy `bytes` into a fresh allocation the caller owns, with the length in front.
unsafe fn give(bytes: &[u8]) -> *mut u8 {
    let total = bytes.len() + HEADER;
    let mut block: Vec<u8> = Vec::with_capacity(total);
    block.extend_from_slice(&(total as u64).to_le_bytes());
    block.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    block.extend_from_slice(bytes);
    let ptr = block.as_mut_ptr();
    mem::forget(block);
    ptr.add(HEADER)
}

/// Take back a buffer from `give`.
///
/// # Safety
/// `ptr` must come from this library, or be null.
unsafe fn take(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    let base = ptr.sub(HEADER);
    let mut len_bytes = [0u8; 8];
    std::ptr::copy_nonoverlapping(base, len_bytes.as_mut_ptr(), 8);
    let total = u64::from_le_bytes(len_bytes) as usize;
    drop(Vec::from_raw_parts(base, total, total));
}

/// Run `body`, turning a panic into an error code instead of an unwinding C frame.
fn guarded<F: FnOnce() -> c_int>(body: F) -> c_int {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)) {
        Ok(code) => code,
        Err(_) => {
            set_error("internal error in the codec");
            -6
        }
    }
}

fn slice<'a>(data: *const u8, len: i32) -> Option<&'a [u8]> {
    if data.is_null() || len <= 0 {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts(data, len as usize) })
}

/// Read the header without decoding anything.
#[no_mangle]
pub unsafe extern "C" fn mtc_info(data: *const u8, len: i32, out: *mut MtcInfo) -> c_int {
    guarded(|| {
        let bytes = match slice(data, len) {
            Some(b) => b,
            None => {
                set_error("null data or empty length");
                return -1;
            }
        };
        if out.is_null() {
            set_error("null MtcInfo");
            return -1;
        }
        match container::probe(bytes) {
            Ok(h) => {
                let channels = match h.mode {
                    container::mode::GRAY_LOSSY | container::mode::GRAY_LOSSLESS => 1,
                    container::mode::RGB444_LOSSY | container::mode::RGB420_LOSSY => 3,
                    _ => 3,
                };
                *out = MtcInfo {
                    width: h.width as i32,
                    height: h.height as i32,
                    channels: if h.has_alpha() { 4 } else { channels },
                    mode: h.mode as i32,
                    quality: h.quality as i32,
                    version: h.version as i32,
                    has_alpha: if h.has_alpha() { 1 } else { 0 },
                    has_aq: if h.aq() { 1 } else { 0 },
                };
                0
            }
            Err(message) => {
                let code = code_of(&message);
                set_error(&message);
                code
            }
        }
    })
}

#[repr(C)]
pub struct MtcInfo {
    pub width: c_int,
    pub height: c_int,
    pub channels: c_int,
    pub mode: c_int,
    pub quality: c_int,
    pub version: c_int,
    pub has_alpha: c_int,
    pub has_aq: c_int,
}

unsafe fn decode_into(
    data: *const u8,
    len: i32,
    max_side: i32,
    out: *mut *mut u8,
    out_w: *mut i32,
    out_h: *mut i32,
    rgb: bool,
) -> c_int {
    guarded(|| {
        let bytes = match slice(data, len) {
            Some(b) => b,
            None => {
                set_error("null data or empty length");
                return -1;
            }
        };
        if out.is_null() || out_w.is_null() || out_h.is_null() {
            set_error("null output pointer");
            return -1;
        }
        let limit = if max_side > 0 { max_side as usize } else { 0 };
        let decoded = if limit > 0 {
            container::decode_downsized(bytes, limit)
        } else {
            container::decode(bytes)
        };
        let img = match decoded {
            Ok(i) => i,
            Err(message) => {
                let code = code_of(&message);
                set_error(&message);
                return code;
            }
        };
        let (w, h) = (img.width as i32, img.height as i32);
        let pixels = if rgb { to_rgb(&img) } else { img.to_rgba() };
        *out = give(&pixels);
        *out_w = w;
        *out_h = h;
        0
    })
}

/// RGB view of a decoded picture: grey is expanded, alpha is dropped.
fn to_rgb(img: &Image) -> Vec<u8> {
    match img.channels {
        1 => {
            let mut out = Vec::with_capacity(img.width * img.height * 3);
            for value in img.data.iter() {
                out.push(*value);
                out.push(*value);
                out.push(*value);
            }
            out
        }
        3 => img.data.clone(),
        _ => {
            let mut out = Vec::with_capacity(img.width * img.height * 3);
            for pixel in img.data.chunks_exact(4) {
                out.push(pixel[0]);
                out.push(pixel[1]);
                out.push(pixel[2]);
            }
            out
        }
    }
}

/// Decode at the picture's own size. This is the whole interface a game needs: the
/// engine scales the result itself, so there is no size argument to get wrong.
#[no_mangle]
pub unsafe extern "C" fn mtc_decode(
    data: *const u8,
    len: i32,
    out: *mut *mut u8,
    out_w: *mut i32,
    out_h: *mut i32,
) -> c_int {
    decode_into(data, len, 0, out, out_w, out_h, false)
}

/// Same, but capped to `max_side` on the long edge. For mip chains and previews; a
/// game that scales its own textures does not need it.
#[no_mangle]
pub unsafe extern "C" fn mtc_decode_scaled(
    data: *const u8,
    len: i32,
    max_side: i32,
    out: *mut *mut u8,
    out_w: *mut i32,
    out_h: *mut i32,
) -> c_int {
    decode_into(data, len, max_side, out, out_w, out_h, false)
}

/// Same, three bytes per pixel, for opaque textures.
#[no_mangle]
pub unsafe extern "C" fn mtc_decode_rgb(
    data: *const u8,
    len: i32,
    out: *mut *mut u8,
    out_w: *mut i32,
    out_h: *mut i32,
) -> c_int {
    decode_into(data, len, 0, out, out_w, out_h, true)
}

/// Decode one rectangle of the picture.
///
/// Same contract as `mtc_decode_rgba`, but only the requested pixels are
/// reconstructed. The coefficient stream is still walked in full (it is sequential),
/// so the win is the transform and the conversion - about 40% for a quarter of the
/// area, and the floor is the entropy decoding.
#[no_mangle]
pub unsafe extern "C" fn mtc_decode_region(
    data: *const u8,
    len: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    out: *mut *mut u8,
    out_w: *mut i32,
    out_h: *mut i32,
) -> c_int {
    guarded(|| {
        let bytes = match slice(data, len) {
            Some(b) => b,
            None => {
                set_error("null data or empty length");
                return -1;
            }
        };
        if out.is_null() || out_w.is_null() || out_h.is_null() || x < 0 || y < 0 || w <= 0 || h <= 0 {
            set_error("bad rectangle or null output pointer");
            return -1;
        }
        match container::decode_region(bytes, x as usize, y as usize, w as usize, h as usize) {
            Ok(img) => {
                *out = give(&img.to_rgba());
                *out_w = img.width as i32;
                *out_h = img.height as i32;
                0
            }
            Err(message) => {
                let code = code_of(&message);
                set_error(&message);
                code
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn mtc_encode(
    pixels: *const u8,
    w: i32,
    h: i32,
    channels: i32,
    quality: i32,
    lossless: i32,
    out: *mut *mut u8,
    out_len: *mut i32,
) -> c_int {
    guarded(|| {
        if pixels.is_null() || out.is_null() || out_len.is_null() || w <= 0 || h <= 0 {
            set_error("null pointer or empty image");
            return -1;
        }
        let (width, height) = (w as usize, h as usize);
        let channels = channels as usize;
        if channels != 1 && channels != 3 && channels != 4 {
            set_error("channels must be 1, 3 or 4");
            return -1;
        }
        let mut data = Vec::with_capacity(width * height * channels);
        data.extend_from_slice(std::slice::from_raw_parts(pixels, width * height * channels));
        let image = match Image::new(width, height, channels, data) {
            Ok(i) => i,
            Err(message) => {
                set_error(&message);
                return -1;
            }
        };
        let options = Options {
            quality: quality.clamp(1, 100) as u8,
            subsample_420: true,
            aq: AqMode::Off,
            version: container::VERSION,
            lossless: lossless != 0,
        };
        match container::encode(&image, &options) {
            Ok(blob) => {
                *out = give(&blob);
                *out_len = blob.len() as i32;
                0
            }
            Err(message) => {
                set_error(&message);
                -6
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn mtc_free(p: *mut u8) {
    take(p);
}

#[no_mangle]
pub extern "C" fn mtc_last_error() -> *const c_char {
    LAST_ERROR.with(|slot| slot.borrow().as_ptr())
}

static VERSION: &[u8] = b"1.6.0\0";

#[no_mangle]
pub extern "C" fn mtc_version() -> *const c_char {
    VERSION.as_ptr() as *const c_char
}
