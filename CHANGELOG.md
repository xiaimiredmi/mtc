# Changelog

All notable changes to this project. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
`MAJOR.MINOR.PATCH`. Two languages per release, English first, 中文在后。

---

## [1.5.0] — 2026-11-22

### 新增

* **`installer\MtcSetup.exe`** —— 2.8 MB 的 Windows 安装程序：命令行三个程序、查看器、
  `.mtc` 文件关联、用户 PATH、开始菜单快捷方式、"应用和功能"里的卸载项，全在一次点击里。
  按用户安装（`%LOCALAPPDATA%\Programs\MTC` + `HKCU`），不需要管理员权限。
* 静默安装与静默卸载：`/S`、`/dir <path>`、`/nopath`、`/noassoc`、`/nostartmenu`、
  `/extract <dir>`、`/uninstall`。卸载不带参数也行，安装位置从注册表读回。

### 网站

* 明暗两套主题，默认跟随系统，点顶栏 ☾/☀ 切换后记住选择（`localStorage`）。
* 下载按钮按 `navigator.userAgent` 认系统：Windows 直接给 `MtcSetup.exe`，
  Android 直接给 APK，其它系统去 GitHub 发布页——不会出现"点了没反应"的死链。
* 新增 GitHub Release `v1.5.0`，附 `MtcSetup.exe` 与 `mtc-camera.apk`。

### 实现

* 用 .NET Framework 自带的 `csc.exe` 编译，不引入 Inno Setup / WiX / NSIS；二进制以
  **逐文件 deflate 资源** 嵌入（`DeflateStream` 在 `System.dll` 里，而 `ZipArchive`
  需要的程序集 `csc` 按简单名不一定解析得到）。
* 卸载用"退出后由批处理删除目录"的方式自删除——Windows 不允许删除正在运行的镜像，
  而这个卸载器本身就是被删文件之一。

### 修掉的两个坑

* `uninstall.exe` 原先只认 `/uninstall` 参数，于是"应用和功能"里点卸载（最多带个 `/S`）
  会变成**重新安装**；而且文件名判断在参数解析**之后**，解析时仍按默认目录处理，
  结果注册表清理了、真正的安装目录却留着。现在文件名就是意图，且在解析之前判定。
* 卸载器只有在目标目录里找到 `mtview.exe` 时才删文件，避免路径写错时带走别的目录。
## [1.4.0] — 2026-11-17

### Added

* **Lossless mode in Rust.** `MODE_GRAY_LOSSLESS` and `MODE_RGB_LOSSLESS` are
  in: median-edge-detector prediction, residuals through the same arithmetic
  coder, YCoCg for colour. Decode is pixel-exact, and the encoder is
  byte-identical to the reference one for the same input (154 246 bytes on both
  sides for a 512×512 photo), so the streams are interchangeable with the files
  the old encoder produced.
* **Alpha coded with the lossless path.** The alpha channel used to travel as a
  PNG blob in version 2 and as a Python-only residual stream in version 1. Both
  now go through the same residual coder, which means the Rust codec can read
  every alpha file, including the ones written before it existed, and write
  smaller ones. A PNG blob is still accepted when reading.
