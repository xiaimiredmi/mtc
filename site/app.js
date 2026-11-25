// The page's behaviour: the live codec demo, the split slider, the rate/distortion
// curve, and the small bits of chrome.

import { loadCodec, parseHeader, hexDump, psnr, humanBytes } from "./codec.js";

const $ = (id) => document.getElementById(id);

// Surface failures in the tab title: this page has no console a visitor can open, and
// a silent half-loaded demo is the worst possible outcome.
window.addEventListener("error", (event) => {
  document.title = "MTC · 出错：" + (event.message || "script error");
});
window.addEventListener("unhandledrejection", (event) => {
  const reason = event.reason;
  document.title = "MTC · 出错：" + (reason && reason.message ? reason.message : reason);
});
const params = new URLSearchParams(location.search);
// `?auto` is the headless check: skip the reveal animations and counters so a
// screenshot shows the finished state instead of a frame mid-transition.
if (params.has("auto")) document.body.classList.add("auto");

// ────────────────────────────── theme ─────────────────────────────────
// Light is the default; dark follows the system until the visitor clicks the
// switch, after which their choice is remembered and the system is ignored.
const THEME_KEY = "mtc-theme";

function systemTheme() {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyTheme(theme, remember) {
  document.documentElement.dataset.theme = theme;
  const button = $("theme");
  if (button) {
    button.textContent = theme === "dark" ? "☀" : "☾";
    button.title = theme === "dark" ? "切换到白天" : "切换到黑夜";
    button.setAttribute("aria-label", button.title);
  }
  if (remember) {
    try { localStorage.setItem(THEME_KEY, theme); } catch (e) { /* private mode */ }
  }
}

function initialTheme() {
  const forced = params.get("theme");        // ?theme=light — used for screenshots
  if (forced === "light" || forced === "dark") return forced;
  try {
    const stored = localStorage.getItem(THEME_KEY);
    if (stored === "light" || stored === "dark") return stored;
  } catch (e) { /* private mode */ }
  return systemTheme();
}

applyTheme(initialTheme(), false);

if ($("theme")) {
  $("theme").addEventListener("click", () => {
    applyTheme(currentThemeName() === "dark" ? "light" : "dark", true);
  });
}

function currentThemeName() {
  return document.documentElement.dataset.theme || systemTheme();
}

// Only follow the system while the visitor has not decided for themselves.
window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
  let chosen = null;
  try { chosen = localStorage.getItem(THEME_KEY); } catch (e) { }
  if (chosen !== "light" && chosen !== "dark") applyTheme(systemTheme(), false);
});

// ───────────────────────── download target by OS ───────────────────────
// One button, pointed at whatever this machine can actually run. Anything the
// project does not build for goes to the releases page rather than a dead asset.
const RELEASES = "https://github.com/xiaimiredmi/mtc/releases";
const LATEST = RELEASES + "/latest";
const ASSET = LATEST + "/download/";

function detectOS() {
  const ua = navigator.userAgent;
  const platform = (navigator.userAgentData && navigator.userAgentData.platform)
    || navigator.platform || "";
  if (/Windows NT/i.test(ua) || /Win(32|64)/i.test(platform)) return "windows";
  if (/Android/i.test(ua)) return "android";
  if (/iPhone|iPad|iPod/i.test(ua)) return "ios";
  if (/Mac/i.test(platform) || /Mac OS X/i.test(ua)) return "mac";
  if (/Linux|X11/i.test(platform) || /Linux/i.test(ua)) return "linux";
  return "other";
}

