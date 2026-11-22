# MTC — a self-built image codec

> Self-built image codec in Rust: 8×8 DCT, arithmetic coding, adaptive
> quantisation, lossless and exact alpha, plus an egui viewer.
> 自研 Rust 图片编解码器：8×8 DCT + 算术编码 + 自适应量化，支持无损与精确 alpha，
> 自带 egui 查看器。

That line is the repository's description on GitHub. If you change it in one
place, change it in the other.

MTC is a from-scratch still-image codec. Its pixels never go through JPEG, WebP
or PNG: the transform, quantiser, adaptive quantisation, arithmetic coder,
colour conversion and the container are all its own. The `image` crate is used
only to read and write ordinary image files.

One format, one extension: **`.mtc`**. Lossy and lossless share the container,
with optional alpha and 4:2:0 / 4:4:4 chroma.

## What it does

- 8×8 DCT with a frequency-tilted quantiser and an AC dead zone.
- YCoCg colour, optional 4:2:0, chroma quantised harder.
- DC predicted from the blocks above and left by a median edge detector.
- Adaptive binary arithmetic coding over zig-zag coefficients.
- Lossless mode (spatial predictor + the same arithmetic coder), pixel-exact.
- Exact alpha channel, coded with the lossless path, for lossy or lossless RGB.
- Optional per-tile adaptive quantisation (see the measurements below).
- White viewer with mouse-wheel zoom, drag to pan, double-click to fit.

## Install

```powershell
cd rust
cargo build --release                      # mtcodec + mtbench
cargo build --release --features viewer    # + mtview

copy target\release\mtcodec.exe,target\release\mtbench.exe,target\release\mtview.exe ..\bin\
powershell -ExecutionPolicy Bypass -File ..\deploy\register.ps1
```

The last step registers `.mtc` to open with `mtview`, clears the old video
container registrations, and puts `bin\` first on the user `Path` (no
administrator needed). `deploy\register.ps1 -Unregister` undoes all of it.

## Use

```powershell
# PNG / JPG / WEBP / BMP -> .mtc
mtcodec encode photo.jpg photo.mtc -q 85

# .mtc -> PNG / JPG / WEBP by output extension
mtcodec decode photo.mtc out.jpg

# header only
mtcodec info photo.mtc

# the viewer (also opens PNG/JPEG/WebP/BMP)
mtview photo.mtc
```

Lossless, and 4:4:4 chroma:

```powershell
mtcodec encode scan.png scan.mtc --lossless
mtcodec encode photo.jpg photo.mtc -q 90 -s 444
```

## Measured

- 2048×1152 photograph, `-q 85`, 4:2:0, version 2: 397 KB at 33.63 dB, about
  115 ms to encode and 130 ms to decode, one core, release build.
- Version 2 against version 1 at matched PSNR: 11.3% fewer bytes over four
  512×512 photographs and six quality settings each.
- Lossless: 512×512 photo at 367 KB (11.2 bpp), decode is bit-exact.
- Adaptive quantisation is opt-in: it buys about +0.3 dB at a fixed `-q` for
  +15% bytes, which is a loss at matched PSNR.

The numbers, the bench tool and the comparison script live in `rust/`.

## Install on Windows

`installer\MtcSetup.exe` is a single 2.8 MB setup: the three binaries, a `.mtc` file
association, the folder on `PATH`, start menu shortcuts and an Add/Remove Programs
entry. Per-user, so no administrator prompt.

```powershell
installer\MtcSetup.exe                             # wizard
installer\MtcSetup.exe /S                          # silent, defaults
installer\MtcSetup.exe /S /dir "D:\Tools\MTC"       # silent, chosen folder
installer\MtcSetup.exe /uninstall                   # remove everything again
```

It is built by `installer\build.ps1` with the C# compiler that ships with .NET
Framework — no Inno Setup, no WiX, no NSIS, and no dependency for the person
running it either. See [installer/README.md](installer/README.md).

## The site

`site/` is one page that is also a working encoder (published at `mtc-xiaimiredmi.*`
once the domain is pointed at it — see [docs/DEPLOY.md](docs/DEPLOY.md)): the codec is compiled to
WebAssembly and the browser runs it on whatever image you drop in, so the numbers
on the page are produced by the same Rust code the command line uses.

```powershell
powershell -ExecutionPolicy Bypass -File site\build.ps1 -Serve   # http://127.0.0.1:8765
```

Light and dark themes, remembered per visitor, and a download button that reads the
user agent: on Windows it goes straight to `MtcSetup.exe` in the latest release, on
Android to the APK, and on anything else to the releases page rather than a dead
link.

For the bundled sample at q85 the page reports 34 304 bytes / 32.57 dB; the
command line reports the same two numbers for the same file. `.github/workflows/site.yml`
publishes `site/` to GitHub Pages once Pages is switched to "GitHub Actions" in
the repository settings.

![the site](site/assets/preview.jpg)

## Repository layout

```
rust/src/container.rs  header, modes, colour, alpha, encode/decode entry points
rust/src/codec.rs      8x8 DCT planes, quantisation, reconstruction
rust/src/entropy.rs    coefficient block coder and residual model
rust/src/aq.rs         adaptive quantisation, one multiplier per 16x16 tile
rust/src/lossless.rs   spatial prediction for the exact paths and alpha
rust/src/arith.rs      binary arithmetic coder
rust/src/transform.rs  DCT matrix, zig-zag scan, quantiser table
rust/src/bin/          mtbench (measurements) and mtview (egui viewer)
deploy/register.ps1    file association and PATH, per user
installer/             the Windows setup (MtcSetup.cs, build.ps1, the .exe)
site/                  the WebAssembly demo page (index.html, codec.js, mtc.wasm)
wasm/                  the thin cdylib that exposes the codec to JavaScript
android/               camera + gallery app, no Gradle
deploy/mtc.ico         icon the association uses
samples/               three .mtc files the removed encoder wrote: two photographs
                       and a 960x540 picture with an alpha channel, plus that
                       picture's source PNG
```

Version 1 streams — the layout the removed Python encoder wrote — still decode
bit-exactly, and `rust/tests/data/` keeps three of them as fixtures.

## Status

The image path is the product and the Rust codec in `rust/` owns the format.
The Python implementation has been removed; the migration is complete. What is
next is size and speed work: the in-house codec still trails WebP on rate at
matched PSNR, which is the number to beat.

## Licence

MIT, see `LICENSE`.