* **`deploy/register.ps1`.** Registers `.mtc` to open with `mtview`, points the
  icon at `deploy/mtc.ico`, removes the old `.mtb` / `MTB.*` registrations, puts
  `bin\` first on the user `Path` and broadcasts the environment change. Per
  user, no administrator, `-Unregister` reverses it.
* **Rust CI.** `.github/workflows/rust.yml` runs the tests on Windows and Linux
  and builds the viewer.

### Removed

* **The whole Python implementation.** `mtcodec/` (including `mtfastdec.dll`),
  `pyproject.toml`, `requirements.txt`, `MTBViewer.spec`, `tools/`, `bench/`,
  `tests/`, `native/`, the Python launchers in `deploy/`, the three Python-only
  documents (`docs/BUILD.md`, `docs/CONVERT.md`, `docs/PUBLISH.md`), the two
  experiment reports, the release archives and the `mtc.exe` bundle. The package
  is uninstalled, so `mtcodec` on `Path` and `python -m mtcodec` both no longer
  exist. The migration started in `rust/README.md` is finished.
* Three of its files survive as test fixtures in `rust/tests/data/`: a lossy
  version 1 stream, a lossless one and an alpha one, each checked against a
  pixel hash taken from the Python decoder before it was removed.

### Fixed

* **Chinese file names rendered as boxes in the viewer.** egui ships no CJK
  glyphs, so `图片信息_lossless.mtc` in the header came out as empty boxes. The
  viewer now borrows a system font (YaHei, DengXian, SimHei or SimSun, whichever
  is installed) as the fallback face.
* **Double-clicking a `.mtc` opened the old viewer.** The `.mtc` class pointed
  at `pythonw.exe deploy\mtb_launcher.py`, which no longer existed; it now
  points at `bin\mtview.exe`. Verified through the shell association, not just by
  reading the registry.

### Measured (not a guarantee)

* Lossless, 512×512 photo: 367 361 bytes (11.2 bpp), decode bit-exact.
* Lossless of a 2048×1152 photo: 621 313 bytes for the version 1 file in
  `Downloads`; the Rust decoder reproduces it bit-for-bit against the Python
  decoder.
* Tests: 32 unit, 5 compatibility, 9 viewer geometry.

## [1.3.0] — 2026-11-17

### Added

* **The Rust codec owns the format.** `rust/` grows from a proof of concept into
  a library (`container`, `codec`, `entropy`, `aq`, `arith`, `transform`,
  `imgio`) with three binaries: `mtcodec`, `mtbench` and `mtview`. It writes a
  new **version 2** stream: an AC dead zone, median-edge-detector prediction of
  the DC coefficient from the blocks above and left, and a `last` symbol
  conditioned on the previous block's sparsity. At matched PSNR that is
  **−11.35 %** over the version 1 stream on four 512×512 photographs and six
  quality settings each (`rust/results/rd_final.csv`).
* **Adaptive quantisation.** One quantiser multiplier per 16×16 tile, seven
  levels from 0.70× to 1.40×, chosen by the encoder from tile activity against
  the picture median, delta-coded against the previous tile and tapered towards
  low frequencies. Because an intra frame gives the decoder no reference plane,
  the level is signalled — it costs well under a byte per tile. Opt-in with
  `--aq on`; `--aq auto` encodes both and keeps the smaller file that does not
  lose quality.
* **`mtview`, the egui viewer.** A white window, the picture fitted to the
  window, mouse wheel zoom around the cursor between fit and 8×, drag to pan,
  double-click or `1` back to fit, Esc to close. The header shows mode, quality
  and size for `.mtc` files; PNG/JPEG/WebP/BMP open as well, which makes it
  usable for eyeballing a decode against its source. Above 1:1 the texture is
  nearest-neighbour, so block edges and ringing are visible as they are.
* **`mtbench` and `bench/rd_compare.py`.** The first measures bytes, bpp, PSNR
  and encode/decode time per image, quality and stream shape; the second
  compares two runs **at matched PSNR**, because two encoder settings cannot be
  compared at the same `-q` — whichever spends more bits scores higher and looks
  better than it is.
* **`tests/data/legacy_v1.mtc`.** A file the Python encoder wrote, decoded in a
  test and compared against a pixel hash taken from the Python decoder.

### Fixed

* **Rust decoded every existing file transposed.** The block was gathered as
  `blk[x][y]` and the DCT was applied to match, so each 8×8 block came out
  rotated by 90° relative to the specification — while the Rust encoder and
  decoder agreed with each other, which is why the round-trip tests passed. All
  version 1 files now decode bit-exactly, and the fixture test fails if the axes
  ever swap again.

### Changed

* Version 2 is what the Rust encoder writes; `--v1` still produces the legacy
  stream and stays a faithful reproduction (plain round-to-nearest, left-only DC
  prediction, flat model), which is what keeps the migration honest.
* Adaptive quantisation is **not** the default: it buys about +0.3 dB at a fixed
  `-q` for +15 % bytes, and at matched PSNR it costs **+8.5 %**. `--aq auto`
  therefore rejects it on all four test pictures. It stays in the format for
  perceptual work.
* The `last` context was measured at −0.45 %; two further contexts (band
  significance and DC class) were measured at +1.4 % and −0.1 % and removed.

### Measured (not a guarantee)

* 2048×1152 photograph, `-q 85`, 4:2:0, version 2: 397 KB at 33.63 dB, 115 ms
  encode, 130 ms decode (one core, release build). Version 1 at the same `-q`:
  457 KB at 34.22 dB.
* Rate at matched PSNR, version 2 versus version 1: photo_a −5.3 %,
  photo_b −15.3 %, photo_c −8.5 %, render_d −15.8 %.
* Tests: 28 unit, 2 compatibility, 9 viewer geometry.

## [1.2.2] — 2026-11-15

### Added

* **Streaming playback for long clips.** The viewer used to decode the whole
  container into RAM before showing anything: a 1.84 GB, 5696-frame 1080p `.mtc`
  needed ~35 GB of RGB buffers, so it spent minutes decoding and then died in a
  `MemoryError` dialog. `video_codec.iter_frames()` is now a generator that yields
  one frame at a time, and `viewer.StreamFrames` runs it on a background thread
  while keeping only the most recent 48 frames (a sliding window). `Player` reads
  frames from that window, shows the previous frame when the plan-ahead frame is
  not ready yet, and retries shortly instead of spinning the clock. Opening the
  same 1.84 GB file now reaches the first frame in **~2 s** with flat memory, and
  playback speed is bounded by decode speed only.
* **Native Rust encoder.** Encoding used to be the slow half of the codec: a
  4K frame took 11.2 s and a 1080p frame 3.18 s in pure Python, i.e. 45 minutes
  for 240 4K frames and roughly 22 hours for a 14-minute 1080p clip. The
  arithmetic coder, context modelling, FDCT/quantisation and inverse
  DCT/reconstruction now live in the same `mtfastdec.dll`
  (`mt_enc_new`, `mt_enc_plane`, `mt_enc_flush`, `mt_enc_quant_plane`,
  `mt_enc_recon_plane`). Measured: 4K **11.2 → 1.53 s/frame**, 1080p
  **3.18 → 0.43 s/frame**, a **7.3× speed-up**, with the entropy stage now only
  14 % of a frame (1080p: prep 71 ms, quantise 115 ms, entropy 42 ms,
  reconstruct 66 ms, RGB pack 2.5 ms).
* **Byte-identical output.** Same frames and same initial probabilities through
  the native and the pure-Python path produce the same md5
  (`3a2428a5e438718cb0117b562fbfd293`, 67 547 bytes), so every existing `.mtc`
  stays readable and both halves of the codec stay in lockstep. The reconstruction
  fed back into prediction is bit-exact against the decoder (max diff 0), and
  real footage at `-q 100` measures **50.5–53.7 dB PSNR** (max sample error 11–26).
* **Streaming conversion.** `convert` used to read the whole clip into memory
  (a 14-minute 1080p clip needs ≈150 GB) and kept every reconstructed frame
  alive, so long clips ended in an allocation failure. `encode_video` now takes
  any iterator, `io_bridge.iter_video_frames` pulls one frame at a time out of
  ffmpeg, reconstructed frames are only retained when a caller asks for them,
  and `convert` has a dedicated streaming path. Resident memory on that clip
  drops to ≈300 MB and the encoder prints progress every 100 frames.
* **No needless rescale.** When the source is already the target size the
  `scale` filter is dropped from the ffmpeg pipeline.

## [1.2.1] — 2026-11-15

### Added

* **Full player controls.** Speed 0.5× / 1× / 1.5× / 2×, a draggable seek bar,
  volume 0–100, loop, and close-when-finished. Space toggles play/pause, ←/→
  seek one second, ↑/↓ change volume, Esc closes.
* **Native Rust decoder.** The whole decode pipeline — entropy decoding,
  dequantisation, inverse DCT, prediction, clipping, YCoCg → RGB — runs in a
  Rust cdylib called through a C ABI, four FFI crossings per frame instead of
  hundreds of millions per clip. Measured on this machine: `demo.mtc`
  64.6 → 2.5 ms per frame (26×), `q_360p.mtc` (120 frames, 638×360)
  21.28 s → 0.90 s (24×), `q_720p.mtc` (120 frames, 1278×720) 64.84 s →
  2.43 s (27×). Entropy decoding stays serial (one bitstream, one adaptive
  context chain); the IDCT and RGB pack fan out across cores. Output is
  unchanged: `demo.mtc` is bit-for-bit equal, the longer clips differ on
  0.004 % of samples by one grey level. Missing `mtfastdec.dll` (or
  `MTCODEC_NO_NATIVE=1`) falls back to pure Python with no behaviour change.
* **Real audio speed and volume.** The embedded PCM is resampled and scaled
  with numpy, written to a temporary WAV and handed to the OS player
  (`winsound` on Windows, `aplay` on Linux, `afplay` on macOS), so speed and
  gain need no extra dependency (no pygame, no sounddevice).
* **Pitch-preserving speed.** Speed now runs through ffmpeg's `atempo` filter
  on the whole track, so 0.5× / 1.5× / 2× change tempo without touching the
  pitch. Verified with a 440 Hz tone: the spectral peak stays at 440 Hz at all
  four speeds (0.5× 1.97 s, 1.5× 0.67 s, 2× 0.50 s). Without ffmpeg the player
  falls back to linear interpolation and the window says the pitch will move.
  Speed WAVs are built on a worker thread and cached per speed, so switching
  speed never blocks the UI.
* **Quiet start-up.** The window stays hidden while the clip decodes and appears
  only once the first frame is ready, so no splash text flashes past and no
  "unpacking" moment is visible — the process is only traceable in Task Manager.
  `MTCODEC_VIEWER_SPLASH=1` restores the old progress splash (bar, percentage and
  a live preview thumbnail); `decode_video` still drives it through its
  `progress(i, nframes, frame)` callback.

### Changed

* **Playback clock is monotonic.** Frame timing is derived from
  `time.monotonic()` and scheduled to the next frame boundary, instead of
  chaining a fixed `after(1000/fps)`, so a long clip no longer drifts slower
  and slower.
* **Resolution-preserving display.** Frames keep their decoded size (only
  scaled down once above 1600 px) and are fitted to the window with LANCZOS,
  so the window can be dragged larger or smaller and zooming in no longer
  turns detail into mush. Still images do the same and open at their native
  resolution.
* **Redrawn player UI.** A header line with a teal `MTC` tag, the file name and
  `width × height · frames · duration`; rounded pill chips for play / speed /
  loop / close-when-finished with a brighter `#12A9B4` accent and hover fill; the
  primary play button filled rather than outlined; a rounded seek track with a
  ring handle; and `m:ss.s` readouts on the Monospace face. The canvas falls
  back to near-black so frames read as pictures, not panels.
