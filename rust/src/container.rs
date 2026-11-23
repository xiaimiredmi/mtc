//! The `.mtc` container: header, modes, colour transform, alpha and the public
//! encode/decode entry points.
//!
//! Layout (little endian):
//!
//! ```text
//! offset  size  field
//! 0       4     magic  b"MTB1"
//! 4       1     version 1 = flat model, 2 = context model
//! 5       1     mode
//! 6       1     quality 1..100
//! 7       1     flags, bit0 alpha, bit1 adaptive quantisation
//! 8       2     width
//! 10      2     height
//! 12      ...   if FLAG_ALPHA: <u32 png_len> + png bytes
//!               then the arithmetic-coded payload
//! ```

use crate::codec::{self, clip_for_plane, Plane};
use crate::entropy::{LAYOUT_CTX, LAYOUT_FLAT};
use crate::lossless;

pub const MAGIC: &[u8; 4] = b"MTB1";
pub const HDR: usize = 12;
/// Version written by this encoder.
pub const VERSION: u8 = 2;
/// Version the Python encoder wrote; still decoded.
pub const VERSION_LEGACY: u8 = 1;

pub mod mode {
    pub const GRAY_LOSSY: u8 = 0;
    pub const RGB444_LOSSY: u8 = 1;
    pub const RGB420_LOSSY: u8 = 2;
    pub const GRAY_LOSSLESS: u8 = 3;
    pub const RGB_LOSSLESS: u8 = 4;
}

pub const FLAG_ALPHA: u8 = 1;
pub const FLAG_AQ: u8 = 2;

#[derive(Clone, Copy, Debug)]
pub struct Header {
    pub version: u8,
    pub mode: u8,
    pub quality: u8,
    pub flags: u8,
    pub width: usize,
    pub height: usize,
    pub bytes: usize,
}

impl Header {
    pub fn has_alpha(&self) -> bool {
        self.flags & FLAG_ALPHA != 0
    }
    pub fn aq(&self) -> bool {
        self.flags & FLAG_AQ != 0
    }
    pub fn layout(&self) -> u8 {
        if self.version >= VERSION {
            LAYOUT_CTX
        } else {
            LAYOUT_FLAT
        }
    }
    pub fn mode_name(&self) -> &'static str {
        match self.mode {
            mode::GRAY_LOSSY => "gray-lossy",
            mode::RGB444_LOSSY => "rgb444-lossy",
            mode::RGB420_LOSSY => "rgb420-lossy",
            mode::GRAY_LOSSLESS => "gray-lossless",
            mode::RGB_LOSSLESS => "rgb-lossless",
            _ => "unknown",
        }
    }
    pub fn channels(&self) -> usize {
        match self.mode {
            mode::GRAY_LOSSY | mode::GRAY_LOSSLESS => 1,
            _ => 3,
        }
    }
}

/// A decoded still image: `channels` is 1, 3 or 4.
#[derive(Clone, Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
    pub data: Vec<u8>,
}

impl Image {
    pub fn new(width: usize, height: usize, channels: usize, data: Vec<u8>) -> Result<Image, String> {
        if width == 0 || height == 0 {
            return Err("image has no pixels".into());
        }
        if channels != 1 && channels != 3 && channels != 4 {
            return Err(format!("unsupported channel count {}", channels));
        }
        if data.len() != width * height * channels {
            return Err(format!(
                "pixel buffer is {} bytes, expected {}",
                data.len(),
                width * height * channels
            ));
        }
        Ok(Image {
            width,
            height,
            channels,
            data,
        })
    }

    /// RGBA bytes, ready for a texture upload.
    pub fn to_rgba(&self) -> Vec<u8> {
        if self.channels == 4 {
            return self.data.clone();
        }
        let mut out = Vec::with_capacity(self.width * self.height * 4);
        for i in 0..self.width * self.height {
            if self.channels == 1 {
                let v = self.data[i];
                out.extend_from_slice(&[v, v, v, 255]);
            } else {
                out.extend_from_slice(&self.data[i * 3..i * 3 + 3]);
                out.push(255);
            }
        }
        out
    }

    /// PSNR over the picture channels, alpha excluded.
    pub fn psnr(&self, other: &Image) -> f64 {
        self.psnr_masked(other, false)
    }

