# The MTC site

A single page that is also a working encoder: the codec is compiled to
WebAssembly and the browser runs it for real, on whatever image you drop in. No
framework, no CDN, no build step for the front end.

```
index.html    the page
styles.css    the look: aurora background, glass cards, reveal-on-scroll
codec.js      loads mtc.wasm, wraps the four C exports, parses the header
app.js        the demo: split slider, quality, rate/distortion curve, downloads
mtc.wasm      built from ../wasm, 104 KB
assets/       sample image, icon, preview, one Python-era .mtc to decode
build.ps1     builds the wasm and copies it here
```

## Build and look at it

```powershell
powershell -ExecutionPolicy Bypass -File site\build.ps1          # compile the wasm
powershell -ExecutionPolicy Bypass -File site\build.ps1 -Serve   # + http://127.0.0.1:8765
```

`build.ps1` needs `rustup target add wasm32-unknown-unknown` once. There is no
`node`, no bundler and no `package.json`: the page is three files and a `.wasm`.

## Light and dark

Light is the default; dark follows the operating system until the visitor clicks the
switch in the navigation bar, after which their choice is remembered in
`localStorage` and the system is ignored. Every colour is a custom property, so the
two palettes are one block each in `styles.css` — nothing else mentions a colour.

`?theme=light` / `?theme=dark` force one, which is how the headless check screenshots
both without pretending to click.

## The download button picks a target

`#dlMain` (and the hero button) point at whatever this machine can actually run,
decided from the user agent in `detectOS()`:

| Detected | Goes to |
| --- | --- |
| Windows | `releases/latest/download/MtcSetup.exe` (2.8 MB installer) |
| Android | `releases/latest/download/mtc-camera.apk` |
| macOS / iOS / Linux / anything else | the releases page, because the project does not build for it |

The note under the button says which system it recognised, so a wrong guess is
visible rather than mysterious. `?os=windows` / `?os=linux` force a target for the
headless check.

## What the demo actually proves

The page is not a mock-up with precomputed numbers. Drag in a photo and it:

1. draws the image into a canvas and reads the pixels,
2. hands them to `mtcodec`'s encoder through `mtc_encode` (WebAssembly),
3. decodes the result back with `mtc_decode`,
4. computes PSNR against the original in JavaScript.

For the bundled `assets/sample.png` at q85, 4:2:0, the page reports **34 304
bytes / 32.57 dB**. The command line, on the same file, reports **34 304 bytes /
32.57 dB** — the same numbers, because it is the same code. That equality is the
point: the WebAssembly build is the codec, not a re-implementation of it.

`assets/legacy.mtc` is a version 1 file written by the old Python encoder; load it
to see the decoder handle a stream this build never wrote.

## The two flags used for checking

The page is verified in a headless browser, which means no clicking:

* `?auto=1` — load the sample, encode it, show the numbers, skip the animations.
* `?auto=mtc` — load `assets/legacy.mtc` through the decode path instead.

```powershell
& "$env:ProgramFiles (x86)\Microsoft\Edge\Application\msedge.exe" --headless `
  --window-size=1440,2600 --virtual-time-budget=25000 `
  --screenshot=shot.png "http://127.0.0.1:8765/?auto=1"
```

## Notes

* The demo caps the working size at 1280 px on the long side, so a phone photo
  does not spend seconds inside WebAssembly. The full size is what the command
  line and the Android app use.
* A fully opaque image is handed to the codec as three channels; storing an alpha
  channel that is 255 everywhere would only cost bytes.
* `mtc.wasm` is generated. `build.ps1` refreshes it; it is committed so the page
  works from a clone without a Rust toolchain.