* `tools/build_exe.py --onedir` builds a folder bundle. Measured on the dev
  machine, start-to-close on the same sample: onefile 6.1–9.2 s, onedir
  4.1–6.1 s (both include a 1.2 s auto-close timer plus the decode step).

## [1.2.0] — 2026-11-15

### Added

* **Image alpha plane.** `MTB1` gains `FLAG_ALPHA` and an independent lossless
  alpha stream written before the main payload, so RGBA images round-trip
  byte-exactly while RGB stays lossy. Pixels with `alpha < 8` are forced to
  RGB 0 to remove colour fringing.
* **MTC v2.** New 20-byte header (`<4sBBBBHHII`) carries `fps × 100` plus an
  embedded 44.1 kHz stereo PCM WAV segment. v1 files still decode.
* **`convert` and `export`.** One command in (PNG/JPG/WebP/GIF/APNG/MP4/MOV/MKV →
  `.mtb`/`.mtc`), one command out (`.mtb` → PNG/JPG/WebP, `.mtc` → MP4 or PNG
  sequence + WAV).
* **Size presets.** `360p / 480p / 540p / 720p / 1080p / 1440p / 2k / 4k / 4x4k / 8k`
  or any explicit `WxH`; images are fitted without stretching and portrait sizes
  swap the box automatically.