    /// PSNR over visible pixels only.
    ///
    /// The format deliberately zeroes the colour of pixels below the
    /// transparency threshold, so scoring those pixels against the source
    /// measures nothing but that rule. With `visible_only`, any pixel that is
    /// transparent in either picture is skipped.
    pub fn psnr_visible(&self, other: &Image) -> f64 {
        self.psnr_masked(other, true)
    }

    fn psnr_masked(&self, other: &Image, visible_only: bool) -> f64 {
        let w = self.width.min(other.width);
        let h = self.height.min(other.height);
        if w == 0 || h == 0 {
            return 99.0;
        }
        let ch = self.channels.min(other.channels).min(3);
        let mut se = 0f64;
        let mut count = 0usize;
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if visible_only {
                    let opaque = |img: &Image| {
                        img.channels != 4 || img.data[(y * img.width + x) * 4 + 3] >= 8
                    };
                    if !opaque(self) || !opaque(other) {
                        continue;
                    }
                }
                for c in 0..ch {
                    let a = self.data[i * self.channels + c] as f64;
                    let b = other.data[i * other.channels + c] as f64;
                    se += (a - b) * (a - b);
                    count += 1;
                }
            }
        }
        if count == 0 {
            return 99.0;
        }
        let mse = se / count as f64;
        if mse <= 0.0 {
            99.0
        } else {
            10.0 * (255.0 * 255.0 / mse).log10()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AqMode {
    /// Same quantiser for every tile.
    Off,
    /// Always signal the per-tile multiplier.
    On,
    /// Encode both and keep the smaller file that does not lose quality.
    Auto,
}

impl AqMode {
    pub fn parse(s: &str) -> Result<AqMode, String> {
        match s {
            "off" | "no" | "0" => Ok(AqMode::Off),
            "on" | "yes" | "1" => Ok(AqMode::On),
            "auto" => Ok(AqMode::Auto),
            other => Err(format!("unknown aq mode {}", other)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub quality: u8,
    pub subsample_420: bool,
    pub aq: AqMode,
    pub version: u8,
    /// Spatial prediction instead of the DCT, pixel-exact (modes 3 and 4).
    pub lossless: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            quality: 85,
            subsample_420: true,
            // Measured: adaptive quantisation spends more bytes than it buys
            // back at matched PSNR, so it is opt-in rather than the default.
            aq: AqMode::Off,
            version: VERSION,
            lossless: false,
        }
    }
}

#[inline]
fn align(v: usize, m: usize) -> usize {
    (v + m - 1) / m * m
}

#[inline]
fn fwd(r: i32, g: i32, b: i32) -> (i32, i32, i32) {
    let co = r - b;
    let t = b + (co >> 1);
    let cg = g - t;
    (t + (cg >> 1), co, cg)
}

#[inline]
fn inv(y: i16, co: i16, cg: i16) -> (i32, i32, i32) {
    let (y, co, cg) = (y as i32, co as i32, cg as i32);
    let t = y - (cg >> 1);
    let g = cg + t;
    let b = t - (co >> 1);
    (co + b, g, b)
}

pub fn probe(data: &[u8]) -> Result<Header, String> {
    if data.len() < HDR {
        return Err("truncated header".into());
    }
    if &data[0..4] != MAGIC {
        return Err("not an MTC image (bad magic)".into());
    }
    let version = data[4];
    if version != VERSION_LEGACY && version != VERSION {
        return Err(format!("unsupported container version {}", version));
    }
    let bit =
        |i: usize| u16::from_le_bytes([data[i], data[i + 1]]) as usize;
    Ok(Header {
        version,
        mode: data[5],
        quality: data[6].clamp(1, 100),
        flags: data[7],
        width: bit(8),
        height: bit(10),
        bytes: data.len(),
    })
}

/// Plane shapes and clips for a header, plus the padded luma size.
fn shapes_for(h: &Header) -> Vec<(usize, usize)> {
    match h.mode {
        mode::RGB420_LOSSY => {
            let pw = align(h.width, 16);
            let ph = align(h.height, 16);
            vec![(pw, ph), (pw / 2, ph / 2), (pw / 2, ph / 2)]
        }
        mode::RGB444_LOSSY => {
            let pw = align(h.width, 8);
            let ph = align(h.height, 8);
            vec![(pw, ph); 3]
        }
        _ => {
            let pw = align(h.width, 8);
            let ph = align(h.height, 8);
            vec![(pw, ph)]
        }
    }
}

/// Only the early version 2 files stored alpha as a PNG; the codec itself never
/// needs the image crate, so a build without it simply refuses those files.
#[cfg(not(feature = "raster"))]
fn read_alpha_png(_bytes: &[u8], _w: usize, _h: usize) -> Result<Vec<u8>, String> {
    Err("this build cannot read a PNG alpha segment".into())
}

#[cfg(feature = "raster")]
fn read_alpha_png(bytes: &[u8], w: usize, h: usize) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|e| format!("alpha channel is not a PNG ({})", e))?;
    let g = img.to_luma8();
    if g.width() as usize != w || g.height() as usize != h {
        return Err(format!(
            "alpha is {}x{}, picture is {}x{}",
            g.width(),
            g.height(),
            w,
            h
        ));
    }
    Ok(g.into_raw())
}