const DOWNLOADS = {
  windows: {
    href: ASSET + "MtcSetup.exe",
    label: "下载 Windows 安装程序",
    short: "下载安装程序",
    note: "检测到 <b>Windows</b>：直接下载 <b>MtcSetup.exe</b>（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。",
  },
  android: {
    href: ASSET + "mtc-camera.apk",
    label: "下载 Android 相机",
    short: "下载 APK",
    note: "检测到 <b>Android</b>：直接下载 <b>mtc-camera.apk</b>（0.8 MB），相机直出 .mtc，另有相册与查看器。",
  },
  mac: {
    href: LATEST,
    label: "在 GitHub 上选择版本",
    short: "去 GitHub",
    note: "检测到 <b>macOS</b>：这个项目目前只发布 Windows 和 Android 版本，所以带你去 GitHub 发布页。",
  },
  ios: {
    href: LATEST,
    label: "在 GitHub 上选择版本",
    short: "去 GitHub",
    note: "检测到 <b>iPhone / iPad</b>：没有 iOS 版本，已在 GitHub 发布页。",
  },
  linux: {
    href: LATEST,
    label: "在 GitHub 上选择版本",
    short: "去 GitHub",
    note: "检测到 <b>Linux</b>：没有打包好的 Linux 版本，但可以从源码构建（见发布页）。",
  },
  other: {
    href: LATEST,
    label: "在 GitHub 上选择版本",
    short: "去 GitHub",
    note: "没认出这个系统，带你去 GitHub 发布页自己挑。",
  },
};

function applyDownload() {
  const kind = params.get("os") || detectOS();   // ?os=linux — used for screenshots
  const target = DOWNLOADS[kind] || DOWNLOADS.other;
  for (const id of ["dlMain", "heroDl"]) {
    const link = $(id);
    if (!link) continue;
    link.href = target.href;
    link.textContent = id === "heroDl" ? target.short : target.label;
    if (kind === "windows" || kind === "android") {
      link.setAttribute("download", "");
    }
  }
  const alt = $("dlAlt");
  if (alt) alt.href = LATEST;
  const note = $("osNote");
  if (note) note.innerHTML = target.note;
}

applyDownload();

// `?scroll=download` is for the headless check: it brings a section to the top so
// a 900 px screenshot contains the thing being checked.
if (params.has("auto") && params.get("scroll")) {
  // Smooth scrolling plus a screenshot is a race: the frame gets captured mid
  // repaint and shows a half-drawn page. Jump, after the first paint.
  document.documentElement.style.scrollBehavior = "auto";
  const target = document.getElementById(params.get("scroll"));
  if (target) {
    setTimeout(() => target.scrollIntoView({ block: "start", behavior: "instant" }), 500);
  }
}
const MAX_DEMO_SIDE = 1024; // smaller than before: the demo is about behaviour, and 1024 is 36% less work

const state = {
  codec: null,
  source: null,   // { rgba, width, height, label, originalBytes }
  bytes: null,    // last .mtc
  decoded: null,  // { rgba, width, height }
  sub: "420",
  q: 85,
  aq: false,
  busy: false,
};

// ─────────────────────────────── boot ──────────────────────────────────
// Prefetch while the visitor is still reading the hero: by the time they press
// "用样图试试" there is nothing left to wait for. Both requests start together.
const samplePrefetch = fetch("assets/sample.png").then((r) => (r.ok ? r.blob() : null)).catch(() => null);

const ready = (async () => {
  try {
    state.codec = await loadCodec();
    $("footnote").textContent =
      `页面上的编码器是 WebAssembly（容器 v${state.codec.version}），和命令行走同一份 Rust 代码。`;
  } catch (err) {
    $("stageNote").textContent = "编解码器加载失败：" + err.message;
  }
})();

// ─────────────────────────── scroll reveals ────────────────────────────
const io = new IntersectionObserver((entries) => {
  for (const entry of entries) {
    if (!entry.isIntersecting) continue;
    entry.target.classList.add("in");
    if (entry.target.id === "bars") entry.target.classList.add("in");
    io.unobserve(entry.target);
  }
}, { threshold: 0.15 });
document.querySelectorAll(".reveal, #bars").forEach((el) => io.observe(el));