* **`tools/batch_convert.py`.** Folder-level batch conversion in both directions,
  one line per file, per-file failure isolation, `--overwrite`, `--quiet`.
* **Icons.** `tools/make_icons.py` draws the red `#C62828` M (image) and blue
  `#1565C0` M (video) PNG/ICO set from scratch.
* **Viewer without overlays.** All on-screen technical text removed — no frame
  counter, fps, gop or byte size. A still image looks like a still image; the
  window title is the file name.
* **Docs.** `README.md` (English), `README.zh-CN.md`, `docs/FORMAT.md`,
  `docs/CONVERT.md`, `docs/BUILD.md`, `docs/PUBLISH.md` — each bilingual where it
  describes behaviour.
* **Release engineering.** `pyproject.toml`, `requirements.txt`, `.gitignore`,
  `LICENSE` (MIT), `.github/workflows/release.yml`, `tools/make_zip.py`.

### Changed

* `threshold` in the alpha path: an image is now reported as `alpha=True` by
  `info` instead of silently flattening.
* `--fps` now also drives playback rate, not just frame sampling.
* Rebuilt `MTBViewer.exe` (29.4 MB) and verified on the alpha, audio and 8K samples
  (exit code 0 for all three).

### Fixed

* Odd `width × height` no longer produce a broken MP4 export
  (`scale=trunc(iw/2)*2:trunc(ih/2)*2` in the ffmpeg call).
