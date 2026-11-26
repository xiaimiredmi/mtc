//! MTC — a self-contained still-image codec.
//!
//! The container, the DCT, the quantiser (including adaptive quantisation), the
//! arithmetic coder and the coefficient model are all in this crate; PNG, JPEG,
//! WebP and BMP read/write come from the `image` crate.
//!
//! ```text
//! mtc::container  header, modes, colour, alpha, encode/decode entry points
//! mtc::codec      8x8 DCT planes, quantisation, reconstruction
//! mtc::entropy    coefficient block coder and its two probability layouts
//! mtc::aq         adaptive quantisation, one multiplier per 16x16 tile
//! mtc::lossless   spatial prediction for the exact paths and for alpha
//! mtc::arith      binary arithmetic coder
//! mtc::transform  DCT matrix, zig-zag scan, quantiser table
//! ```

pub mod aq;
pub mod arith;
pub mod codec;
pub mod container;
/// Multi-frame merging: night mode and HDR, the algorithm halves of them.
pub mod merge;
pub mod entropy;
#[cfg(feature = "raster")]
pub mod imgio;
pub mod lossless;
pub mod transform;