/// Encode an image into a complete `.mtc` file.
pub fn encode(img: &Image, opts: &Options) -> Result<Vec<u8>, String> {
    let quality = opts.quality.clamp(1, 100);
    let mut opts = *opts;
    opts.quality = quality;
    if opts.lossless {
        // No quantiser, so adaptive quantisation has nothing to scale.
        return assemble(img, &opts, false);
    }
    match opts.aq {
        AqMode::Off => assemble(img, &opts, false),
        AqMode::On => assemble(img, &opts, true),
        AqMode::Auto => {
            let plain = assemble(img, &opts, false)?;
            let adaptive = assemble(img, &opts, true)?;
            let p_plain = decode(&plain)?.psnr(img);
            let p_adaptive = decode(&adaptive)?.psnr(img);
            // Only pay for the side information when it buys bytes without
            // giving up measurable quality.
            if adaptive.len() < plain.len() && p_adaptive + 0.10 >= p_plain {
                Ok(adaptive)
            } else {
                Ok(plain)
            }
        }
    }
}

fn assemble(img: &Image, opts: &Options, aq_on: bool) -> Result<Vec<u8>, String> {
    let quality = opts.quality;
    let layout = if opts.version >= VERSION {
        LAYOUT_CTX
    } else {
        LAYOUT_FLAT
    };
    let w = img.width;
    let h = img.height;

    // Alpha travels losslessly and separately; fully transparent pixels lose
    // their colour so edges do not fringe.
    let (rgb, alpha) = if img.channels == 4 {
        let mut rgb = Vec::with_capacity(w * h * 3);
        let mut a = Vec::with_capacity(w * h);
        for p in img.data.chunks_exact(4) {
            if p[3] < 8 {
                rgb.extend_from_slice(&[0, 0, 0]);
            } else {
                rgb.extend_from_slice(&p[..3]);
            }
            a.push(p[3]);
        }
        (rgb, Some(a))
    } else {
        (img.data.clone(), None)
    };

    let solid = if img.channels == 1 { 1 } else { 3 };
    let (md, payload) = if opts.lossless {
        // No padding and no quantiser: planes are coded at their real size.
        let planes = if solid == 1 {
            let px = rgb.iter().map(|v| *v as i16).collect::<Vec<i16>>();
            vec![Plane { w, h, px }]
        } else {
            let mut y_plane = vec![0i16; w * h];
            let mut co = vec![0i16; w * h];
            let mut cg = vec![0i16; w * h];
            for p in 0..w * h {
                let (yy, co_v, cg_v) = fwd(
                    rgb[p * 3] as i32,
                    rgb[p * 3 + 1] as i32,
                    rgb[p * 3 + 2] as i32,
                );
                y_plane[p] = yy as i16;
                co[p] = co_v as i16;
                cg[p] = cg_v as i16;
            }
            vec![
                Plane {
                    w,
                    h,
                    px: y_plane,
                },
                Plane { w, h, px: co },
                Plane { w, h, px: cg },
            ]
        };
        let md = if solid == 1 {
            mode::GRAY_LOSSLESS
        } else {
            mode::RGB_LOSSLESS
        };
        (md, lossless::encode_planes(&planes))
    } else if solid == 1 {
        let pw = align(w, 8);
        let ph = align(h, 8);
        let mut px = vec![0i16; pw * ph];
        for y in 0..ph {
            for x in 0..pw {
                px[y * pw + x] = rgb[y.min(h - 1) * w + x.min(w - 1)] as i16;
            }
        }
        let planes = vec![Plane { w: pw, h: ph, px }];
        (
            mode::GRAY_LOSSY,
            codec::encode_planes(&planes, quality, layout, aq_on),
        )
    } else {
        let mult = if opts.subsample_420 { 16 } else { 8 };
        let pw = align(w, mult);
        let ph = align(h, mult);
        let mut y_plane = vec![0i16; pw * ph];
        let (cw, ch) = if opts.subsample_420 {
            (pw / 2, ph / 2)
        } else {
            (pw, ph)
        };
        let mut co = vec![0i16; cw * ch];
        let mut cg = vec![0i16; cw * ch];
        if opts.subsample_420 {
            let mut acc_co = vec![0i32; cw * ch];
            let mut acc_cg = vec![0i32; cw * ch];
            for yy in 0..ph {
                for xx in 0..pw {
                    let p = (yy.min(h - 1) * w + xx.min(w - 1)) * 3;
                    let (a, b, c) =
                        fwd(rgb[p] as i32, rgb[p + 1] as i32, rgb[p + 2] as i32);
                    y_plane[yy * pw + xx] = a as i16;
                    let q = (yy / 2) * cw + xx / 2;
                    acc_co[q] += b;   // acc_co is i32
                    acc_cg[q] += c;
                }
            }
            for i in 0..cw * ch {
                co[i] = (acc_co[i] >> 2) as i16;
                cg[i] = (acc_cg[i] >> 2) as i16;
            }
        } else {
            for yy in 0..ph {
                for xx in 0..pw {
                    let p = (yy.min(h - 1) * w + xx.min(w - 1)) * 3;
                    let (a, b, c) =
                        fwd(rgb[p] as i32, rgb[p + 1] as i32, rgb[p + 2] as i32);
                    y_plane[yy * pw + xx] = a as i16;
                    co[yy * pw + xx] = b as i16;
                    cg[yy * pw + xx] = c as i16;
                }
            }
        }
        let planes = vec![
            Plane {
                w: pw,
                h: ph,
                px: y_plane,
            },
            Plane {
                w: cw,
                h: ch,
                px: co,
            },
            Plane {
                w: cw,
                h: ch,
                px: cg,
            },
        ];
        let m = if opts.subsample_420 {
            mode::RGB420_LOSSY
        } else {
            mode::RGB444_LOSSY
        };
        (
            m,
            codec::encode_planes(&planes, quality, layout, aq_on),
        )
    };

    let mut flags = 0u8;
    if aq_on && !opts.lossless {
        flags |= FLAG_AQ;
    }
    let alpha_blob = match alpha {
        Some(a) => {
            flags |= FLAG_ALPHA;
            let plane = Plane {
                w,
                h,
                px: a.iter().map(|v| *v as i16).collect(),
            };
            Some(lossless::encode_planes(std::slice::from_ref(&plane)))
        }
        None => None,
    };

    let mut out = Vec::with_capacity(HDR + 4 + payload.len());
    out.extend_from_slice(MAGIC);
    out.push(opts.version);
    out.push(md);
    out.push(quality);
    out.push(flags);
    out.extend_from_slice(&(w as u16).to_le_bytes());
    out.extend_from_slice(&(h as u16).to_le_bytes());
    if let Some(png) = alpha_blob {
        out.extend_from_slice(&(png.len() as u32).to_le_bytes());
        out.extend_from_slice(&png);
    }
    out.extend_from_slice(&payload);
    Ok(out)
}