* `decode_video` on a v1 stream no longer divides by an absent fps field.

### Measured (not a guarantee)

* 7680×4320 at q80: 1.58 MB, PSNR 38.83 dB, encode 16.6 s, decode 18.0 s.
* 5.56 MB MP4 → 371.6 KB `.mtc` (172.3 KB of it audio), export back to MP4 with sound.
* 1080×1080 RGBA → 128.7 KB `.mtb` → PNG, alpha bit-exact.

---

## [1.1.0] — 2026-11-11

### Added

* `MTC1` v1 container: I/P frames, `gop` with automatic scene-cut detection
  (`SCENE_CUT_SAD = 24.0`), full decode.
* CLI subcommands `encode`, `decode`, `open`, `video-encode`, `video-decode`,
  `video-info`.
* tkinter viewer with space-to-pause and Esc-to-close.

### Changed

* DCT coefficient coding switched to per-plane, per-position context models,
  about 8 % smaller files at the same quality.

---

## [1.0.0] — 2026-11-09

### Added

* `MTB1` container: gray / YCoCg 4:4:4 / 4:2:0 lossy modes (0–2), gray and RGB
  lossless modes (3–4).
* 8×8 DCT, zig-zag scan, DC prediction, arithmetic coder with adaptive models.
* `encode` / `decode` / `info` for images.
* First benchmark run: 2048×2048 photo at 350 KB, 32.6 dB.

---

# 中文

## [1.5.0] — 2026-11-22

### 新增

* **`installer\MtcSetup.exe`** —— 2.8 MB 的 Windows 安装程序：命令行三个程序、查看器、
  `.mtc` 文件关联、用户 PATH、开始菜单快捷方式、"应用和功能"里的卸载项，全在一次点击里。
  按用户安装（`%LOCALAPPDATA%\Programs\MTC` + `HKCU`），不需要管理员权限。
* 静默安装与静默卸载：`/S`、`/dir <path>`、`/nopath`、`/noassoc`、`/nostartmenu`、
  `/extract <dir>`、`/uninstall`。卸载不带参数也行，安装位置从注册表读回。

### 网站

* 明暗两套主题，默认跟随系统，点顶栏 ☾/☀ 切换后记住选择（`localStorage`）。
* 下载按钮按 `navigator.userAgent` 认系统：Windows 直接给 `MtcSetup.exe`，
  Android 直接给 APK，其它系统去 GitHub 发布页——不会出现"点了没反应"的死链。
* 新增 GitHub Release `v1.5.0`，附 `MtcSetup.exe` 与 `mtc-camera.apk`。

### 实现

* 用 .NET Framework 自带的 `csc.exe` 编译，不引入 Inno Setup / WiX / NSIS；二进制以
  **逐文件 deflate 资源** 嵌入（`DeflateStream` 在 `System.dll` 里，而 `ZipArchive`
  需要的程序集 `csc` 按简单名不一定解析得到）。
* 卸载用"退出后由批处理删除目录"的方式自删除——Windows 不允许删除正在运行的镜像，
  而这个卸载器本身就是被删文件之一。

### 修掉的两个坑

