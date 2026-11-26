//! JNI surface between the Android app and the MTC codec.
//!
//! The app needs four things: turn a camera frame into `.mtc` bytes, turn `.mtc`
//! bytes back into pixels, ask a file what it is, and nothing else. Pixel buffers
//! are `jbyteArray` of RGBA, so the Java side never touches the container.
//!
//! A decoded buffer carries its own size in the first eight bytes (two
//! big-endian `u32`), which saves a second JNI round trip per image, and is
//! exactly the layout `Bitmap.copyPixelsFromBuffer` wants (Android stores
//! `ARGB_8888` as RGBA bytes).
//!
//! Two camera realities are handled here rather than in Java, because the pixel
//! loop already lives here: the colour range of a `YUV_420_888` frame (devices
//! disagree between BT.601 full and limited range) and the quarter turn needed to
//! make a still upright, since `JPEG_ORIENTATION` only applies to JPEG output.

use jni::objects::{JByteArray, JClass};
use jni::sys::{jboolean, jbyteArray, jint, JNI_FALSE};
use jni::JNIEnv;

use mtc::container::{self, AqMode, Image, Options};

fn options(quality: i32) -> Options {
    Options {
        quality: quality.clamp(1, 100) as u8,
        subsample_420: true,
        aq: AqMode::Off,
        version: container::VERSION,
        lossless: false,
    }
}

fn take_bytes(env: &mut JNIEnv, array: JByteArray) -> Result<Vec<u8>, String> {
    env.convert_byte_array(&array)
        .map_err(|e| format!("reading byte array: {}", e))
}

/// Run codec work behind a panic gate.
///
/// <p>A Rust panic unwinding out of an `extern "system"` function aborts the
/// process, which on a phone is the app disappearing while the user looks at it.
/// Nothing here should panic - `rust/tests/robustness.rs` feeds the decoder
/// garbage to keep it that way - but a corrupt file from anywhere is not a reason
/// to die, so a panic becomes a null return instead.
fn guard<T>(f: impl FnOnce() -> T) -> Option<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).ok()
}