/// Decode a complete `.mtc` file.
pub fn decode(data: &[u8]) -> Result<Image, String> {
    let h = probe(data)?;
    let (offset, alpha) = alpha_segment(data, &h)?;
    let payload = &data[offset..];
    if h.mode == mode::GRAY_LOSSLESS || h.mode == mode::RGB_LOSSLESS {
        let n = if h.mode == mode::GRAY_LOSSLESS { 1 } else { 3 };
        let shapes = vec![(h.width, h.height); n];
        let clips: Vec<_> = (0..n).map(clip_for_plane).collect();
        let planes = lossless::decode_planes(payload, &shapes, &clips);
        let w = h.width;
        let hh = h.height;
        let mut solid: Vec<u8> = Vec::with_capacity(w * hh * n);
        if n == 1 {
            for p in planes[0].px.iter() {
                solid.push((*p).clamp(0, 255) as u8);
            }
        } else {
            for i in 0..w * hh {
                let (r, g, b) = inv(planes[0].px[i], planes[1].px[i], planes[2].px[i]);
                solid.push(r.clamp(0, 255) as u8);
                solid.push(g.clamp(0, 255) as u8);
                solid.push(b.clamp(0, 255) as u8);
            }
        }
        return attach_alpha(solid, alpha, w, hh, n);
    }
    let shapes = shapes_for(&h);
    let clips: Vec<_> = (0..shapes.len()).map(clip_for_plane).collect();
    let planes = codec::decode_planes(
        payload,
        &shapes,
        &clips,
        h.quality,
        h.layout(),
        h.aq(),
    );

    let w = h.width;
    let hh = h.height;
    let mut solid: Vec<u8>;
    let channels;
    match h.mode {
        mode::GRAY_LOSSY => {
            let p = &planes[0];
            solid = Vec::with_capacity(w * hh);
            for y in 0..hh {
                for x in 0..w {
                    solid.push(p.px[y * p.w + x].clamp(0, 255) as u8);
                }
            }
            channels = 1;
        }
        mode::RGB420_LOSSY => {
            let (yp, cop, cgp) = (&planes[0], &planes[1], &planes[2]);
            solid = Vec::with_capacity(w * hh * 3);
            for y in 0..hh {
                for x in 0..w {
                    let c = (y / 2) * cop.w + x / 2;
                    let (r, g, b) = inv(yp.px[y * yp.w + x], cop.px[c], cgp.px[c]);
                    solid.push(r.clamp(0, 255) as u8);
                    solid.push(g.clamp(0, 255) as u8);
                    solid.push(b.clamp(0, 255) as u8);
                }
            }
            channels = 3;
        }
        mode::RGB444_LOSSY => {
            let (yp, cop, cgp) = (&planes[0], &planes[1], &planes[2]);
            solid = Vec::with_capacity(w * hh * 3);
            for y in 0..hh {
                for x in 0..w {
                    let p = y * yp.w + x;
                    let (r, g, b) = inv(yp.px[p], cop.px[p], cgp.px[p]);
                    solid.push(r.clamp(0, 255) as u8);
                    solid.push(g.clamp(0, 255) as u8);
                    solid.push(b.clamp(0, 255) as u8);
                }
            }
            channels = 3;
        }
        mode::GRAY_LOSSLESS | mode::RGB_LOSSLESS => {
            // Handled before the lossy plane loop above.
            return Err("lossless mode reached the lossy path".into());
        }
        other => return Err(format!("unknown mode {}", other)),
    }

    attach_alpha(solid, alpha, w, hh, channels)
}