* `uninstall.exe` 原先只认 `/uninstall` 参数，于是"应用和功能"里点卸载（最多带个 `/S`）
  会变成**重新安装**；而且文件名判断在参数解析**之后**，解析时仍按默认目录处理，
  结果注册表清理了、真正的安装目录却留着。现在文件名就是意图，且在解析之前判定。
* 卸载器只有在目标目录里找到 `mtview.exe` 时才删文件，避免路径写错时带走别的目录。
## [1.4.0] — 2026-11-17

### 新增

* **Rust 版无损模式。** `MODE_GRAY_LOSSLESS` 与 `MODE_RGB_LOSSLESS` 到位：中值
  边缘检测预测、残差走同一个算术编码器、彩色走 YCoCg。解码逐像素精确，编码
  结果与参考实现**逐字节相同**（同样是 512×512 实拍，两边都是 154246 字节），
  所以和旧编码器产出的文件可以互换。
* **alpha 也走无损通路。** 以前版本 2 的 alpha 是一个 PNG 数据块，版本 1 的
  alpha 是 Python 独有的残差流；现在两者都走同一套残差编码器，于是 Rust 能读
  所有带 alpha 的文件（包括它诞生之前写的），写出来的也更小。读取时仍然接受
  PNG 数据块。
* **`deploy/register.ps1`。** 把 `.mtc` 关联到 `mtview`、图标指向
  `deploy/mtc.ico`、清掉旧的 `.mtb` / `MTB.*` 注册、把 `bin\` 放到用户 `Path`
  最前面并广播环境变更。按用户生效，不需要管理员，`-Unregister` 可完全撤销。
* **Rust CI。** `.github/workflows/rust.yml` 在 Windows 与 Linux 上跑测试，并
  构建查看器。

### 删除

* **整套 Python 实现。** `mtcodec/`（含 `mtfastdec.dll`）、`pyproject.toml`、
  `requirements.txt`、`MTBViewer.spec`、`tools/`、`bench/`、`tests/`、`native/`、
  `deploy/` 里的 Python 启动器、三份 Python 专用文档（`docs/BUILD.md`、
  `docs/CONVERT.md`、`docs/PUBLISH.md`）、两份实验报告、release 压缩包，以及
  `mtc.exe` 打包产物。包也已卸载，`Path` 上的 `mtcodec` 和 `python -m mtcodec`
  都不存在了。`rust/README.md` 里列出的迁移到此结束。
* 其中三个文件以夹具形式留在 `rust/tests/data/`：一个版本 1 有损流、一个无损
  流、一个带 alpha 的流，各自带着删除前从 Python 解码器算出的像素哈希。

### 修复

* **查看器里中文文件名显示成方块。** egui 自带字体没有中日韩字形，
  `图片信息_lossless.mtc` 在标题栏里是一排空框。现在会借用系统字体
  （雅黑、等线、黑体、宋体，谁在就用谁）作为回退字形。
* **双击 `.mtc` 打开的还是旧查看器。** `.mtc` 的类指向已经不存在的
  `pythonw.exe deploy\mtb_launcher.py`，现在指向 `bin\mtview.exe`。是用 shell
  关联真实启动验证过的，不只是读注册表。

### 实测（不是承诺）

* 无损，512×512 实拍：367361 字节（11.2 bpp），解码逐位精确。
* 2048×1152 实拍的无损文件（`Downloads` 里那个版本 1 文件，621313 字节）：
  Rust 解码结果与 Python 解码器逐位一致。
* 测试：32 个单元、5 个兼容性、9 个查看器几何。

## [1.3.0] — 2026-11-17

### 新增

* **Rust 编解码器接手格式。** `rust/` 从一个验证性的小程序长成一个库
  （`container`、`codec`、`entropy`、`aq`、`arith`、`transform`、`imgio`），
  外加三个可执行文件：`mtcodec`、`mtbench`、`mtview`。它写新的**版本 2**
  码流：交流系数零区、直流系数用左/上/左上做中值边缘检测预测、`last` 符号的
  上下文取前一块的稀疏程度。同 PSNR 下，四张 512×512 实拍、每张六个质量档，
  比版本 1 码流小 **11.35%**（`rust/results/rd_final.csv`）。
* **自适应量化。** 每个 16×16 瓦片一个量化倍率，七档从 0.70× 到 1.40×，
  编码器按瓦片活跃度与全图中位数的比值选档，对前一瓦片做差分编码，倍率向低频
  倾斜。帧内编码没有参考帧可供解码端推导，所以档位是**写进码流**的——平均每个
  瓦片远不到一个字节。默认不开，`--aq on` 打开；`--aq auto` 两种都编，
  选出"质量不降且更小"的那个。
* **`mtview`，egui 查看器。** 纯白窗口，图片适应窗口，滚轮以光标为中心在
  "适应窗口"到 8× 之间缩放，拖动平移，双击或按 `1` 回到适应，Esc 关闭。
  `.mtc` 文件的头部栏显示模式、质量和体积；PNG/JPEG/WebP/BMP 也能直接开，
  所以拿它对比"原图与解码结果"很顺手。放大到 1:1 以上会切到最近邻采样，
  块边界和振铃按原样显示，不被插值抹平。
* **`mtbench` 与 `bench/rd_compare.py`。** 前者按"图片 × 质量 × 码流形态"
  测字节、bpp、PSNR 和编解码耗时；后者在**同 PSNR** 下比较两次运行——
  两套编码参数不能用同一个 `-q` 比，谁的字节多谁画质高，看上去占便宜。
* **`tests/data/legacy_v1.mtc`。** 一个 Python 编码器写出来的文件，测试里解码
  后与 Python 解码器算出的像素哈希比对。

### 修复

* **Rust 解所有已有文件时都是转置的。** 取块时写成了 `blk[x][y]`，DCT 也跟着
  这么算，于是每个 8×8 块相对规范都转了 90°——而 Rust 自己的编码器和解码器
  互相自洽，所以往返测试一直是绿的。现在版本 1 的文件逐位一致，夹具测试会
  在轴再次写反时报错。

### 变更

* Rust 编码器默认写版本 2；`--v1` 仍能产出旧码流，并且仍然是对参考实现的忠实
  复刻（普通四舍五入、只用左边块预测直流、扁平模型），迁移才站得住。
* 自适应量化**不是**默认项：固定 `-q` 下大约 +0.3 dB 换 +15% 字节，
  而同 PSNR 下要多花 **8.5%**。所以 `--aq auto` 在四张测试图上都把它拒掉。
  它留在格式里，供偏感知的用途。
* `last` 上下文实测 −0.45%；另外两个上下文（频带显著性、直流类别）实测分别
  是 +1.4% 和 −0.1%，已移除。

### 实测（不是承诺）

* 2048×1152 实拍，`-q 85`，4:2:0，版本 2：397 KB、33.63 dB，编码 115 ms、
  解码 130 ms（单核，release）。同 `-q` 的版本 1：457 KB、34.22 dB。
* 同 PSNR 下版本 2 相对版本 1：photo_a −5.3%、photo_b −15.3%、photo_c −8.5%、
  render_d −15.8%。
* 测试：28 个单元、2 个兼容性、9 个查看器几何。

## [1.2.1] — 2026-11-15

### 新增

* **完整播放控件。** 倍速 0.5× / 1× / 1.5× / 2×、可拖动进度条、音量 0–100、
  循环播放、播完关闭。空格播放/暂停，←/→ 前后跳 1 秒，↑/↓ 调音量，Esc 关闭。
* **真正的音频变速与音量。** 内嵌 PCM 用 numpy 重采样并做增益，写成临时 WAV
  交给系统播放器（Windows 用 `winsound`，Linux 用 `aplay`，macOS 用 `afplay`），
  所以变速和音量不需要 pygame / sounddevice 这类额外依赖。
* **解码进度条。** 打开片子时窗口里就是一根进度条加百分比，还会显示正在解码的
  那一帧；解码完成后同一个窗口直接变成播放器，这根进度条交给播放进度条接着用。
  `decode_video` 新增 `progress(i, nframes, frame)` 回调。

### 变更

* **播放时钟改为单调时钟。** 帧时刻由 `time.monotonic()` 推出并排到下一帧边界，
  不再用固定 `after(1000/fps)` 接力，长片不会越播越慢。
* **内存更省、贴图更顺。** 解码后先把帧缩到不超过 960×720，再按需生成
  `PhotoImage` 并做 32 帧 LRU 缓存，不再一次性预生成全部原尺寸帧。
* `tools/build_exe.py --onedir` 可产出目录版。本机实测同一样本从启动到关闭：
  onefile 6.1–9.2 秒，onedir 4.1–6.1 秒（都含 1.2 秒自动关闭与解码）。

## [1.2.0] — 2026-11-15

### 新增

* **图片透明平面。** `MTB1` 增加 `FLAG_ALPHA` 和一个独立无损 alpha 流，写在
  主数据前面；RGBA 图往返逐位一致，RGB 仍可有损。`alpha < 8` 的像素 RGB 置 0，
  消掉半透明边缘脏色。
* **MTC v2。** 新的 20 字节头（`<4sBBBBHHII`）带 `fps × 100`，并内嵌
  44.1 kHz 立体声 PCM WAV。v1 文件仍能解码。
* **`convert` 与 `export`。** 一条命令进（PNG/JPG/WebP/GIF/APNG/MP4/MOV/MKV →
  `.mtb`/`.mtc`），一条命令出（`.mtb` → PNG/JPG/WebP，`.mtc` → MP4 或
  PNG 序列 + WAV）。
* **尺寸预设。** `360p / 480p / 540p / 720p / 1080p / 1440p / 2k / 4k / 4x4k / 8k`
  或任意 `WxH`；等比装框不拉伸，竖图自动对调宽高。
* **`tools/batch_convert.py`。** 目录级双向批量转换，每文件一行，单文件失败隔离，
  支持 `--overwrite`、`--quiet`。
* **图标。** `tools/make_icons.py` 从零画红色 `#C62828` M（图片）与蓝色
  `#1565C0` M（视频）的 PNG/ICO 一套。