// counters in the hero
const counters = document.querySelectorAll("#strip b");
if (document.body.classList.contains("auto")) {
  // No animation in auto mode: a screenshot should show final numbers, and
  // requestAnimationFrame does not advance under a virtual clock.
  counters.forEach((el) => {
    const target = parseFloat(el.dataset.count);
    el.textContent = (target % 1 === 0 ? Math.round(target) : target.toFixed(1))
      + (el.dataset.suffix || "");
  });
}
const counterIO = new IntersectionObserver((entries) => {
  for (const entry of entries) {
    if (!entry.isIntersecting) continue;
    const el = entry.target;
    const target = parseFloat(el.dataset.count);
    const suffix = el.dataset.suffix || "";
    if (document.body.classList.contains("auto")) {
      el.textContent = (target % 1 === 0 ? Math.round(target) : target.toFixed(1)) + suffix;
      counterIO.unobserve(el);
      continue;
    }
    const start = performance.now();
    const tick = (now) => {
      const t = Math.min(1, (now - start) / 900);
      const eased = 1 - Math.pow(1 - t, 3);
      const value = target * eased;
      el.textContent = (target % 1 === 0 ? Math.round(value) : value.toFixed(1)) + suffix;
      if (t < 1) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
    counterIO.unobserve(el);
  }
}, { threshold: 0.6 });
counters.forEach((el) => counterIO.observe(el));

// ───────────────────────────── split slider ────────────────────────────
const split = $("split");
const stage = $("stage");
const cvA = $("cvA");
const cvB = $("cvB");

function setSplit(fraction) {
  const pct = Math.max(0, Math.min(100, fraction * 100));
  split.style.left = pct + "%";
  cvB.style.clipPath = `inset(0 0 0 ${pct}%)`;
}
setSplit(0.5);

let dragging = false;
const moveTo = (event) => {
  const rect = stage.getBoundingClientRect();
  setSplit((event.clientX - rect.left) / rect.width);
};
split.addEventListener("pointerdown", (event) => {
  dragging = true;
  split.setPointerCapture(event.pointerId);
});
split.addEventListener("pointermove", (event) => { if (dragging) moveTo(event); });
split.addEventListener("pointerup", () => { dragging = false; });
stage.addEventListener("pointerdown", (event) => {
  if (event.target === split || split.contains(event.target)) return;
  if (state.source) moveTo(event);
});

// ───────────────────────────── file loading ────────────────────────────
function isMtc(bytes) {
  return bytes.length > 12 && bytes[0] === 0x4d && bytes[1] === 0x54
    && bytes[2] === 0x42 && bytes[3] === 0x31;
}

async function loadMtc(bytes, label) {
  const header = parseHeader(bytes);
  const decoded = state.codec.decode(bytes, 0);
  // Show the file's own content; the "original" side has nothing to compare to,
  // so both halves show the decode and the note says so.
  state.source = null;
  state.bytes = bytes;
  state.decoded = decoded;
  drawImage(cvA, decoded);
  drawImage(cvB, decoded);
  $("drop").classList.add("gone");
  $("s0").textContent = humanBytes(bytes.length);
  $("s1").textContent = "—";
  $("s2").textContent = "—";
  $("s3").textContent = "—";
  $("s4").textContent = "—";
  $("s5").textContent = decoded.ms.toFixed(0) + " ms";
  $("save").disabled = true;
  $("asPng").disabled = false;
  $("curve").getContext("2d").clearRect(0, 0, 320, 120);
  $("stageNote").textContent = `${label}：${header ? header.modeName + " · q" + header.quality + " · " + header.width + "×" + header.height : "已解码"}`;
  $("hex").textContent = describe(header, bytes);
  return header;
}

async function loadImage(file, label) {
  const bitmap = await createImageBitmap(file);
  const scale = Math.min(1, MAX_DEMO_SIDE / Math.max(bitmap.width, bitmap.height));
  const width = Math.max(1, Math.round(bitmap.width * scale));
  const height = Math.max(1, Math.round(bitmap.height * scale));

  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  ctx.drawImage(bitmap, 0, 0, width, height);
  const data = ctx.getImageData(0, 0, width, height).data;

  // Drop a fully opaque alpha channel: storing one would only cost bytes.
  let opaque = true;
  for (let i = 3; i < data.length; i += 4) {
    if (data[i] !== 255) { opaque = false; break; }
  }

  state.source = {
    rgba: data,
    width,
    height,
    channels: opaque ? 3 : 4,
    label,
    scaled: scale < 1,
    pixelBytes: opaque ? data.length * 3 / 4 : data.length,
  };
  $("drop").classList.add("gone");
  $("stageNote").textContent = scale < 1
    ? `${label}：为了在浏览器里跑得舒服，缩到 ${width}×${height}`
    : `${label}：${width}×${height}`;
  await run();
}

/** The pixels in the layout the codec asked for: RGB when opaque, RGBA otherwise. */
function packedOf(source) {
  if (source.channels === 4) return source.rgba;
  const out = new Uint8Array(source.width * source.height * 3);
  for (let i = 0, o = 0; i < source.rgba.length; i += 4, o += 3) {
    out[o] = source.rgba[i];
    out[o + 1] = source.rgba[i + 1];
    out[o + 2] = source.rgba[i + 2];
  }
  return out;
}

// `source.rgba` is always the canvas's RGBA; `channels` only says what gets
// handed to the encoder, so drawing and PSNR never need a conversion.
function rgbaOf(source) {
  return source.rgba;
}

/** A picture drawn in a canvas: no network, so the demo can be checked offline. */
function syntheticImage() {
  const width = 320;
  const height = 240;
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  const gradient = ctx.createLinearGradient(0, 0, width, height);
  gradient.addColorStop(0, "#1f6feb");
  gradient.addColorStop(0.5, "#8b5cf6");
  gradient.addColorStop(1, "#f97316");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, width, height);
  ctx.fillStyle = "rgba(255,255,255,.85)";
  for (let i = 0; i < 24; i++) {
    ctx.beginPath();
    ctx.arc((i * 53) % width, (i * 97) % height, 6 + (i % 5) * 3, 0, Math.PI * 2);
    ctx.fill();
  }
  return { data: ctx.getImageData(0, 0, width, height).data, width, height };
}