/// Locate the main payload, decoding the alpha segment on the way past.
fn alpha_segment(data: &[u8], h: &Header) -> Result<(usize, Option<Vec<u8>>), String> {
    if !h.has_alpha() {
        return Ok((HDR, None));
    }
    if data.len() < HDR + 4 {
        return Err("truncated alpha length".into());
    }
    let len = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
    if data.len() < HDR + 4 + len {
        return Err("truncated alpha payload".into());
    }
    let blob = &data[HDR + 4..HDR + 4 + len];
    Ok((HDR + 4 + len, Some(decode_alpha(blob, h.width, h.height)?)))
}

/// Decode straight to a smaller picture.
///
/// The coefficient stream has to be walked in full, because the model is
/// sequential, but the output is sampled while the planes are converted, so the
/// full-size RGBA buffer - the biggest single allocation in the decoder - never
/// exists. A 12 MP photo becomes a 400 px thumbnail without ever holding 48 MB of
/// RGBA, which is what keeps the grid alive on a phone. Lossless modes fall back
/// to a full decode; they are not the camera path.
pub fn decode_downsized(data: &[u8], max_side: usize) -> Result<Image, String> {
    let h = probe(data)?;
    if max_side == 0 || (h.width <= max_side && h.height <= max_side) {
        return decode(data);
    }
    if h.mode == mode::GRAY_LOSSLESS || h.mode == mode::RGB_LOSSLESS {
        return decode(data);
    }
    let (offset, alpha) = alpha_segment(data, &h)?;
    let payload = &data[offset..];
    let shapes = shapes_for(&h);
    let clips: Vec<_> = (0..shapes.len()).map(clip_for_plane).collect();
    let planes = codec::decode_planes(payload, &shapes, &clips, h.quality, h.layout(), h.aq());

    let step = ((h.width.max(h.height) + max_side - 1) / max_side).max(1);
    let ow = (h.width + step - 1) / step;
    let oh = (h.height + step - 1) / step;
    let solid_channels = if h.mode == mode::GRAY_LOSSY { 1 } else { 3 };
    let mut solid = Vec::with_capacity(ow * oh * solid_channels);
    match h.mode {
        mode::GRAY_LOSSY => {
            let p = &planes[0];
            for y in (0..h.height).step_by(step) {
                for x in (0..h.width).step_by(step) {
                    solid.push(p.px[y * p.w + x].clamp(0, 255) as u8);
                }
            }
        }
        mode::RGB420_LOSSY => {
            let (yp, cop, cgp) = (&planes[0], &planes[1], &planes[2]);
            for y in (0..h.height).step_by(step) {
                for x in (0..h.width).step_by(step) {
                    let c = (y / 2) * cop.w + x / 2;
                    let (r, g, b) = inv(yp.px[y * yp.w + x], cop.px[c], cgp.px[c]);
                    solid.push(r.clamp(0, 255) as u8);
                    solid.push(g.clamp(0, 255) as u8);
                    solid.push(b.clamp(0, 255) as u8);
                }
            }
        }
        mode::RGB444_LOSSY => {
            let (yp, cop, cgp) = (&planes[0], &planes[1], &planes[2]);
            for y in (0..h.height).step_by(step) {
                for x in (0..h.width).step_by(step) {
                    let p = y * yp.w + x;
                    let (r, g, b) = inv(yp.px[p], cop.px[p], cgp.px[p]);
                    solid.push(r.clamp(0, 255) as u8);
                    solid.push(g.clamp(0, 255) as u8);
                    solid.push(b.clamp(0, 255) as u8);
                }
            }
        }
        other => return Err(format!("mode {} cannot be scaled", other)),
    }
    let alpha = alpha.map(|a| {
        let mut small = Vec::with_capacity(ow * oh);
        for y in (0..h.height).step_by(step) {
            for x in (0..h.width).step_by(step) {
                small.push(*a.get(y * h.width + x).unwrap_or(&255));
            }
        }
        small
    });
    attach_alpha(solid, alpha, ow, oh, solid_channels)
}