* **查看器去掉叠加信息。** 屏幕上所有技术文字全部移除——没有帧号、帧率、
  gop、字节数。单张图片就是单张图片，窗口标题只有文件名。
* **文档。** `README.md`（英文）、`README.zh-CN.md`、`docs/FORMAT.md`、
  `docs/CONVERT.md`、`docs/BUILD.md`、`docs/PUBLISH.md`，凡描述行为处均双语。
* **发布工程。** `pyproject.toml`、`requirements.txt`、`.gitignore`、`LICENSE`（MIT）、
  `.github/workflows/release.yml`、`tools/make_zip.py`。

### 变更

* `info` 现在正确报告 `has_alpha=True`，不再静默扁平化。
* `--fps` 同时决定播放帧率，不只是抽帧率。
* 重打 `MTBViewer.exe`（29.4 MB），alpha、音频、8K 三个样本全部 exit code 0。

### 修复

* 奇数宽高不再导出损坏的 MP4（ffmpeg 调用里加了
  `scale=trunc(iw/2)*2:trunc(ih/2)*2`）。
* v1 流解码不再除一个不存在的 fps 字段。

### 实测（不是承诺）

* 7680×4320 q80：1.58 MB，PSNR 38.83 dB，编码 16.6 s，解码 18.0 s。
* 5.56 MB MP4 → 371.6 KB `.mtc`（其中音频 172.3 KB），导回 MP4 带声音。
* 1080×1080 RGBA → 128.7 KB `.mtb` → PNG，alpha 逐位相等。

## [1.1.0] — 2026-11-11

### 新增

* `MTC1` v1 容器：I/P 帧、`gop` 与自动场景切换检测（`SCENE_CUT_SAD = 24.0`）、完整解码。
* CLI 子命令 `encode`、`decode`、`open`、`video-encode`、`video-decode`、`video-info`。
* tkinter 查看器，空格暂停、Esc 关闭。

### 变更

* DCT 系数编码改为"按平面 × 按位置"的上下文模型，同质量下体积约小 8%。

## [1.0.0] — 2026-11-09

### 新增

* `MTB1` 容器：灰度 / YCoCg 4:4:4 / 4:2:0 有损模式（0–2），灰度与 RGB 无损模式（3–4）。
* 8×8 DCT、zig-zag、DC 预测、带自适应模型的算术编码器。
* 图片的 `encode` / `decode` / `info`。
* 首次基准：2048×2048 照片 350 KB，32.6 dB。