// ───────────────────────────── the pipeline ────────────────────────────
async function run() {
  if (!state.codec || !state.source || state.busy) return;
  state.busy = true;
  const source = state.source;
  try {
    const lossless = state.sub === "lossless";
    const encoded = state.codec.encode({
      data: packedOf(source),
      width: source.width,
      height: source.height,
      channels: source.channels,
      quality: state.q,
      lossless,
      aq: state.aq && !lossless,
    });
    const decoded = state.codec.decode(encoded.bytes, 0);
    const reference = rgbaOf(source);
    const quality = psnr(reference, decoded.rgba, source.width, source.height);

    state.bytes = encoded.bytes;
    state.decoded = decoded;

    drawImage(cvA, { rgba: reference, width: source.width, height: source.height });
    drawImage(cvB, decoded);

    $("s0").textContent = humanBytes(Math.round(source.pixelBytes));
    $("s1").textContent = humanBytes(encoded.bytes.length);
    $("s2").textContent = (encoded.bytes.length * 8 / (source.width * source.height)).toFixed(3);
    $("s3").textContent = quality >= 99 ? "无损" : quality.toFixed(2) + " dB";
    $("s4").textContent = encoded.ms.toFixed(1) + " ms";
    $("s5").textContent = decoded.ms.toFixed(1) + " ms";
    $("save").disabled = false;
    $("asPng").disabled = false;

    const header = parseHeader(encoded.bytes);
    $("hex").textContent = describe(header, encoded.bytes);
    // The curve costs four more encode+decode rounds. Paint the result first, then
    // spend that time in the background so the page never feels stuck.
    scheduleCurve();
  } catch (err) {
    $("stageNote").textContent = "出错了：" + err.message;
    document.title = "MTC · 出错：" + err.message;
    console.error("run", err);
  } finally {
    state.busy = false;
    window.__mtcLast = {
      bytes: state.bytes ? state.bytes.length : 0,
      psnr: $("s3").textContent,
      bpp: $("s2").textContent,
    };
  }
}

function describe(header, bytes) {
  const hex = hexDump(bytes, 12);
  if (!header) return "头 12 字节：\n" + hex;
  return [
    "头 12 字节：",
    hex,
    "",
    `magic ${header.magic}   version ${header.version}   mode ${header.mode} (${header.modeName})`,
    `quality ${header.quality}   flags ${header.flags}` +
      (header.hasAlpha ? " [alpha]" : "") + (header.hasAq ? " [aq]" : ""),
    `size ${header.width}×${header.height}   file ${humanBytes(bytes.length)}`,
  ].join("\n");
}