/// Append the alpha channel, if the file carried one.
fn attach_alpha(
    solid: Vec<u8>,
    alpha: Option<Vec<u8>>,
    w: usize,
    h: usize,
    channels: usize,
) -> Result<Image, String> {
    let has_alpha = alpha.is_some();
    let data = match alpha {
        Some(a) => {
            let mut out = Vec::with_capacity(w * h * 4);
            for i in 0..w * h {
                if channels == 1 {
                    let v = solid[i];
                    out.extend_from_slice(&[v, v, v]);
                } else {
                    out.extend_from_slice(&solid[i * 3..i * 3 + 3]);
                }
                out.push(a.get(i).copied().unwrap_or(255));
            }
            out
        }
        None => solid,
    };
    let channels = if has_alpha { 4 } else { channels };
    Image::new(w, h, channels, data)
}

/// The alpha segment: the lossless residual coder, or a PNG for the early
/// version 2 files that stored it that way.
fn decode_alpha(blob: &[u8], w: usize, h: usize) -> Result<Vec<u8>, String> {
    if blob.starts_with(b"\x89PNG") {
        return read_alpha_png(blob, w, h);
    }
    let planes = lossless::decode_planes(blob, &[(w, h)], &[clip_for_plane(0)]);
    Ok(planes[0].px.iter().map(|v| (*v).clamp(0, 255) as u8).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(w: usize, h: usize) -> Image {
        let mut px = Vec::with_capacity(w * h * 3);
        for y in 0..h {
            for x in 0..w {
                px.push(((x * 7 + y * 3) % 256) as u8);
                px.push(((x * 2 + y * 11) / 2 % 256) as u8);
                px.push(((x + y) * 3 % 256) as u8);
            }
        }
        Image::new(w, h, 3, px).unwrap()
    }

    fn roundtrip(opts: Options) {
        let img = gradient(48, 40);
        let blob = encode(&img, &opts).unwrap();
        let h = probe(&blob).unwrap();
        assert_eq!((h.width, h.height), (48, 40));
        let back = decode(&blob).unwrap();
        assert_eq!((back.width, back.height, back.channels), (48, 40, 3));
        let p = img.psnr(&back);
        assert!(p > 25.0, "psnr {:.2} version {}", p, opts.version);
    }

    #[test]
    fn version2_context_roundtrip() {
        roundtrip(Options {
            quality: 85,
            ..Options::default()
        });
    }

    #[test]
    fn version2_with_aq() {
        roundtrip(Options {
            quality: 70,
            aq: AqMode::On,
            ..Options::default()
        });
    }

    #[test]
    fn version1_legacy_roundtrip() {
        roundtrip(Options {
            version: VERSION_LEGACY,
            aq: AqMode::Off,
            ..Options::default()
        });
    }

    #[test]
    fn rgba_alpha_is_exact() {
        let (w, h) = (24usize, 16usize);
        let mut px = Vec::with_capacity(w * h * 4);
        for y in 0..h {
            for x in 0..w {
                px.push((x * 9) as u8);
                px.push((y * 13) as u8);
                px.push(200);
                px.push(if (x + y) % 3 == 0 { 0 } else { (x * 17 % 256) as u8 });
            }
        }
        let img = Image::new(w, h, 4, px).unwrap();
        let blob = encode(&img, &Options::default()).unwrap();
        let back = decode(&blob).unwrap();
        assert_eq!(back.channels, 4);
        for i in 0..w * h {
            assert_eq!(back.data[i * 4 + 3], img.data[i * 4 + 3], "alpha {}", i);
        }
    }

    #[test]
    fn both_versions_carry_alpha_exactly() {
        let (w, h) = (24usize, 16usize);
        let mut px = Vec::with_capacity(w * h * 4);
        for y in 0..h {
            for x in 0..w {
                px.push((x * 9) as u8);
                px.push((y * 13) as u8);
                px.push(200);
                px.push(((x * 17 + y * 5) % 256) as u8);
            }
        }
        let img = Image::new(w, h, 4, px).unwrap();
        for version in [VERSION, VERSION_LEGACY] {
            let blob = encode(
                &img,
                &Options {
                    version,
                    ..Options::default()
                },
            )
            .unwrap();
            let back = decode(&blob).unwrap();
            assert_eq!(back.channels, 4, "version {}", version);
            for i in 0..w * h {
                assert_eq!(back.data[i * 4 + 3], img.data[i * 4 + 3], "alpha {}", i);
            }
        }
    }

    #[test]
    fn transparent_pixels_are_not_scored() {
        let (w, h) = (16usize, 16usize);
        let mut a = Vec::with_capacity(w * h * 4);
        let mut b = Vec::with_capacity(w * h * 4);
        for i in 0..w * h {
            let alpha = if i % 2 == 0 { 0 } else { 255 } as u8;
            a.extend_from_slice(&[10, 10, 10, alpha]);
            // Same visible pixels, wildly different colour where transparent.
            b.extend_from_slice(&[if alpha == 0 { 250 } else { 10 }, 10, 10, alpha]);
        }
        let ia = Image::new(w, h, 4, a).unwrap();
        let ib = Image::new(w, h, 4, b).unwrap();
        assert!(ia.psnr(&ib) < 10.0, "plain psnr should see the difference");
        assert_eq!(ia.psnr_visible(&ib), 99.0);
    }

    #[test]
    fn gray_roundtrip() {
        let (w, h) = (32usize, 24usize);
        let mut px = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                px.push(((x * 5 + y * 9) % 256) as u8);
            }
        }
        let img = Image::new(w, h, 1, px).unwrap();
        let blob = encode(
            &img,
            &Options {
                quality: 90,
                aq: AqMode::Off,
                ..Options::default()
            },
        )
        .unwrap();
        let back = decode(&blob).unwrap();
        assert_eq!(back.channels, 1);
        assert!(img.psnr(&back) > 30.0);
    }

    #[test]
    fn aq_auto_never_loses_to_plain() {
        let img = gradient(64, 64);
        let auto = encode(
            &img,
            &Options {
                aq: AqMode::Auto,
                ..Options::default()
            },
        )
        .unwrap();
        let off = encode(
            &img,
            &Options {
                aq: AqMode::Off,
                ..Options::default()
            },
        )
        .unwrap();
        let p_auto = decode(&auto).unwrap().psnr(&img);
        let p_off = decode(&off).unwrap().psnr(&img);
        assert!(auto.len() <= off.len() || p_auto >= p_off - 0.11);
    }
}