fn give_bytes(env: &mut JNIEnv, data: &[u8]) -> jbyteArray {
    match env.byte_array_from_slice(data) {
        Ok(a) => a.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn encode_rgba_bytes(rgba: Vec<u8>, w: usize, h: usize, quality: i32) -> Option<Vec<u8>> {
    if w == 0 || h == 0 || rgba.len() != w * h * 4 {
        return None;
    }
    let img = Image::new(w, h, 4, rgba).ok()?;
    container::encode(&img, &options(quality)).ok()
}

/// `encodeRgba(rgba, width, height, quality) -> byte[] | null`
#[no_mangle]
pub extern "system" fn Java_dev_mtc_Mtc_encodeRgba(
    mut env: JNIEnv,
    _class: JClass,
    rgba: JByteArray,
    width: jint,
    height: jint,
    quality: jint,
) -> jbyteArray {
    let blob = guard(|| {
        let px = take_bytes(&mut env, rgba).ok()?;
        let (w, h) = (width.max(0) as usize, height.max(0) as usize);
        encode_rgba_bytes(px, w, h, quality)
    })
    .flatten();
    match blob {
        Some(blob) => give_bytes(&mut env, &blob),
        None => std::ptr::null_mut(),
    }
}

/// `encodeYuv(y, u, v, width, height, yRowStride, uvRowStride, uvPixelStride,
///            quality, fullRange, rotation, mirror) -> byte[] | null`
///
/// `rotation` is a clockwise quarter turn in degrees, 0/90/180/270; `mirror`
/// flips the result horizontally, which is what a front camera needs.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "system" fn Java_dev_mtc_Mtc_encodeYuv(
    mut env: JNIEnv,
    _class: JClass,
    y: JByteArray,
    u: JByteArray,
    v: JByteArray,
    width: jint,
    height: jint,
    y_row_stride: jint,
    uv_row_stride: jint,
    uv_pixel_stride: jint,
    quality: jint,
    full_range: jboolean,
    rotation: jint,
    mirror: jboolean,
) -> jbyteArray {
    let blob = guard(|| {
        let py = take_bytes(&mut env, y).ok()?;
        let pu = take_bytes(&mut env, u).ok()?;
        let pv = take_bytes(&mut env, v).ok()?;
        let (w, h) = (width.max(0) as usize, height.max(0) as usize);
        if w == 0 || h == 0 {
            return None;
        }
        let y_stride = y_row_stride.max(width) as usize;
        let uv_stride = uv_row_stride.max(width / 2) as usize;
        let uv_pixel = uv_pixel_stride.max(1) as usize;
        if py.len() < y_stride.saturating_mul(h) {
            return None;
        }

        let (rgba, ow, oh) = yuv_to_rgba(
            &py,
            &pu,
            &pv,
            w,
            h,
            y_stride,
            uv_stride,
            uv_pixel,
            full_range != JNI_FALSE as jboolean,
            rotation.max(0) as u32,
            mirror != JNI_FALSE as jboolean,
        );
        encode_rgba_bytes(rgba, ow, oh, quality)
    })
    .flatten();
    match blob {
        Some(blob) => give_bytes(&mut env, &blob),
        None => std::ptr::null_mut(),
    }
}

/// `decodeRgba(data, maxSide) -> byte[] | null`
///
/// Returns an 8-byte big-endian `width`, `height` header followed by RGBA.
/// `maxSide > 0` decodes and downsamples in one pass, which is what thumbnails
/// and the on-screen viewer want; `0` means the full picture.
#[no_mangle]
pub extern "system" fn Java_dev_mtc_Mtc_decodeRgba(
    mut env: JNIEnv,
    _class: JClass,
    data: JByteArray,
    max_side: jint,
) -> jbyteArray {
    let out = guard(|| {
        let blob = take_bytes(&mut env, data).ok()?;
        let limit = max_side.max(0) as usize;
        // A bounded size decodes straight to that size, so the full RGBA buffer
        // never exists; that is what keeps thumbnails cheap on a phone.
        let img = if limit > 0 {
            container::decode_downsized(&blob, limit).ok()?
        } else {
            container::decode(&blob).ok()?
        };
        Some(pack(img.width, img.height, &img.to_rgba()))
    })
    .flatten();
    match out {
        Some(out) => give_bytes(&mut env, &out),
        None => std::ptr::null_mut(),
    }
}


/// `info(data) -> String` — a one-line summary for the header, or null.
/// `merge(frames, count, width, height, channels, mode, exposures) -> byte[] | null`
///
/// <p>Night mode and HDR are the same job twice: line the frames up, then combine them.
/// `mode` is 0 for night (trimmed mean, which is what kills noise) and 1 for HDR
/// (exposure-weighted average, which is what recovers both ends of the range). The
/// frames arrive as one flat array of `count` RGB buffers so the call crosses JNI once.
#[no_mangle]
pub extern "system" fn Java_dev_mtc_Mtc_merge(
    mut env: JNIEnv,
    _class: JClass,
    frames: JByteArray,
    count: jint,
    width: jint,
    height: jint,
    channels: jint,
    mode: jint,
    exposures: JByteArray,
) -> jbyteArray {
    let out = guard(|| {
        let flat = take_bytes(&mut env, frames).ok()?;
        let n = count.max(0) as usize;
        let w = width.max(0) as usize;
        let h = height.max(0) as usize;
        let ch = channels;
        if n == 0 || w == 0 || h == 0 || (ch != 3 && ch != 4) {
            return None;
        }
        let per = w * h * ch as usize;
        if flat.len() != per * n {
            return None;
        }
        let frames: Vec<Vec<u8>> = (0..n)
            .map(|i| flat[i * per..(i + 1) * per].to_vec())
            .collect();
        let merged = if mode == 1 {
            let evs = take_bytes(&mut env, exposures).ok()?;
            let stops: Vec<f32> = if evs.len() == n * 4 {
                evs.chunks_exact(4)
                    .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect()
            } else {
                (0..n).map(|i| i as f32 - (n as f32 - 1.0) / 2.0).collect()
            };
            mtc::merge::merge_hdr(&frames, &stops, w, h, ch as usize)?
        } else {
            mtc::merge::merge_night(&frames, w, h, ch as usize)?
        };
        Some(merged)
    })
    .flatten();
    match out {
        Some(bytes) => give_bytes(&mut env, &bytes),
        None => std::ptr::null_mut(),
    }
}
#[no_mangle]
pub extern "system" fn Java_dev_mtc_Mtc_info(
    mut env: JNIEnv,
    _class: JClass,
    data: JByteArray,
) -> jni::sys::jstring {
    let text = guard(|| {
        let blob = take_bytes(&mut env, data).ok()?;
        let h = container::probe(&blob).ok()?;
        Some(format!(
            "{}x{} {} q{} v{}",
            h.width,
            h.height,
            h.mode_name(),
            h.quality,
            h.version
        ))
    })
    .flatten();
    let text = match text {
        Some(t) => t,
        None => return std::ptr::null_mut(),
    };
    match env.new_string(text) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Luma to `0..255`, honouring the range the device used.
#[inline]
fn luma(y: u8, full: bool) -> i32 {
    let y = y as i32;
    if full {
        y
    } else {
        (((y - 16) * 255) / 219).clamp(0, 255)
    }
}

/// Chroma to a signed offset around zero, honouring the range.
#[inline]
fn chroma(c: u8, full: bool) -> i32 {
    let c = c as i32 - 128;
    if full {
        c
    } else {
        (c * 255) / 224
    }
}

/// `YUV_420_888` to RGBA, BT.601, optionally rotated clockwise.
///
/// Camera2 hands out the same sampling grid for U and V with arbitrary strides,
/// so one pair of strides describes both planes. Returns the pixels and the
/// rotated size.
#[allow(clippy::too_many_arguments)]
pub fn yuv_to_rgba(
    py: &[u8],
    pu: &[u8],
    pv: &[u8],
    w: usize,
    h: usize,
    y_stride: usize,
    uv_stride: usize,
    uv_pixel: usize,
    full_range: bool,
    rotation: u32,
    mirror: bool,
) -> (Vec<u8>, usize, usize) {
    let quarter = (rotation / 90) % 4;
    let (ow, oh) = if quarter % 2 == 0 { (w, h) } else { (h, w) };
    let mut out = vec![0u8; ow * oh * 4];
    for row in 0..h {
        for col in 0..w {
            let yv = luma(py[row * y_stride + col], full_range);
            let idx = (row / 2) * uv_stride + (col / 2) * uv_pixel;
            let (uu, vv) = if idx < pu.len() && idx < pv.len() {
                (chroma(pu[idx], full_range), chroma(pv[idx], full_range))
            } else {
                (0, 0)
            };
            let r = yv + (91881 * vv >> 16);
            let g = yv - (22554 * uu + 46802 * vv >> 16);
            let b = yv + (116130 * uu >> 16);

            let (dr, dc) = match quarter {
                1 => (col, h - 1 - row),
                2 => (h - 1 - row, w - 1 - col),
                3 => (w - 1 - col, row),
                _ => (row, col),
            };
            // A front camera shows a mirror image; flip after the turn so the
            // mirroring is relative to the upright picture.
            let dc = if mirror { ow - 1 - dc } else { dc };
            let o = (dr * ow + dc) * 4;
            out[o] = r.clamp(0, 255) as u8;
            out[o + 1] = g.clamp(0, 255) as u8;
            out[o + 2] = b.clamp(0, 255) as u8;
            out[o + 3] = 255;
        }
    }
    (out, ow, oh)
}

/// A decoded picture with its size in front, which is what Java reads back.
pub fn pack(w: usize, h: usize, rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + rgba.len());
    out.extend_from_slice(&(w as u32).to_be_bytes());
    out.extend_from_slice(&(h as u32).to_be_bytes());
    out.extend_from_slice(rgba);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grey_frame(w: usize, h: usize, luma_value: u8) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        (
            vec![luma_value; w * h],
            vec![128; (w / 2) * (h / 2)],
            vec![128; (w / 2) * (h / 2)],
        )
    }

    #[test]
    fn neutral_chroma_is_grey() {
        let (y, u, v) = grey_frame(8, 8, 180);
        let (rgba, _, _) = yuv_to_rgba(&y, &u, &v, 8, 8, 8, 4, 1, true, 0, false);
        assert_eq!(&rgba[0..4], &[180, 180, 180, 255]);
        assert_eq!(&rgba[4..8], &[180, 180, 180, 255]);
    }

    #[test]
    fn limited_range_is_expanded() {
        let (y, u, v) = grey_frame(4, 4, 16);
        let (black, _, _) = yuv_to_rgba(&y, &u, &v, 4, 4, 4, 2, 1, false, 0, false);
        assert_eq!(black[0], 0);
        let (y, u, v) = grey_frame(4, 4, 235);
        let (white, _, _) = yuv_to_rgba(&y, &u, &v, 4, 4, 4, 2, 1, false, 0, false);
        assert_eq!(white[0], 255);
        // The same luma reads brighter through limited range than through full.
        let (y, u, v) = grey_frame(4, 4, 180);
        let (lim, _, _) = yuv_to_rgba(&y, &u, &v, 4, 4, 4, 2, 1, false, 0, false);
        let (full, _, _) = yuv_to_rgba(&y, &u, &v, 4, 4, 4, 2, 1, true, 0, false);
        assert!(lim[0] > full[0], "{} vs {}", lim[0], full[0]);
    }

    #[test]
    fn chroma_shifts_the_channels_the_right_way() {
        // Pure blue in BT.601: Y=41, U=240, V=110.
        let (w, h) = (4, 4);
        let y = vec![41u8; w * h];
        let u = vec![240u8; (w / 2) * (h / 2)];
        let v = vec![110u8; (w / 2) * (h / 2)];
        let (rgba, _, _) = yuv_to_rgba(&y, &u, &v, w, h, w, w / 2, 1, true, 0, false);
        let (r, g, b) = (rgba[0] as i32, rgba[1] as i32, rgba[2] as i32);
        assert!(b > 200, "blue {} {} {}", r, g, b);
        assert!(r < 60, "red {} {} {}", r, g, b);
        assert!(g < 80, "green {} {} {}", r, g, b);
    }

    #[test]
    fn strided_rows_are_walked_correctly() {
        // Two visible pixels in a three-wide row: the padding must be skipped.
        let y = vec![10u8, 20, 99, 30, 40, 99];
        let u = vec![128u8, 128];
        let v = vec![128u8, 128];
        let (rgba, _, _) = yuv_to_rgba(&y, &u, &v, 2, 2, 3, 1, 1, true, 0, false);
        assert_eq!(rgba[0], 10);
        assert_eq!(rgba[4], 20);
        assert_eq!(rgba[8], 30);
        assert_eq!(rgba[12], 40);
    }

    #[test]
    fn quarter_turns_rotate_and_swap_the_size() {
        // 3x2 source, one distinct grey per pixel, so the mapping is readable.
        let (w, h) = (3usize, 2usize);
        let y = vec![10u8, 20, 30, 40, 50, 60];
        let u = vec![128u8; (w / 2) * (h / 2)];
        let v = vec![128u8; (w / 2) * (h / 2)];
        let go = |rotation: u32| {
            let (rgba, ow, oh) = yuv_to_rgba(&y, &u, &v, w, h, w, w / 2, 1, true, rotation, false);
            (move |row: usize, col: usize| rgba[(row * ow + col) * 4], ow, oh)
        };

        let (px, ow, oh) = go(0);
        assert_eq!((ow, oh), (3, 2));
        assert_eq!((px(0, 0), px(1, 2)), (10, 60));

        // Clockwise: the old top-left is the new top-right, and the size swaps.
        //   40 10
        //   50 20
        //   60 30
        let (px, ow, oh) = go(90);
        assert_eq!((ow, oh), (2, 3));
        assert_eq!((px(0, 0), px(0, 1)), (40, 10));
        assert_eq!((px(1, 0), px(1, 1)), (50, 20));
        assert_eq!((px(2, 0), px(2, 1)), (60, 30));

        // Half turn: opposite corners swap, the size does not.
        let (px, ow, oh) = go(180);
        assert_eq!((ow, oh), (3, 2));
        assert_eq!((px(0, 0), px(1, 2)), (60, 10));

        // Three quarters: the old bottom-left becomes the new top-left.
        let (px, ow, oh) = go(270);
        assert_eq!((ow, oh), (2, 3));
        assert_eq!(px(0, 0), 30);
    }

    #[test]
    fn mirror_flips_horizontally_after_the_turn() {
        let (w, h) = (3usize, 2usize);
        let y = vec![10u8, 20, 30, 40, 50, 60];
        let u = vec![128u8; (w / 2) * (h / 2)];
        let v = vec![128u8; (w / 2) * (h / 2)];
        // No turn, mirrored: the row is reversed.
        let (rgba, ow, _) = yuv_to_rgba(&y, &u, &v, w, h, w, w / 2, 1, true, 0, true);
        assert_eq!(rgba[0], 30);
        assert_eq!(rgba[(ow - 1) * 4], 10);
        // Turned then mirrored:
        //   old            turn          turn + mirror
        //   10 20 30       40 10         10 40
        //   40 50 60       50 20         20 50
        //                  60 30         30 60
        let (rgba, ow, oh) = yuv_to_rgba(&y, &u, &v, w, h, w, w / 2, 1, true, 90, true);
        assert_eq!((ow, oh), (2, 3));
        assert_eq!(rgba[0], 10);
        assert_eq!(rgba[(ow - 1) * 4], 40);
        assert_eq!(rgba[((oh - 1) * ow) * 4], 30);
    }

    #[test]
    fn pack_puts_the_size_first() {
        let rgba = vec![1u8; 3 * 2 * 4];
        let packed = pack(3, 2, &rgba);
        assert_eq!(&packed[0..4], &[0, 0, 0, 3]);
        assert_eq!(&packed[4..8], &[0, 0, 0, 2]);
        assert_eq!(packed.len(), 8 + rgba.len());
        assert_eq!(&packed[8..], &rgba[..]);
    }

    #[test]
    fn camera_frame_round_trips_through_the_codec() {
        // The whole phone path, minus the phone: YUV frame -> RGBA -> .mtc -> RGBA.
        let (w, h) = (64usize, 48usize);
        let mut y = vec![0u8; w * h];
        for row in 0..h {
            for col in 0..w {
                y[row * w + col] = (80 + col * 2) as u8;
            }
        }
        let u = vec![110u8; (w / 2) * (h / 2)];
        let v = vec![140u8; (w / 2) * (h / 2)];
        let (rgba, ow, oh) = yuv_to_rgba(&y, &u, &v, w, h, w, w / 2, 1, true, 90, false);
        assert_eq!((ow, oh), (48, 64));

        let img = mtc::container::Image::new(ow, oh, 4, rgba).unwrap();
        let blob = mtc::container::encode(&img, &mtc::container::Options::default()).unwrap();
        let back = mtc::container::decode(&blob).unwrap();
        assert_eq!((back.width, back.height), (ow, oh));
        let psnr = back.psnr_visible(&img);
        assert!(psnr > 30.0, "psnr {:.2}, {} bytes", psnr, blob.len());

        let packed = pack(back.width, back.height, &back.to_rgba());
        assert_eq!(&packed[0..4], &(ow as u32).to_be_bytes());
        assert_eq!(packed.len(), 8 + ow * oh * 4);
    }
}