function drawImage(canvas, image) {
  canvas.width = image.width;
  canvas.height = image.height;
  const ctx = canvas.getContext("2d");
  const data = image.rgba instanceof Uint8ClampedArray
    ? image.rgba
    : new Uint8ClampedArray(image.rgba);
  ctx.putImageData(new ImageData(new Uint8ClampedArray(data), image.width, image.height), 0, 0);
}

// ───────────────────────── rate / distortion curve ─────────────────────
const CURVE_QUALITIES = [35, 60, 80, 95];

let curveTimer = null;

/** Draw the rate/distortion curve once the page is idle. */
function scheduleCurve() {
  clearTimeout(curveTimer);
  curveTimer = setTimeout(() => {
    const idle = window.requestIdleCallback || ((fn) => setTimeout(fn, 120));
    idle(() => { drawCurve(); });
  }, 260);
}

async function drawCurve() {
  const canvas = $("curve");
  const ctx = canvas.getContext("2d");
  const dpr = window.devicePixelRatio || 1;
  canvas.width = canvas.clientWidth * dpr;
  canvas.height = 120 * dpr;
  ctx.scale(dpr, dpr);
  const w = canvas.width / dpr;
  const h = 120;
  ctx.clearRect(0, 0, w, h);
  if (!state.source) return;

  const source = state.source;
  const lossless = state.sub === "lossless";
  const reference = rgbaOf(source);
  const points = [];
  for (const quality of CURVE_QUALITIES) {
    try {
      const encoded = state.codec.encode({
        data: packedOf(source),
        width: source.width,
        height: source.height,
        channels: source.channels,
        quality,
        lossless,
        aq: state.aq && !lossless,
      });
      const decoded = state.codec.decode(encoded.bytes, 0);
      points.push({
        quality,
        bpp: encoded.bytes.length * 8 / (source.width * source.height),
        psnr: psnr(reference, decoded.rgba, source.width, source.height),
      });
    } catch { /* skip a bad point */ }
  }
  if (points.length < 2) return;

  const xs = points.map((p) => p.psnr);
  const ys = points.map((p) => p.bpp);
  const minX = Math.min(...xs) - 0.6;
  const maxX = Math.max(...xs) + 0.6;
  const maxY = Math.max(...ys) * 1.12;

  const px = (value) => 26 + ((value - minX) / (maxX - minX)) * (w - 40);
  const py = (value) => h - 18 - (value / maxY) * (h - 30);

  ctx.strokeStyle = "rgba(255,255,255,.06)";
  ctx.lineWidth = 1;
  for (let i = 0; i <= 3; i++) {
    const y = 12 + i * ((h - 30) / 3);
    ctx.beginPath();
    ctx.moveTo(26, y);
    ctx.lineTo(w - 14, y);
    ctx.stroke();
  }

  ctx.beginPath();
  points.forEach((point, index) => {
    const x = px(point.psnr);
    const y = py(point.bpp);
    if (index === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
  });
  const gradient = ctx.createLinearGradient(26, 0, w - 14, 0);
  gradient.addColorStop(0, "#48e5f0");
  gradient.addColorStop(1, "#8b7cff");
  ctx.strokeStyle = gradient;
  ctx.lineWidth = 2;
  ctx.stroke();

  for (const point of points) {
    const current = point.quality === state.q;
    ctx.beginPath();
    ctx.arc(px(point.psnr), py(point.bpp), current ? 4.5 : 2.6, 0, Math.PI * 2);
    ctx.fillStyle = current ? "#ff5fa2" : "rgba(255,255,255,.55)";
    ctx.fill();
  }

  ctx.fillStyle = "rgba(255,255,255,.35)";
  ctx.font = "10px ui-monospace, monospace";
  ctx.fillText(maxY.toFixed(2) + " B/px", 2, 14);
  ctx.fillText(minX.toFixed(0) + " dB", 26, h - 4);
  ctx.fillText(maxX.toFixed(0) + " dB", w - 46, h - 4);
}

// ───────────────────────────── controls ────────────────────────────────
let typing = null;
$("q").addEventListener("input", (event) => {
  state.q = Number(event.target.value);
  $("qv").textContent = state.q;
  clearTimeout(typing);
  typing = setTimeout(run, 160);
});

for (const button of document.querySelectorAll(".seg button")) {
  button.addEventListener("click", () => {
    document.querySelectorAll(".seg button").forEach((b) => b.classList.remove("on"));
    button.classList.add("on");
    state.sub = button.dataset.sub;
    $("q").disabled = state.sub === "lossless";
    run();
  });
}

$("aq").addEventListener("change", (event) => {
  state.aq = event.target.checked;
  run();
});

$("file").addEventListener("change", async (event) => {
  const file = event.target.files && event.target.files[0];
  if (!file) return;
  const bytes = new Uint8Array(await file.arrayBuffer());
  if (isMtc(bytes)) await loadMtc(bytes, file.name);
  else await loadImage(file, file.name);
});

$("sample").addEventListener("click", async () => {
  $("stageNote").textContent = "载入样图…";
  try {
    // Never leave the button spinning: the prefetch gets three seconds, then a picture
    // drawn locally stands in. A visitor on a bad connection still sees the codec work.
    const blob = await Promise.race([
      samplePrefetch,
      new Promise((resolve) => setTimeout(() => resolve(null), 3000)),
    ]);
    if (blob) {
      await loadImage(new File([blob], "sample.png", { type: "image/png" }), "样图");
    } else {
      const image = syntheticImage();
      state.source = {
        rgba: image.data,
        width: image.width,
        height: image.height,
        channels: 3,
        label: "自检图",
        scaled: false,
        pixelBytes: image.data.length * 3 / 4,
      };
      $("drop").classList.add("gone");
      $("stageNote").textContent = "样图没能在 3 秒内加载，用一张本地生成的图代替（编码器是真的）";
      await run();
    }
  } catch (err) {
    $("stageNote").textContent = "样图加载失败：" + err.message;
    document.title = "MTC · 样图加载失败：" + err.message;
    console.error("sample", err);
  }
});

const drop = $("drop");
for (const type of ["dragenter", "dragover"]) {
  stage.addEventListener(type, (event) => {
    event.preventDefault();
    drop.classList.add("hot");
  });
}
for (const type of ["dragleave", "drop"]) {
  stage.addEventListener(type, () => drop.classList.remove("hot"));
}
stage.addEventListener("drop", async (event) => {
  event.preventDefault();
  const file = event.dataTransfer.files && event.dataTransfer.files[0];
  if (!file) return;
  const bytes = new Uint8Array(await file.arrayBuffer());
  if (isMtc(bytes)) await loadMtc(bytes, file.name);
  else await loadImage(file, file.name);
});

$("save").addEventListener("click", () => {
  if (!state.bytes) return;
  const blob = new Blob([state.bytes], { type: "application/octet-stream" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = "demo.mtc";
  link.click();
  setTimeout(() => URL.revokeObjectURL(link.href), 4000);
});

$("asPng").addEventListener("click", () => {
  if (!state.decoded) return;
  cvB.toBlob((blob) => {
    if (!blob) return;
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = "demo-decoded.png";
    link.click();
    setTimeout(() => URL.revokeObjectURL(link.href), 4000);
  }, "image/png");
});

window.addEventListener("resize", () => { if (state.source) drawCurve(); });

// `?auto` runs the sample on load, which is how the page is checked in a headless
// browser without a human clicking anything.
(async () => {
  await ready;
  if (!state.codec) return;
  if (params.has("auto")) {
    const what = params.get("auto");
    if (what === "synth") {
      // Deterministic check with no network at all.
      const image = syntheticImage();
      const opaque = true;
      state.source = {
        rgba: image.data,
        width: image.width,
        height: image.height,
        channels: opaque ? 3 : 4,
        label: "自检",
        scaled: false,
        pixelBytes: image.data.length * 3 / 4,
      };
      $("drop").classList.add("gone");
      $("stageNote").textContent = "自检图：" + image.width + "×" + image.height;
      await run();
    } else if (what === "mtc") {
      const response = await fetch("assets/legacy.mtc");
      await loadMtc(new Uint8Array(await response.arrayBuffer()), "legacy.mtc");
    } else {
      await $("sample").click();
    }
    document.body.dataset.mtcAuto = "done";
  }
})();
