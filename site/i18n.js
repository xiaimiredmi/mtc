/*
 * Translations for the page.
 *
 * The keys are the Chinese text as it appears in the HTML. That is unusual, and it is on
 * purpose: it means index.html needs no marking up - no data-i18n attributes to keep in
 * sync, no chance of a key drifting away from the paragraph it belongs to. The first pass
 * remembers the original text of every node, so switching languages repeatedly always
 * translates from the same source.
 *
 * English is complete. Japanese, Korean, Russian, Spanish and French cover the parts a
 * visitor reads first - navigation, headings, buttons, the download panel, the footer -
 * and fall back to English for the long explanatory paragraphs rather than leaving a
 * half-Chinese page behind.
 */
(function () {
  "use strict";

  var FALLBACK = "en";

  var EN = {
    "MTC · 自己写的图片编解码器": "MTC · an image codec written from scratch",
    "试一下": "Try it",
    "原理": "How it works",
    "数据": "Numbers",
    "安卓": "Android",
    "下载": "Download",
    "Rust · 自研格式 · 无第三方编解码库": "Rust · own format · no third-party codec",
    "把照片压小": "Smaller photos,",
    "用的还是": "with a codec",
    "编解码器": "",
    "8×8 DCT、零区量化、上下文算术编码、无损与精确 alpha。": "8×8 DCT, dead-zone quantisation, context arithmetic coding, lossless and exact alpha.",
    "这个页面里的每一个字节都是你的浏览器现场算出来的":
      "Every byte on this page is computed by your browser, right now",
    "立刻压一张图": "Compress an image",
    "看源代码": "Read the source",
    "同画质体积更小": "smaller at the same quality",
    "2048×1152 · q85 的体积": "size at 2048×1152 · q85",
    "第三方编解码依赖": "third-party codecs",
    "在浏览器里压一张": "Compress one in the browser",
    "图片不出本机。编码、解码、PSNR 全在这里跑完。":
      "Your image never leaves this machine. Encoding, decoding and PSNR all run here.",
    "原图": "Original",
    "MTC 解码": "MTC decoded",
    "把图片拖进来": "Drop an image here",
    "PNG / JPG / WebP，或者一个": "PNG / JPG / WebP, or an",
    "选文件": "Choose a file",
    "用样图试试": "Try a sample",
    "还没算过——数据会出现在右边。": "Nothing computed yet - the numbers appear on the right.",
    "质量": "Quality",
    "无损": "Lossless",
    "自适应量化": "Adaptive quantisation",
    "(实测更费码率)": "(costs bytes in practice)",
    "原始像素": "Raw pixels",
    "每像素": "Per pixel",
    "编码": "Encode",
    "解码": "Decode",
    "一条真实的率失真曲线：横轴 PSNR，纵轴每像素字节，越靠左下越好。":
      "A real rate/distortion curve: PSNR across, bytes per pixel down. Lower left is better.",
    "下载 .mtc": "Download .mtc",
    "导出 PNG": "Export PNG",
    "等待第一张图片…": "Waiting for the first image…",
    "它到底怎么压的": "What it actually does",
    "一条主线，五步，每一步都可以单独解释给你听。":
      "One pipeline, five steps, and each one can be explained on its own.",
    "YCoCg 换色": "YCoCg colour",
    "可逆整数变换，把亮度与两个色差分开放。色度再用":
      "A reversible integer transform that separates luma from two chroma channels. Chroma then gets",
    "的步长多压一点。": "coarser steps.",
    "正交基，直流系数正好等于块均值 ×8，所以量化可以按像素级误差直接定步长。":
      "An orthogonal basis whose DC coefficient is exactly eight times the block mean, so quantisation can be set straight from a pixel-level error budget.",
    "零区量化": "Dead-zone quantisation",
    "交流系数用": "AC coefficients use a",
    "个步长的零区：小的直接归零，省下的字节比丢掉的精度多得多。":
      "-step dead zone: small ones snap to zero, and the bytes saved are worth far more than the precision lost.",
    "上下文算术编码": "Context arithmetic coding",
    "11 位自适应概率、按频带分模型，": "11-bit adaptive probabilities, one model per band,",
    "还用前一块的稀疏程度做上下文。": "and the sparsity of the previous block as context.",
    "容器": "Container",
    "12 字节头加若干数据段。同一份容器放有损、无损、alpha，版本号只决定系数模型的布局。":
      "A 12-byte header plus segments. The same container carries lossy, lossless and alpha; the version only decides the coefficient model layout.",
    "自适应量化，为什么默认关着": "Adaptive quantisation, and why it is off by default",
    "每个 16×16 瓦片一个倍率（0.70×–1.40×），编码器按瓦片活跃度选档、对前一片差分编码、向低频倾斜。":
      "Each 16×16 tile gets a multiplier (0.70×-1.40×). The encoder picks the step from tile activity, delta-codes it against the previous tile, and leans towards low frequencies.",
    "固定": "Fixed",
    "下它确实更好看（约 +0.3 dB），但同 PSNR 下要多花":
      "it genuinely looks better (about +0.3 dB) but costs",
    "字节。所以默认关，勾上就能在这个页面里自己复现。":
      "more bytes at the same PSNR. So it is off by default; tick the box and reproduce it here.",
    "12 字节的头，就这么点": "A 12-byte header. That is all of it",
    "上面那堆十六进制就是你这张图的真实头 12 字节。没有 EXIF，没有元数据块——格式小到可以在一屏里讲完。":
      "The hex above is the real 12-byte header of your picture. No EXIF, no metadata blocks - the format fits on one screen.",
    "测出来的，不是吹出来的": "Measured, not claimed",
    "四张 512×512 实拍，每张六个质量档，全部按":
      "Four real 512×512 photographs, six quality levels each, all compared at the same",
    "同 PSNR": "PSNR",
    "比较（同一个": "compared (the same",
    "比是不公平的）。": "is not a fair comparison).",
    "交流零区": "AC dead zone",
    "DC 二维预测": "2D DC prediction",
    "last 上下文": "last context",
    "频带上下文": "band context",
    "逐瓦片 AQ": "per-tile AQ",
    "图片": "Image",
    "v1 字节": "v1 bytes",
    "v2 字节": "v2 bytes",
    "变化": "Change",
    "合计（几何平均）": "Total (geometric mean)",
    "主机实测：2048×1152 · q85 · 4:2:0 → 397 KB / 33.63 dB / 编码 ~115 ms、解码 ~130 ms（单核）。手机上会慢一些，安卓端因此把单张上限压到 8 MP。":
      "Measured on a desktop: 2048×1152 · q85 · 4:2:0 to 397 KB / 33.63 dB / encode ~115 ms, decode ~130 ms, single core. Phones are slower, which is why the Android side caps a single shot at 8 MP.",
    "手机上：相机直接写 .mtc": "On a phone: the camera writes .mtc directly",
    "整条路径不产生一张 JPEG。Camera2 的 YUV 帧进 Rust，出来的文件就已经是 MTC。":
      "Not one JPEG is produced along the way. Camera2 hands over YUV frames, Rust takes them, and what comes out is already MTC.",
    "三个界面": "Three screens",
    "相册": "Album",
    "：网格、缩略图缓存、长按删除、从文件导入":
      ": grid, cached thumbnails, long-press to delete, import from a file",
    "相机": "Camera",
    "：前后切换、色彩范围开关、实时显示角度":
      ": front/back switch, colour range toggle, live angle readout",
    "查看器": "Viewer",
    "：双指缩放、双击回中、导出 PNG 到系统相册":
      ": pinch to zoom, double-tap to recentre, export PNG to the album",
    "省内存的几件事": "Three things that save memory",
    "解码按目标尺寸直接出图（12 MP 峰值 92.8 MB → 400 px 时 64.9 MB），缩略图一次只解一张并落盘缓存，8 MP 上限，":
      "Decoding goes straight to the target size (12 MP peaks at 92.8 MB, 64.9 MB when the target is 400 px), thumbnails decode one at a time and land in a cache, and a shot is capped at 8 MP.",
    "安装包 783 KB，含 arm64 与 armeabi-v7a，无 Gradle、无 Maven，一条 PowerShell 脚本出包。":
      "The APK is 783 KB with arm64 and armeabi-v7a, built by one PowerShell script - no Gradle, no Maven.",
    "拿走它": "Take it",
    "页面会先认一下你用的系统，把合适的那个直接递给你。":
      "The page works out which system you are on and hands you the right one.",
    "全部版本": "All releases",
    "正在识别系统…": "Detecting your system…",
    "源代码": "Source",
    "格式规范": "Format spec",
    "反馈 / 建议：": "Feedback: ",
    "页面上的编码器是 WebAssembly，和命令行走同一份代码。":
      "The encoder on this page is WebAssembly, running the same code as the command line.",
    "自己写的": " of my own",
    "——同一份 Rust 代码， 编译成 WebAssembly，拖一张图进来就能看到。":
      " - the same Rust code, compiled to WebAssembly. Drop an image in and watch.",
    "页面上的编码器是 WebAssembly（容器 v2），和命令行走同一份 Rust 代码。":
      "The encoder here is WebAssembly (container v2), running the same Rust as the command line.",
    "下载安装程序": "Download the installer",
    "下载 Windows 安装程序": "Download for Windows",
    "检测到": "Detected",
    "：直接下载": ": download directly",
    "（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      " (2.8 MB). Double-click to install; it brings the command line, the viewer and the .mtc association.",
    "检测到 <b>Windows</b>：直接下载 <b>MtcSetup.exe</b>（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      "Detected <b>Windows</b>: download <b>MtcSetup.exe</b> (2.8 MB), double-click to install. It brings the command line, the viewer and the .mtc association.",
    "下载 Android 相机": "Download the Android camera",
    "下载 APK": "Download the APK",
    "检测到 <b>Android</b>：直接下载 <b>mtc-camera.apk</b>（0.8 MB），相机直出 .mtc，另有相册与查看器。":
      "Detected <b>Android</b>: download <b>mtc-camera.apk</b> (0.8 MB). The camera writes .mtc directly, and there is an album and a viewer.",
    "在 GitHub 上选择版本": "Pick a release on GitHub",
    "去 GitHub": "Go to GitHub",
    "检测到 <b>macOS</b>：这个项目目前只发布 Windows 和 Android 版本，所以带你去 GitHub 发布页。":
      "Detected <b>macOS</b>: this project ships Windows and Android only, so here is the releases page.",
    "检测到 <b>iPhone / iPad</b>：没有 iOS 版本，已在 GitHub 发布页。":
      "Detected <b>iPhone / iPad</b>: there is no iOS build; the releases page is the way in.",
    "检测到 <b>Linux</b>：没有打包好的 Linux 版本，但可以从源码构建（见发布页）。":
      "Detected <b>Linux</b>: no packaged build, but it does compile from source (see the releases page).",
    "没认出这个系统，带你去 GitHub 发布页自己挑。":
      "Could not tell what you are running, so here is the releases page.",
    "试试看": "Try it",
    "拖进来": "Drop it",
    "点这里选择文件": "Choose a file",
    "无损模式不采样色度，也不量化。": "Lossless mode neither subsamples chroma nor quantises.",
    "画质": "Quality",
    "体积": "Size",
    "更快": "Faster",
    "更小": "Smaller",
    "原汁原味": "Exactly as shot"
  };

  var JA = {
    "MTC · 自己写的图片编解码器": "MTC · 一から書いた画像コーデック",
    "试一下": "試す",
    "原理": "仕組み",
    "数据": "数値",
    "安卓": "Android",
    "下载": "ダウンロード",
    "Rust · 自研格式 · 无第三方编解码库": "Rust · 独自フォーマット · 外部コーデックなし",
    "把照片压小": "写真を小さく、",
    "用的还是": "使うのは",
    "自己写的": "自作の",
    "编解码器": "コーデック",
    "8×8 DCT、零区量化、上下文算术编码、无损与精确 alpha。": "8×8 DCT、デッドゾーン量子化、文脈算術符号化、ロスレスと正確なアルファ。",
    "这个页面里的每一个字节都是你的浏览器现场算出来的":
      "このページのバイトはすべて、いまあなたのブラウザが計算したものです",
    "——同一份 Rust 代码， 编译成 WebAssembly，拖一张图进来就能看到。":
      "——同じ Rust のコードを WebAssembly にしたもの。画像をドロップすれば見られます。",
    "立刻压一张图": "画像を圧縮する",
    "看源代码": "ソースを見る",
    "同画质体积更小": "同じ画質でより小さい",
    "2048×1152 · q85 的体积": "2048×1152 · q85 のサイズ",
    "第三方编解码依赖": "外部コーデック依存",
    "在浏览器里压一张": "ブラウザで圧縮する",
    "图片不出本机。编码、解码、PSNR 全在这里跑完。":
      "画像はこの端末から出ません。エンコード、デコード、PSNR はすべてここで完結します。",
    "原图": "元画像",
    "MTC 解码": "MTC デコード",
    "把图片拖进来": "画像をドロップ",
    "PNG / JPG / WebP，或者一个": "PNG / JPG / WebP、または",
    "选文件": "ファイルを選ぶ",
    "用样图试试": "サンプルで試す",
    "还没算过——数据会出现在右边。": "まだ計算していません——数値は右側に出ます。",
    "质量": "画質",
    "无损": "ロスレス",
    "自适应量化": "適応量子化",
    "(实测更费码率)": "（実際はビットレートが増える）",
    "原始像素": "元のピクセル",
    "每像素": "ピクセルあたり",
    "编码": "エンコード",
    "解码": "デコード",
    "一条真实的率失真曲线：横轴 PSNR，纵轴每像素字节，越靠左下越好。":
      "実際のレート歪み曲線：横軸 PSNR、縦軸はピクセルあたりのバイト数。左下ほど良い。",
    "下载 .mtc": ".mtc をダウンロード",
    "导出 PNG": "PNG を書き出す",
    "等待第一张图片…": "最初の画像を待っています…",
    "它到底怎么压的": "実際に何をしているか",
    "一条主线，五步，每一步都可以单独解释给你听。":
      "一本の流れ、五つの段階。どれも単独で説明できます。",
    "YCoCg 换色": "YCoCg 色変換",
    "可逆整数变换，把亮度与两个色差分开放。色度再用":
      "可逆な整数変換で、輝度と二つの色差を分ける。色差にはさらに",
    "的步长多压一点。": "粗いステップを使う。",
    "正交基，直流系数正好等于块均值 ×8，所以量化可以按像素级误差直接定步长。":
      "直交基底で、直流係数がブロック平均のちょうど 8 倍。だから量子化のステップはピクセル単位の誤差から直接決められる。",
    "零区量化": "デッドゾーン量子化",
    "交流系数用": "交流係数には",
    "个步长的零区：小的直接归零，省下的字节比丢掉的精度多得多。":
      "ステップ分のデッドゾーンを置く。小さいものはゼロになり、節約できるバイトは失う精度よりずっと大きい。",
    "上下文算术编码": "文脈算術符号化",
    "11 位自适应概率、按频带分模型，": "11 ビットの適応確率、帯域ごとのモデル、",
    "还用前一块的稀疏程度做上下文。": "さらに前のブロックの疎密を文脈に使う。",
    "容器": "コンテナ",
    "12 字节头加若干数据段。同一份容器放有损、无损、alpha，版本号只决定系数模型的布局。":
      "12 バイトのヘッダとデータセグメント。同じコンテナに非可逆・可逆・アルファを載せ、バージョンは係数モデルの並びだけを決める。",
    "自适应量化，为什么默认关着": "適応量子化と、既定で切っている理由",
    "每个 16×16 瓦片一个倍率（0.70×–1.40×），编码器按瓦片活跃度选档、对前一片差分编码、向低频倾斜。":
      "16×16 のタイルごとに倍率（0.70×–1.40×）。エンコーダはタイルの活動度で段を選び、前のタイルとの差分を符号化し、低域に寄せる。",
    "固定": "固定",
    "下它确实更好看（约 +0.3 dB），但同 PSNR 下要多花":
      "のほうが見た目は良い（約 +0.3 dB）が、同じ PSNR では",
    "字节。所以默认关，勾上就能在这个页面里自己复现。":
      "バイト増える。だから既定では切ってあり、チェックすればこのページで再現できる。",
    "12 字节的头，就这么点": "12 バイトのヘッダ、これだけ",
    "上面那堆十六进制就是你这张图的真实头 12 字节。没有 EXIF，没有元数据块——格式小到可以在一屏里讲完。":
      "上の十六進数が、この画像の本当の 12 バイトヘッダ。EXIF もメタデータ塊もない——一画面で説明しきれる小ささ。",
    "测出来的，不是吹出来的": "測った値であり、誇張ではない",
    "四张 512×512 实拍，每张六个质量档，全部按":
      "実写 512×512 を四枚、各六つの画質段階で、すべて同じ",
    "同 PSNR": "PSNR",
    "比较（同一个": "で比較（同じ",
    "比是不公平的）。": "で比べるのは不公平）。",
    "交流零区": "交流デッドゾーン",
    "DC 二维预测": "DC 二次元予測",
    "last 上下文": "last 文脈",
    "频带上下文": "帯域文脈",
    "逐瓦片 AQ": "タイル単位 AQ",
    "图片": "画像",
    "v1 字节": "v1 バイト",
    "v2 字节": "v2 バイト",
    "变化": "変化",
    "合计（几何平均）": "合計（幾何平均）",
    "主机实测：2048×1152 · q85 · 4:2:0 → 397 KB / 33.63 dB / 编码 ~115 ms、解码 ~130 ms（单核）。手机上会慢一些，安卓端因此把单张上限压到 8 MP。":
      "実機測定：2048×1152 · q85 · 4:2:0 → 397 KB / 33.63 dB / エンコード約 115 ms、デコード約 130 ms（シングルコア）。スマホは遅いので、Android 側は一枚の上限を 8 MP にしている。",
    "手机上：相机直接写 .mtc": "スマホでは：カメラが直接 .mtc を書く",
    "整条路径不产生一张 JPEG。Camera2 的 YUV 帧进 Rust，出来的文件就已经是 MTC。":
      "途中で JPEG は一枚も作らない。Camera2 の YUV フレームが Rust に入り、出てくるファイルはすでに MTC。",
    "三个界面": "三つの画面",
    "相册": "アルバム",
    "：网格、缩略图缓存、长按删除、从文件导入":
      "：グリッド、サムネイルのキャッシュ、長押しで削除、ファイルから取り込み",
    "相机": "カメラ",
    "：前后切换、色彩范围开关、实时显示角度":
      "：前後切替、色域の切替、角度の表示",
    "查看器": "ビューア",
    "：双指缩放、双击回中、导出 PNG 到系统相册":
      "：二本指でズーム、ダブルタップで中央へ、PNG をアルバムへ書き出し",
    "省内存的几件事": "メモリを節約する三つの工夫",
    "解码按目标尺寸直接出图（12 MP 峰值 92.8 MB → 400 px 时 64.9 MB），缩略图一次只解一张并落盘缓存，8 MP 上限，":
      "デコードは目標サイズへ直接出力（12 MP のピーク 92.8 MB、400 px なら 64.9 MB）、サムネイルは一枚ずつ解いてキャッシュに保存、上限 8 MP、",
    "安装包 783 KB，含 arm64 与 armeabi-v7a，无 Gradle、无 Maven，一条 PowerShell 脚本出包。":
      "APK は 783 KB、arm64 と armeabi-v7a を収録、Gradle も Maven も使わず PowerShell 一本でビルド。",
    "拿走它": "持っていく",
    "页面会先认一下你用的系统，把合适的那个直接递给你。":
      "ページがあなたの OS を見分けて、合うものをそのまま渡します。",
    "全部版本": "すべてのリリース",
    "正在识别系统…": "システムを判定中…",
    "源代码": "ソースコード",
    "格式规范": "フォーマット仕様",
    "反馈 / 建议：": "ご意見：",
    "页面上的编码器是 WebAssembly，和命令行走同一份代码。":
      "このページのエンコーダは WebAssembly で、コマンドラインと同じコードです。",
    "下载安装程序": "インストーラーをダウンロード",
    "下载 Windows 安装程序": "Windows 版をダウンロード",
    "检测到": "検出",
    "：直接下载": "：直接ダウンロード",
    "（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      "（2.8 MB）。ダブルクリックでインストール。コマンドライン、ビューア、.mtc の関連付けが入ります。",
    "检测到 <b>Windows</b>：直接下载 <b>MtcSetup.exe</b>（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      "<b>Windows</b> を検出：<b>MtcSetup.exe</b>（2.8 MB）をダウンロードして実行。コマンドライン、ビューア、.mtc の関連付けが入ります。",
    "下载 Android 相机": "Android 版カメラをダウンロード",
    "下载 APK": "APK をダウンロード",
    "检测到 <b>Android</b>：直接下载 <b>mtc-camera.apk</b>（0.8 MB），相机直出 .mtc，另有相册与查看器。":
      "<b>Android</b> を検出：<b>mtc-camera.apk</b>（0.8 MB）をダウンロード。カメラが直接 .mtc を書き、アルバムとビューアも入っています。",
    "在 GitHub 上选择版本": "GitHub で版を選ぶ",
    "去 GitHub": "GitHub へ",
    "检测到 <b>macOS</b>：这个项目目前只发布 Windows 和 Android 版本，所以带你去 GitHub 发布页。":
      "<b>macOS</b> を検出：このプロジェクトは Windows と Android のみなので、リリースページへご案内します。",
    "检测到 <b>iPhone / iPad</b>：没有 iOS 版本，已在 GitHub 发布页。":
      "<b>iPhone / iPad</b> を検出：iOS 版はありません。リリースページへどうぞ。",
    "检测到 <b>Linux</b>：没有打包好的 Linux 版本，但可以从源码构建（见发布页）。":
      "<b>Linux</b> を検出：パッケージ版はありませんが、ソースからビルドできます（リリースページ参照）。",
    "没认出这个系统，带你去 GitHub 发布页自己挑。":
      "システムを判別できませんでした。リリースページで選んでください。",
    "试试看": "試してみる",
    "拖进来": "ドロップ",
    "点这里选择文件": "ファイルを選ぶ",
    "无损模式不采样色度，也不量化。": "ロスレスでは色差を間引かず、量子化もしません。",
    "画质": "画質",
    "体积": "サイズ",
    "更快": "より速く",
    "更小": "より小さく",
    "原汁原味": "撮ったまま",
    "载入样图…": "サンプルを読み込み中…",
    "出错了：": "エラー：",
    "样图加载失败：": "サンプルの読み込みに失敗：",
    "编解码器加载失败：": "コーデックの読み込みに失敗：",
    "样图没能在 3 秒内加载，用一张本地生成的图代替（编码器是真的）":
      "サンプルが 3 秒以内に読み込めなかったため、ローカルで描いた画像で代用します（エンコーダは本物です）",
    "自检图": "セルフチェック画像",
    "样图": "サンプル"
  };

  var KO = {
    "MTC · 自己写的图片编解码器": "MTC · 직접 만든 이미지 코덱",
    "试一下": "써 보기",
    "原理": "원리",
    "数据": "수치",
    "安卓": "안드로이드",
    "下载": "다운로드",
    "Rust · 自研格式 · 无第三方编解码库": "Rust · 자체 포맷 · 외부 코덱 없음",
    "把照片压小": "사진을 더 작게,",
    "用的还是": "사용한 것은",
    "自己写的": "직접 만든",
    "编解码器": "코덱",
    "8×8 DCT、零区量化、上下文算术编码、无损与精确 alpha。": "8×8 DCT, 데드존 양자화, 문맥 산술 부호화, 무손실과 정확한 알파.",
    "这个页面里的每一个字节都是你的浏览器现场算出来的":
      "이 페이지의 모든 바이트는 지금 당신의 브라우저가 계산한 것입니다",
    "——同一份 Rust 代码， 编译成 WebAssembly，拖一张图进来就能看到。":
      "— 같은 Rust 코드를 WebAssembly로 빌드한 것. 이미지를 끌어다 놓으면 볼 수 있습니다.",
    "立刻压一张图": "이미지 압축하기",
    "看源代码": "소스 보기",
    "同画质体积更小": "같은 화질에 더 작게",
    "2048×1152 · q85 的体积": "2048×1152 · q85 크기",
    "第三方编解码依赖": "외부 코덱 의존",
    "在浏览器里压一张": "브라우저에서 압축하기",
    "图片不出本机。编码、解码、PSNR 全在这里跑完。":
      "이미지는 이 기기를 떠나지 않습니다. 인코딩, 디코딩, PSNR 모두 여기서 끝납니다.",
    "原图": "원본",
    "MTC 解码": "MTC 디코드",
    "把图片拖进来": "이미지를 끌어다 놓기",
    "PNG / JPG / WebP，或者一个": "PNG / JPG / WebP 또는",
    "选文件": "파일 선택",
    "用样图试试": "샘플로 시험",
    "还没算过——数据会出现在右边。": "아직 계산하지 않았습니다 — 수치는 오른쪽에 나옵니다.",
    "质量": "화질",
    "无损": "무손실",
    "自适应量化": "적응 양자화",
    "(实测更费码率)": "(실제로는 비트레이트가 늘어남)",
    "原始像素": "원본 픽셀",
    "每像素": "픽셀당",
    "编码": "인코드",
    "解码": "디코드",
    "一条真实的率失真曲线：横轴 PSNR，纵轴每像素字节，越靠左下越好。":
      "실제 율-왜곡 곡선: 가로축 PSNR, 세로축 픽셀당 바이트. 왼쪽 아래일수록 좋습니다.",
    "下载 .mtc": ".mtc 내려받기",
    "导出 PNG": "PNG 내보내기",
    "等待第一张图片…": "첫 이미지를 기다리는 중…",
    "它到底怎么压的": "실제로 하는 일",
    "一条主线，五步，每一步都可以单独解释给你听。":
      "하나의 흐름, 다섯 단계. 각각 따로 설명할 수 있습니다.",
    "YCoCg 换色": "YCoCg 색 변환",
    "可逆整数变换，把亮度与两个色差分开放。色度再用":
      "가역 정수 변환으로 밝기와 두 색차를 분리합니다. 색차에는 더",
    "的步长多压一点。": "거친 단계를 씁니다.",
    "正交基，直流系数正好等于块均值 ×8，所以量化可以按像素级误差直接定步长。":
      "직교 기저이고 DC 계수가 블록 평균의 정확히 8배라서, 양자화 단계를 픽셀 단위 오차에서 바로 정할 수 있습니다.",
    "零区量化": "데드존 양자화",
    "交流系数用": "AC 계수에는",
    "个步长的零区：小的直接归零，省下的字节比丢掉的精度多得多。":
      "단계만큼의 데드존을 둡니다. 작은 값은 0이 되고, 아낀 바이트가 잃은 정밀도보다 훨씬 큽니다.",
    "上下文算术编码": "문맥 산술 부호화",
    "11 位自适应概率、按频带分模型，": "11비트 적응 확률, 대역별 모델,",
    "还用前一块的稀疏程度做上下文。": "이전 블록의 희소성을 문맥으로 씁니다.",
    "容器": "컨테이너",
    "12 字节头加若干数据段。同一份容器放有损、无损、alpha，版本号只决定系数模型的布局。":
      "12바이트 헤더와 데이터 세그먼트. 같은 컨테이너에 손실, 무손실, 알파를 담고, 버전은 계수 모델 배치만 정합니다.",
    "自适应量化，为什么默认关着": "적응 양자화, 기본으로 꺼 둔 이유",
    "每个 16×16 瓦片一个倍率（0.70×–1.40×），编码器按瓦片活跃度选档、对前一片差分编码、向低频倾斜。":
      "16×16 타일마다 배율(0.70×–1.40×). 인코더는 타일 활동도로 단계를 고르고, 이전 타일과의 차이를 부호화하며, 저주파로 기웁니다.",
    "固定": "고정",
    "下它确实更好看（约 +0.3 dB），但同 PSNR 下要多花":
      "에서는 확실히 더 좋아 보이지만(약 +0.3 dB), 같은 PSNR에서",
    "字节。所以默认关，勾上就能在这个页面里自己复现。":
      "바이트가 더 듭니다. 그래서 기본은 꺼져 있고, 체크하면 이 페이지에서 재현할 수 있습니다.",
    "12 字节的头，就这么点": "12바이트 헤더, 이게 전부",
    "上面那堆十六进制就是你这张图的真实头 12 字节。没有 EXIF，没有元数据块——格式小到可以在一屏里讲完。":
      "위 16진수가 이 사진의 진짜 12바이트 헤더입니다. EXIF도 메타데이터 블록도 없습니다 — 한 화면에 다 설명되는 크기.",
    "测出来的，不是吹出来的": "측정한 값이고 과장이 아님",
    "四张 512×512 实拍，每张六个质量档，全部按":
      "실사 512×512 네 장, 각 여섯 단계, 모두 같은",
    "同 PSNR": "PSNR",
    "比较（同一个": "에서 비교(같은",
    "比是不公平的）。": "로 비교하는 건 불공정).",
    "交流零区": "AC 데드존",
    "DC 二维预测": "DC 2차원 예측",
    "last 上下文": "last 문맥",
    "频带上下文": "대역 문맥",
    "逐瓦片 AQ": "타일별 AQ",
    "图片": "이미지",
    "v1 字节": "v1 바이트",
    "v2 字节": "v2 바이트",
    "变化": "변화",
    "合计（几何平均）": "합계(기하평균)",
    "主机实测：2048×1152 · q85 · 4:2:0 → 397 KB / 33.63 dB / 编码 ~115 ms、解码 ~130 ms（单核）。手机上会慢一些，安卓端因此把单张上限压到 8 MP。":
      "데스크톱 실측: 2048×1152 · q85 · 4:2:0 → 397 KB / 33.63 dB / 인코딩 약 115 ms, 디코딩 약 130 ms(단일 코어). 휴대폰은 느려서 안드로이드 쪽은 한 장을 8 MP로 제한합니다.",
    "手机上：相机直接写 .mtc": "휴대폰에서는: 카메라가 바로 .mtc를 씁니다",
    "整条路径不产生一张 JPEG。Camera2 的 YUV 帧进 Rust，出来的文件就已经是 MTC。":
      "경로 어디에서도 JPEG를 만들지 않습니다. Camera2의 YUV 프레임이 Rust로 들어가고, 나오는 파일이 이미 MTC입니다.",
    "三个界面": "세 개의 화면",
    "相册": "앨범",
    "：网格、缩略图缓存、长按删除、从文件导入":
      ": 그리드, 썸네일 캐시, 길게 눌러 삭제, 파일에서 가져오기",
    "相机": "카메라",
    "：前后切换、色彩范围开关、实时显示角度":
      ": 전후면 전환, 색역 전환, 각도 표시",
    "查看器": "뷰어",
    "：双指缩放、双击回中、导出 PNG 到系统相册":
      ": 두 손가락 확대, 두 번 탭해 가운데로, PNG를 앨범으로 내보내기",
    "省内存的几件事": "메모리를 아끼는 세 가지",
    "解码按目标尺寸直接出图（12 MP 峰值 92.8 MB → 400 px 时 64.9 MB），缩略图一次只解一张并落盘缓存，8 MP 上限，":
      "디코딩이 목표 크기로 바로 출력되고(12 MP 최대 92.8 MB, 400 px면 64.9 MB), 썸네일은 한 장씩 풀어 캐시에 저장하며, 한 장은 8 MP로 제한,",
    "安装包 783 KB，含 arm64 与 armeabi-v7a，无 Gradle、无 Maven，一条 PowerShell 脚本出包。":
      "APK는 783 KB, arm64와 armeabi-v7a 포함, Gradle도 Maven도 없이 PowerShell 스크립트 하나로 빌드.",
    "拿走它": "가져가기",
    "页面会先认一下你用的系统，把合适的那个直接递给你。":
      "페이지가 당신의 시스템을 알아보고 맞는 것을 바로 건넵니다.",
    "全部版本": "모든 릴리스",
    "正在识别系统…": "시스템 확인 중…",
    "源代码": "소스 코드",
    "格式规范": "포맷 규격",
    "反馈 / 建议：": "의견: ",
    "页面上的编码器是 WebAssembly，和命令行走同一份代码。":
      "이 페이지의 인코더는 WebAssembly이고, 명령줄과 같은 코드입니다.",
    "下载安装程序": "설치 프로그램 받기",
    "下载 Windows 安装程序": "Windows용 받기",
    "检测到": "감지됨",
    "：直接下载": ": 바로 받기",
    "（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      " (2.8 MB). 두 번 눌러 설치하면 명령줄, 뷰어, .mtc 연결이 함께 들어갑니다.",
    "检测到 <b>Windows</b>：直接下载 <b>MtcSetup.exe</b>（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      "<b>Windows</b> 감지: <b>MtcSetup.exe</b>(2.8 MB)를 받아 실행하세요. 명령줄, 뷰어, .mtc 연결이 들어갑니다.",
    "下载 Android 相机": "Android 카메라 받기",
    "下载 APK": "APK 받기",
    "检测到 <b>Android</b>：直接下载 <b>mtc-camera.apk</b>（0.8 MB），相机直出 .mtc，另有相册与查看器。":
      "<b>Android</b> 감지: <b>mtc-camera.apk</b>(0.8 MB)를 받으세요. 카메라가 바로 .mtc를 쓰고 앨범과 뷰어도 있습니다.",
    "在 GitHub 上选择版本": "GitHub에서 버전 선택",
    "去 GitHub": "GitHub로",
    "检测到 <b>macOS</b>：这个项目目前只发布 Windows 和 Android 版本，所以带你去 GitHub 发布页。":
      "<b>macOS</b> 감지: 이 프로젝트는 Windows와 Android만 배포하므로 릴리스 페이지로 안내합니다.",
    "检测到 <b>iPhone / iPad</b>：没有 iOS 版本，已在 GitHub 发布页。":
      "<b>iPhone / iPad</b> 감지: iOS 빌드는 없습니다. 릴리스 페이지를 보세요.",
    "检测到 <b>Linux</b>：没有打包好的 Linux 版本，但可以从源码构建（见发布页）。":
      "<b>Linux</b> 감지: 패키지 빌드는 없지만 소스에서 빌드할 수 있습니다(릴리스 페이지 참고).",
    "没认出这个系统，带你去 GitHub 发布页自己挑。":
      "시스템을 알아보지 못했습니다. 릴리스 페이지에서 골라 주세요.",
    "试试看": "한번 해 보기",
    "拖进来": "끌어다 놓기",
    "点这里选择文件": "파일 선택",
    "无损模式不采样色度，也不量化。": "무손실 모드는 색차를 줄이지도, 양자화하지도 않습니다.",
    "画质": "화질",
    "体积": "크기",
    "更快": "더 빠르게",
    "更小": "더 작게",
    "原汁原味": "찍은 그대로",
    "载入样图…": "샘플을 불러오는 중…",
    "出错了：": "오류: ",
    "样图加载失败：": "샘플을 불러오지 못했습니다: ",
    "编解码器加载失败：": "코덱을 불러오지 못했습니다: ",
    "样图没能在 3 秒内加载，用一张本地生成的图代替（编码器是真的）":
      "샘플이 3초 안에 오지 않아 로컬에서 그린 이미지로 대신합니다(인코더는 진짜입니다)",
    "自检图": "자체 점검 이미지",
    "样图": "샘플"
  };

  var RU = {
    "MTC · 自己写的图片编解码器": "MTC · кодек изображений, написанный с нуля",
    "试一下": "Попробовать",
    "原理": "Как работает",
    "数据": "Цифры",
    "安卓": "Android",
    "下载": "Скачать",
    "Rust · 自研格式 · 无第三方编解码库": "Rust · собственный формат · без сторонних кодеков",
    "把照片压小": "Фотографии меньше,",
    "用的还是": "и всё это на",
    "自己写的": " собственном",
    "编解码器": " кодеке",
    "8×8 DCT、零区量化、上下文算术编码、无损与精确 alpha。": "ДКП 8×8, квантование с мёртвой зоной, контекстное арифметическое кодирование, сжатие без потерь и точная альфа.",
    "这个页面里的每一个字节都是你的浏览器现场算出来的":
      "Каждый байт на этой странице посчитан вашим браузером прямо сейчас",
    "——同一份 Rust 代码， 编译成 WebAssembly，拖一张图进来就能看到。":
      " - тот же код на Rust, собранный в WebAssembly. Перетащите картинку и смотрите.",
    "立刻压一张图": "Сжать изображение",
    "看源代码": "Смотреть код",
    "同画质体积更小": "меньше при том же качестве",
    "2048×1152 · q85 的体积": "размер при 2048×1152 · q85",
    "第三方编解码依赖": "сторонних кодеков",
    "在浏览器里压一张": "Сожмите картинку в браузере",
    "图片不出本机。编码、解码、PSNR 全在这里跑完。":
      "Изображение не покидает эту машину. Кодирование, декодирование и PSNR считаются здесь.",
    "原图": "Оригинал",
    "MTC 解码": "Декодировано MTC",
    "把图片拖进来": "Перетащите изображение",
    "PNG / JPG / WebP，或者一个": "PNG / JPG / WebP или",
    "选文件": "выберите файл",
    "用样图试试": "Попробовать образец",
    "还没算过——数据会出现在右边。": "Пока ничего не посчитано - цифры появятся справа.",
    "质量": "Качество",
    "无损": "Без потерь",
    "自适应量化": "Адаптивное квантование",
    "(实测更费码率)": "(на практике дороже по байтам)",
    "原始像素": "Исходные пиксели",
    "每像素": "На пиксель",
    "编码": "Кодирование",
    "解码": "Декодирование",
    "一条真实的率失真曲线：横轴 PSNR，纵轴每像素字节，越靠左下越好。":
      "Настоящая кривая скорость/искажение: по горизонтали PSNR, по вертикали байты на пиксель. Левый низ - лучше.",
    "下载 .mtc": "Скачать .mtc",
    "导出 PNG": "Экспорт PNG",
    "等待第一张图片…": "Ждём первое изображение…",
    "它到底怎么压的": "Что именно происходит",
    "一条主线，五步，每一步都可以单独解释给你听。":
      "Один конвейер, пять шагов, и каждый можно объяснить отдельно.",
    "YCoCg 换色": "Цветовое пространство YCoCg",
    "可逆整数变换，把亮度与两个色差分开放。色度再用":
      "Обратимое целочисленное преобразование, разделяющее яркость и две цветовые компоненты. К цветности затем применяется",
    "的步长多压一点。": "более грубый шаг.",
    "正交基，直流系数正好等于块均值 ×8，所以量化可以按像素级误差直接定步长。":
      "Ортогональный базис, у которого постоянная составляющая ровно в восемь раз больше среднего по блоку, - поэтому шаг квантования задаётся прямо из допуска ошибки на пиксель.",
    "零区量化": "Квантование с мёртвой зоной",
    "交流系数用": "У переменных коэффициентов мёртвая зона шириной в",
    "个步长的零区：小的直接归零，省下的字节比丢掉的精度多得多。":
      "шаг: малые обнуляются, и сэкономленные байты стоят куда больше потерянной точности.",
    "上下文算术编码": "Контекстное арифметическое кодирование",
    "11 位自适应概率、按频带分模型，": "11-битные адаптивные вероятности, отдельная модель на полосу,",
    "还用前一块的稀疏程度做上下文。": "а разреженность предыдущего блока служит контекстом.",
    "容器": "Контейнер",
    "12 字节头加若干数据段。同一份容器放有损、无损、alpha，版本号只决定系数模型的布局。":
      "Заголовок в 12 байт и несколько сегментов. Один и тот же контейнер несёт сжатие с потерями, без потерь и альфу; версия задаёт лишь раскладку моделей коэффициентов.",
    "自适应量化，为什么默认关着": "Адаптивное квантование и почему оно выключено",
    "每个 16×16 瓦片一个倍率（0.70×–1.40×），编码器按瓦片活跃度选档、对前一片差分编码、向低频倾斜。":
      "Каждый блок 16×16 получает свой множитель (0.70×-1.40×): кодировщик выбирает шаг по активности блока, кодирует разницу с предыдущим и смещается к низким частотам.",
    "固定": "При фиксированном",
    "下它确实更好看（约 +0.3 dB），但同 PSNR 下要多花":
      "шаге картинка правда лучше (примерно +0.3 дБ), но при том же PSNR стоит",
    "字节。所以默认关，勾上就能在这个页面里自己复现。":
      "больше байт. Поэтому по умолчанию выключено; поставьте галочку и повторите это здесь же.",
    "12 字节的头，就这么点": "Заголовок в 12 байт. Вот и весь он",
    "上面那堆十六进制就是你这张图的真实头 12 字节。没有 EXIF，没有元数据块——格式小到可以在一屏里讲完。":
      "Шестнадцатеричный код выше - настоящий 12-байтовый заголовок вашей картинки. Ни EXIF, ни блоков метаданных: формат укладывается в один экран.",
    "测出来的，不是吹出来的": "Измерено, а не обещано",
    "四张 512×512 实拍，每张六个质量档，全部按":
      "Четыре реальных снимка 512×512, по шесть уровней качества, всё сравнивается при одинаковом",
    "同 PSNR": "PSNR",
    "比较（同一个": "(одинаковое",
    "比是不公平的）。": "сравнивать нечестно).",
    "交流零区": "Мёртвая зона AC",
    "DC 二维预测": "2D-предсказание DC",
    "last 上下文": "контекст last",
    "频带上下文": "контекст полосы",
    "逐瓦片 AQ": "AQ по блокам",
    "图片": "Снимок",
    "v1 字节": "байт v1",
    "v2 字节": "байт v2",
    "变化": "Разница",
    "合计（几何平均）": "Итого (среднее геометрическое)",
    "主机实测：2048×1152 · q85 · 4:2:0 → 397 KB / 33.63 dB / 编码 ~115 ms、解码 ~130 ms（单核）。手机上会慢一些，安卓端因此把单张上限压到 8 MP。":
      "Замер на настольном ПК: 2048×1152 · q85 · 4:2:0 в 397 КБ / 33.63 дБ / кодирование ~115 мс, декодирование ~130 мс на одном ядре. Телефоны медленнее, поэтому на Android один снимок ограничен 8 МП.",
    "手机上：相机直接写 .mtc": "На телефоне: камера пишет .mtc напрямую",
    "整条路径不产生一张 JPEG。Camera2 的 YUV 帧进 Rust，出来的文件就已经是 MTC。":
      "По пути не создаётся ни одного JPEG. Camera2 отдаёт YUV-кадры, Rust их принимает, и на выходе уже MTC.",
    "三个界面": "Три экрана",
    "相册": "Альбом",
    "：网格、缩略图缓存、长按删除、从文件导入":
      ": сетка, кэш миниатюр, удаление долгим нажатием, импорт из файла",
    "相机": "Камера",
    "：前后切换、色彩范围开关、实时显示角度":
      ": переключение камер, диапазон цвета, живой вывод угла",
    "查看器": "Просмотр",
    "：双指缩放、双击回中、导出 PNG 到系统相册":
      ": масштаб двумя пальцами, двойной тап по центру, экспорт PNG в галерею",
    "省内存的几件事": "Три вещи, которые экономят память",
    "解码按目标尺寸直接出图（12 MP 峰值 92.8 MB → 400 px 时 64.9 MB），缩略图一次只解一张并落盘缓存，8 MP 上限，":
      "Декодирование сразу в нужный размер (12 МП дают пик 92.8 МБ, а при цели 400 px - 64.9 МБ), миниатюры декодируются по одной и попадают в кэш, снимок ограничен 8 МП,",
    "安装包 783 KB，含 arm64 与 armeabi-v7a，无 Gradle、无 Maven，一条 PowerShell 脚本出包。":
      "APK весит 783 КБ, содержит arm64 и armeabi-v7a, собирается одним скриптом PowerShell - без Gradle и Maven.",
    "拿走它": "Забрать",
    "页面会先认一下你用的系统，把合适的那个直接递给你。":
      "Страница определит вашу систему и сразу предложит подходящий вариант.",
    "全部版本": "Все версии",
    "正在识别系统…": "Определяем систему…",
    "源代码": "Исходники",
    "格式规范": "Спецификация",
    "反馈 / 建议：": "Отзывы: ",
    "页面上的编码器是 WebAssembly，和命令行走同一份代码。":
      "Кодировщик на этой странице - WebAssembly, тот же код, что в командной строке.",
    "下载安装程序": "Скачать установщик",
    "下载 Windows 安装程序": "Скачать для Windows",
    "检测到": "Обнаружено",
    "：直接下载": ": скачать напрямую",
    "（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      " (2.8 МБ). Двойной щелчок - установка; внутри командная строка, просмотрщик и связь с .mtc.",
    "检测到 <b>Windows</b>：直接下载 <b>MtcSetup.exe</b>（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      "Обнаружено <b>Windows</b>: скачайте <b>MtcSetup.exe</b> (2.8 МБ) и запустите. Внутри командная строка, просмотрщик и связь с .mtc.",
    "下载 Android 相机": "Скачать камеру для Android",
    "下载 APK": "Скачать APK",
    "检测到 <b>Android</b>：直接下载 <b>mtc-camera.apk</b>（0.8 MB），相机直出 .mtc，另有相册与查看器。":
      "Обнаружено <b>Android</b>: скачайте <b>mtc-camera.apk</b> (0.8 МБ). Камера пишет .mtc напрямую, плюс альбом и просмотрщик.",
    "在 GitHub 上选择版本": "Выбрать версию на GitHub",
    "去 GitHub": "Перейти на GitHub",
    "检测到 <b>macOS</b>：这个项目目前只发布 Windows 和 Android 版本，所以带你去 GitHub 发布页。":
      "Обнаружено <b>macOS</b>: проект выходит только для Windows и Android, поэтому вот страница релизов.",
    "检测到 <b>iPhone / iPad</b>：没有 iOS 版本，已在 GitHub 发布页。":
      "Обнаружено <b>iPhone / iPad</b>: сборки для iOS нет, вот страница релизов.",
    "检测到 <b>Linux</b>：没有打包好的 Linux 版本，但可以从源码构建（见发布页）。":
      "Обнаружено <b>Linux</b>: готовой сборки нет, но проект собирается из исходников (см. страницу релизов).",
    "没认出这个系统，带你去 GitHub 发布页自己挑。":
      "Систему распознать не удалось, поэтому вот страница релизов.",
    "试试看": "Попробуйте",
    "拖进来": "Перетащите",
    "点这里选择文件": "Выберите файл",
    "无损模式不采样色度，也不量化。": "Режим без потерь не прореживает цветность и не квантует.",
    "画质": "Качество",
    "体积": "Размер",
    "更快": "Быстрее",
    "更小": "Меньше",
    "原汁原味": "Точно как снято",
    "等待第一张图片…": "Ждём первое изображение…",
    "载入样图…": "Загружаем образец…",
    "出错了：": "Ошибка: ",
    "样图加载失败：": "Не удалось загрузить образец: ",
    "编解码器加载失败：": "Не удалось загрузить кодек: ",
    "样图没能在 3 秒内加载，用一张本地生成的图代替（编码器是真的）":
      "Образец не загрузился за 3 секунды - вместо него локально нарисованная картинка (кодировщик настоящий)",
    "自检图": "Тестовая картинка",
    "样图": "Образец"
  };

  var ES = {
    "MTC · 自己写的图片编解码器": "MTC · un códec de imagen escrito desde cero",
    "试一下": "Probar",
    "原理": "Cómo funciona",
    "数据": "Cifras",
    "安卓": "Android",
    "下载": "Descargar",
    "Rust · 自研格式 · 无第三方编解码库": "Rust · formato propio · sin códecs de terceros",
    "把照片压小": "Fotos más pequeñas,",
    "用的还是": "con un códec",
    "自己写的": " propio",
    "编解码器": "",
    "8×8 DCT、零区量化、上下文算术编码、无损与精确 alpha。": "DCT de 8×8, cuantización con zona muerta, codificación aritmética con contexto, sin pérdida y alfa exacto.",
    "这个页面里的每一个字节都是你的浏览器现场算出来的":
      "Cada byte de esta página lo está calculando tu navegador ahora mismo",
    "——同一份 Rust 代码， 编译成 WebAssembly，拖一张图进来就能看到。":
      " — el mismo código en Rust, compilado a WebAssembly. Arrastra una imagen y míralo.",
    "立刻压一张图": "Comprimir una imagen",
    "看源代码": "Ver el código",
    "同画质体积更小": "más pequeñas con la misma calidad",
    "2048×1152 · q85 的体积": "tamaño a 2048×1152 · q85",
    "第三方编解码依赖": "códecs de terceros",
    "在浏览器里压一张": "Comprime una en el navegador",
    "图片不出本机。编码、解码、PSNR 全在这里跑完。":
      "La imagen no sale de este equipo. Codificación, decodificación y PSNR ocurren aquí.",
    "原图": "Original",
    "MTC 解码": "Decodificado con MTC",
    "把图片拖进来": "Arrastra una imagen",
    "PNG / JPG / WebP，或者一个": "PNG / JPG / WebP, o",
    "选文件": "elige un archivo",
    "用样图试试": "Probar una muestra",
    "还没算过——数据会出现在右边。": "Aún no se ha calculado nada: los datos salen a la derecha.",
    "质量": "Calidad",
    "无损": "Sin pérdida",
    "自适应量化": "Cuantización adaptativa",
    "(实测更费码率)": "(en la práctica cuesta bytes)",
    "原始像素": "Píxeles originales",
    "每像素": "Por píxel",
    "编码": "Codificar",
    "解码": "Decodificar",
    "一条真实的率失真曲线：横轴 PSNR，纵轴每像素字节，越靠左下越好。":
      "Una curva real de tasa/distorsión: PSNR en horizontal, bytes por píxel en vertical. Abajo a la izquierda es mejor.",
    "下载 .mtc": "Descargar .mtc",
    "导出 PNG": "Exportar PNG",
    "等待第一张图片…": "Esperando la primera imagen…",
    "它到底怎么压的": "Qué hace en realidad",
    "一条主线，五步，每一步都可以单独解释给你听。":
      "Una tubería, cinco pasos, y cada uno se puede explicar por separado.",
    "YCoCg 换色": "Color YCoCg",
    "可逆整数变换，把亮度与两个色差分开放。色度再用":
      "Una transformada entera reversible que separa la luminancia de dos componentes de croma. Al croma se le aplica",
    "的步长多压一点。": "un paso más grueso.",
    "正交基，直流系数正好等于块均值 ×8，所以量化可以按像素级误差直接定步长。":
      "Una base ortogonal cuyo coeficiente DC es exactamente ocho veces la media del bloque, así que el paso de cuantización se fija directamente con un presupuesto de error por píxel.",
    "零区量化": "Cuantización con zona muerta",
    "交流系数用": "Los coeficientes AC usan una",
    "个步长的零区：小的直接归零，省下的字节比丢掉的精度多得多。":
      "zona muerta de un paso: los pequeños van a cero, y los bytes ahorrados valen mucho más que la precisión perdida.",
    "上下文算术编码": "Codificación aritmética con contexto",
    "11 位自适应概率、按频带分模型，": "probabilidades adaptativas de 11 bits, un modelo por banda,",
    "还用前一块的稀疏程度做上下文。": "y la dispersión del bloque anterior como contexto.",
    "容器": "Contenedor",
    "12 字节头加若干数据段。同一份容器放有损、无损、alpha，版本号只决定系数模型的布局。":
      "Una cabecera de 12 bytes y varios segmentos. El mismo contenedor lleva con pérdida, sin pérdida y alfa; la versión solo decide el reparto de los modelos de coeficientes.",
    "自适应量化，为什么默认关着": "Cuantización adaptativa, y por qué viene apagada",
    "每个 16×16 瓦片一个倍率（0.70×–1.40×），编码器按瓦片活跃度选档、对前一片差分编码、向低频倾斜。":
      "Cada baldosa de 16×16 recibe un multiplicador (0.70×–1.40×). El codificador elige el paso según la actividad, lo codifica como diferencia con la anterior y se inclina hacia bajas frecuencias.",
    "固定": "Con un paso fijo",
    "下它确实更好看（约 +0.3 dB），但同 PSNR 下要多花":
      "se ve genuinamente mejor (unos +0.3 dB) pero cuesta",
    "字节。所以默认关，勾上就能在这个页面里自己复现。":
      "más bytes al mismo PSNR. Por eso viene apagada; márcala y reprodúcelo aquí mismo.",
    "12 字节的头，就这么点": "Una cabecera de 12 bytes. Eso es todo",
    "上面那堆十六进制就是你这张图的真实头 12 字节。没有 EXIF，没有元数据块——格式小到可以在一屏里讲完。":
      "El hexadecimal de arriba es la cabecera real de 12 bytes de tu imagen. Sin EXIF y sin bloques de metadatos: el formato cabe en una pantalla.",
    "测出来的，不是吹出来的": "Medido, no prometido",
    "四张 512×512 实拍，每张六个质量档，全部按":
      "Cuatro fotos reales de 512×512, seis niveles de calidad cada una, todo comparado al mismo",
    "同 PSNR": "PSNR",
    "比较（同一个": " (comparar al mismo",
    "比是不公平的）。": "no es justo).",
    "交流零区": "Zona muerta AC",
    "DC 二维预测": "Predicción DC 2D",
    "last 上下文": "contexto last",
    "频带上下文": "contexto de banda",
    "逐瓦片 AQ": "AQ por baldosa",
    "图片": "Imagen",
    "v1 字节": "bytes v1",
    "v2 字节": "bytes v2",
    "变化": "Cambio",
    "合计（几何平均）": "Total (media geométrica)",
    "主机实测：2048×1152 · q85 · 4:2:0 → 397 KB / 33.63 dB / 编码 ~115 ms、解码 ~130 ms（单核）。手机上会慢一些，安卓端因此把单张上限压到 8 MP。":
      "Medido en un equipo de escritorio: 2048×1152 · q85 · 4:2:0 a 397 KB / 33.63 dB / codificación ~115 ms y decodificación ~130 ms en un núcleo. Los teléfonos son más lentos, por eso Android limita cada foto a 8 MP.",
    "手机上：相机直接写 .mtc": "En el móvil: la cámara escribe .mtc directamente",
    "整条路径不产生一张 JPEG。Camera2 的 YUV 帧进 Rust，出来的文件就已经是 MTC。":
      "No se genera ni un JPEG por el camino. Camera2 entrega tramas YUV, Rust las recibe y lo que sale ya es MTC.",
    "三个界面": "Tres pantallas",
    "相册": "Álbum",
    "：网格、缩略图缓存、长按删除、从文件导入":
      ": cuadrícula, miniaturas en caché, mantener pulsado para borrar, importar desde un archivo",
    "相机": "Cámara",
    "：前后切换、色彩范围开关、实时显示角度":
      ": cambio frontal/trasera, rango de color, ángulo en vivo",
    "查看器": "Visor",
    "：双指缩放、双击回中、导出 PNG 到系统相册":
      ": pellizcar para ampliar, doble toque para centrar, exportar PNG al álbum",
    "省内存的几件事": "Tres cosas que ahorran memoria",
    "解码按目标尺寸直接出图（12 MP 峰值 92.8 MB → 400 px 时 64.9 MB），缩略图一次只解一张并落盘缓存，8 MP 上限，":
      "La decodificación va directa al tamaño destino (12 MP llega a 92.8 MB, 64.9 MB si el destino son 400 px), las miniaturas se decodifican de una en una y se guardan en caché, y cada foto se limita a 8 MP,",
    "安装包 783 KB，含 arm64 与 armeabi-v7a，无 Gradle、无 Maven，一条 PowerShell 脚本出包。":
      "El APK pesa 783 KB con arm64 y armeabi-v7a, y se compila con un script de PowerShell: sin Gradle ni Maven.",
    "拿走它": "Llévatelo",
    "页面会先认一下你用的系统，把合适的那个直接递给你。":
      "La página detecta tu sistema y te entrega la versión que corresponde.",
    "全部版本": "Todas las versiones",
    "正在识别系统…": "Detectando tu sistema…",
    "源代码": "Código",
    "格式规范": "Especificación",
    "反馈 / 建议：": "Comentarios: ",
    "页面上的编码器是 WebAssembly，和命令行走同一份代码。":
      "El codificador de esta página es WebAssembly, el mismo código que la línea de comandos.",
    "下载安装程序": "Descargar el instalador",
    "下载 Windows 安装程序": "Descargar para Windows",
    "检测到": "Detectado",
    "：直接下载": ": descargar directamente",
    "（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      " (2.8 MB). Doble clic para instalar; incluye la línea de comandos, el visor y la asociación .mtc.",
    "检测到 <b>Windows</b>：直接下载 <b>MtcSetup.exe</b>（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      "<b>Windows</b> detectado: descarga <b>MtcSetup.exe</b> (2.8 MB) y ejecútalo. Incluye la línea de comandos, el visor y la asociación .mtc.",
    "下载 Android 相机": "Descargar la cámara para Android",
    "下载 APK": "Descargar el APK",
    "检测到 <b>Android</b>：直接下载 <b>mtc-camera.apk</b>（0.8 MB），相机直出 .mtc，另有相册与查看器。":
      "<b>Android</b> detectado: descarga <b>mtc-camera.apk</b> (0.8 MB). La cámara escribe .mtc directamente, y hay álbum y visor.",
    "在 GitHub 上选择版本": "Elegir versión en GitHub",
    "去 GitHub": "Ir a GitHub",
    "检测到 <b>macOS</b>：这个项目目前只发布 Windows 和 Android 版本，所以带你去 GitHub 发布页。":
      "<b>macOS</b> detectado: este proyecto solo publica Windows y Android, así que aquí está la página de versiones.",
    "检测到 <b>iPhone / iPad</b>：没有 iOS 版本，已在 GitHub 发布页。":
      "<b>iPhone / iPad</b> detectado: no hay versión para iOS; la página de versiones es la entrada.",
    "检测到 <b>Linux</b>：没有打包好的 Linux 版本，但可以从源码构建（见发布页）。":
      "<b>Linux</b> detectado: no hay paquete, pero compila desde el código fuente (ver la página de versiones).",
    "没认出这个系统，带你去 GitHub 发布页自己挑。":
      "No he reconocido tu sistema, así que aquí está la página de versiones.",
    "试试看": "Pruébalo",
    "拖进来": "Suéltala",
    "点这里选择文件": "Elegir un archivo",
    "无损模式不采样色度，也不量化。": "El modo sin pérdida no submuestrea el croma ni cuantiza.",
    "画质": "Calidad",
    "体积": "Tamaño",
    "更快": "Más rápido",
    "更小": "Más pequeño",
    "原汁原味": "Tal como se tomó",
    "载入样图…": "Cargando la muestra…",
    "出错了：": "Error: ",
    "样图加载失败：": "No se pudo cargar la muestra: ",
    "编解码器加载失败：": "No se pudo cargar el códec: ",
    "样图没能在 3 秒内加载，用一张本地生成的图代替（编码器是真的）":
      "La muestra no llegó en 3 segundos: uso una imagen dibujada aquí (el codificador es real)",
    "自检图": "Imagen de prueba",
    "样图": "Muestra"
  };

  var FR = {
    "MTC · 自己写的图片编解码器": "MTC · un codec d'image écrit de zéro",
    "试一下": "Essayer",
    "原理": "Fonctionnement",
    "数据": "Chiffres",
    "安卓": "Android",
    "下载": "Télécharger",
    "Rust · 自研格式 · 无第三方编解码库": "Rust · format maison · sans codec tiers",
    "把照片压小": "Des photos plus petites,",
    "用的还是": "avec un codec",
    "自己写的": " maison",
    "编解码器": "",
    "8×8 DCT、零区量化、上下文算术编码、无损与精确 alpha。": "DCT 8×8, quantification à zone morte, codage arithmétique contextuel, sans perte et alpha exact.",
    "这个页面里的每一个字节都是你的浏览器现场算出来的":
      "Chaque octet de cette page est calculé par votre navigateur, maintenant",
    "——同一份 Rust 代码， 编译成 WebAssembly，拖一张图进来就能看到。":
      " — le même code Rust, compilé en WebAssembly. Déposez une image et regardez.",
    "立刻压一张图": "Compresser une image",
    "看源代码": "Voir le code",
    "同画质体积更小": "plus petites à qualité égale",
    "2048×1152 · q85 的体积": "taille à 2048×1152 · q85",
    "第三方编解码依赖": "codecs tiers",
    "在浏览器里压一张": "Compressez-en une ici",
    "图片不出本机。编码、解码、PSNR 全在这里跑完。":
      "L'image ne quitte pas cette machine. Encodage, décodage et PSNR se font ici.",
    "原图": "Originale",
    "MTC 解码": "Décodé par MTC",
    "把图片拖进来": "Déposez une image",
    "PNG / JPG / WebP，或者一个": "PNG / JPG / WebP, ou",
    "选文件": "choisir un fichier",
    "用样图试试": "Essayer un exemple",
    "还没算过——数据会出现在右边。": "Rien de calculé pour l'instant - les chiffres arrivent à droite.",
    "质量": "Qualité",
    "无损": "Sans perte",
    "自适应量化": "Quantification adaptative",
    "(实测更费码率)": "(coûte des octets en pratique)",
    "原始像素": "Pixels d'origine",
    "每像素": "Par pixel",
    "编码": "Encodage",
    "解码": "Décodage",
    "一条真实的率失真曲线：横轴 PSNR，纵轴每像素字节，越靠左下越好。":
      "Une vraie courbe débit/distorsion : PSNR en abscisse, octets par pixel en ordonnée. En bas à gauche, c'est mieux.",
    "下载 .mtc": "Télécharger le .mtc",
    "导出 PNG": "Exporter en PNG",
    "等待第一张图片…": "En attente de la première image…",
    "它到底怎么压的": "Ce qu'il fait vraiment",
    "一条主线，五步，每一步都可以单独解释给你听。":
      "Un pipeline, cinq étapes, et chacune s'explique séparément.",
    "YCoCg 换色": "Couleur YCoCg",
    "可逆整数变换，把亮度与两个色差分开放。色度再用":
      "Une transformée entière réversible qui sépare la luminance de deux composantes de chrominance. La chrominance reçoit ensuite",
    "的步长多压一点。": "un pas plus grossier.",
    "正交基，直流系数正好等于块均值 ×8，所以量化可以按像素级误差直接定步长。":
      "Une base orthogonale dont le coefficient DC vaut exactement huit fois la moyenne du bloc : le pas de quantification se déduit donc directement d'un budget d'erreur au pixel.",
    "零区量化": "Quantification à zone morte",
    "交流系数用": "Les coefficients AC utilisent une",
    "个步长的零区：小的直接归零，省下的字节比丢掉的精度多得多。":
      "zone morte d'un pas : les petits passent à zéro, et les octets gagnés valent bien plus que la précision perdue.",
    "上下文算术编码": "Codage arithmétique contextuel",
    "11 位自适应概率、按频带分模型，": "probabilités adaptatives sur 11 bits, un modèle par bande,",
    "还用前一块的稀疏程度做上下文。": "et la densité du bloc précédent comme contexte.",
    "容器": "Conteneur",
    "12 字节头加若干数据段。同一份容器放有损、无损、alpha，版本号只决定系数模型的布局。":
      "Un en-tête de 12 octets et des segments. Le même conteneur transporte avec perte, sans perte et alpha ; la version ne décide que de la disposition des modèles.",
    "自适应量化，为什么默认关着": "Quantification adaptative, et pourquoi elle est coupée",
    "每个 16×16 瓦片一个倍率（0.70×–1.40×），编码器按瓦片活跃度选档、对前一片差分编码、向低频倾斜。":
      "Chaque tuile de 16×16 reçoit un multiplicateur (0,70×–1,40×). L'encodeur choisit le pas selon l'activité, le code en différence avec la précédente et penche vers les basses fréquences.",
    "固定": "À pas fixe",
    "下它确实更好看（约 +0.3 dB），但同 PSNR 下要多花":
      "l'image est vraiment meilleure (environ +0,3 dB) mais coûte",
    "字节。所以默认关，勾上就能在这个页面里自己复现。":
      "plus d'octets à PSNR égal. D'où le réglage par défaut ; cochez et reproduisez-le ici.",
    "12 字节的头，就这么点": "Un en-tête de 12 octets. C'est tout",
    "上面那堆十六进制就是你这张图的真实头 12 字节。没有 EXIF，没有元数据块——格式小到可以在一屏里讲完。":
      "L'hexadécimal ci-dessus est le vrai en-tête de 12 octets de votre image. Ni EXIF ni blocs de métadonnées : le format tient sur un écran.",
    "测出来的，不是吹出来的": "Mesuré, pas promis",
    "四张 512×512 实拍，每张六个质量档，全部按":
      "Quatre photos réelles en 512×512, six niveaux de qualité chacune, tout comparé au même",
    "同 PSNR": "PSNR",
    "比较（同一个": " (comparer au même",
    "比是不公平的）。": "n'est pas honnête).",
    "交流零区": "Zone morte AC",
    "DC 二维预测": "Prédiction DC 2D",
    "last 上下文": "contexte last",
    "频带上下文": "contexte de bande",
    "逐瓦片 AQ": "AQ par tuile",
    "图片": "Image",
    "v1 字节": "octets v1",
    "v2 字节": "octets v2",
    "变化": "Écart",
    "合计（几何平均）": "Total (moyenne géométrique)",
    "主机实测：2048×1152 · q85 · 4:2:0 → 397 KB / 33.63 dB / 编码 ~115 ms、解码 ~130 ms（单核）。手机上会慢一些，安卓端因此把单张上限压到 8 MP。":
      "Mesuré sur un poste fixe : 2048×1152 · q85 · 4:2:0 vers 397 Ko / 33,63 dB / encodage ~115 ms et décodage ~130 ms sur un cœur. Les téléphones sont plus lents, d'où la limite de 8 MP par photo côté Android.",
    "手机上：相机直接写 .mtc": "Sur téléphone : l'appareil écrit du .mtc directement",
    "整条路径不产生一张 JPEG。Camera2 的 YUV 帧进 Rust，出来的文件就已经是 MTC。":
      "Aucun JPEG n'est produit en chemin. Camera2 fournit des trames YUV, Rust les prend, et ce qui sort est déjà du MTC.",
    "三个界面": "Trois écrans",
    "相册": "Album",
    "：网格、缩略图缓存、长按删除、从文件导入":
      " : grille, vignettes en cache, appui long pour supprimer, import depuis un fichier",
    "相机": "Appareil",
    "：前后切换、色彩范围开关、实时显示角度":
      " : bascule avant/arrière, plage de couleur, angle en direct",
    "查看器": "Visionneuse",
    "：双指缩放、双击回中、导出 PNG 到系统相册":
      " : pincer pour zoomer, double tap pour recentrer, export PNG vers l'album",
    "省内存的几件事": "Trois choses qui économisent la mémoire",
    "解码按目标尺寸直接出图（12 MP 峰值 92.8 MB → 400 px 时 64.9 MB），缩略图一次只解一张并落盘缓存，8 MP 上限，":
      "Le décodage va droit à la taille visée (12 MP culminent à 92,8 Mo, 64,9 Mo pour une cible de 400 px), les vignettes se décodent une par une et finissent en cache, une photo est limitée à 8 MP,",
    "安装包 783 KB，含 arm64 与 armeabi-v7a，无 Gradle、无 Maven，一条 PowerShell 脚本出包。":
      "L'APK pèse 783 Ko avec arm64 et armeabi-v7a, et se construit avec un seul script PowerShell : ni Gradle ni Maven.",
    "拿走它": "Emporte-le",
    "页面会先认一下你用的系统，把合适的那个直接递给你。":
      "La page reconnaît votre système et vous tend la bonne version.",
    "全部版本": "Toutes les versions",
    "正在识别系统…": "Détection du système…",
    "源代码": "Code source",
    "格式规范": "Spécification",
    "反馈 / 建议：": "Retours : ",
    "页面上的编码器是 WebAssembly，和命令行走同一份代码。":
      "L'encodeur de cette page est du WebAssembly, le même code que la ligne de commande.",
    "下载安装程序": "Télécharger l'installateur",
    "下载 Windows 安装程序": "Télécharger pour Windows",
    "检测到": "Détecté",
    "：直接下载": " : télécharger directement",
    "（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      " (2,8 Mo). Double-clic pour installer ; il apporte la ligne de commande, la visionneuse et l'association .mtc.",
    "检测到 <b>Windows</b>：直接下载 <b>MtcSetup.exe</b>（2.8 MB），双击安装，含命令行、查看器和 .mtc 关联。":
      "<b>Windows</b> détecté : téléchargez <b>MtcSetup.exe</b> (2,8 Mo) et lancez-le. Il apporte la ligne de commande, la visionneuse et l'association .mtc.",
    "下载 Android 相机": "Télécharger l'appareil Android",
    "下载 APK": "Télécharger l'APK",
    "检测到 <b>Android</b>：直接下载 <b>mtc-camera.apk</b>（0.8 MB），相机直出 .mtc，另有相册与查看器。":
      "<b>Android</b> détecté : téléchargez <b>mtc-camera.apk</b> (0,8 Mo). L'appareil écrit du .mtc directement, avec album et visionneuse.",
    "在 GitHub 上选择版本": "Choisir une version sur GitHub",
    "去 GitHub": "Aller sur GitHub",
    "检测到 <b>macOS</b>：这个项目目前只发布 Windows 和 Android 版本，所以带你去 GitHub 发布页。":
      "<b>macOS</b> détecté : ce projet ne publie que Windows et Android, voici donc la page des versions.",
    "检测到 <b>iPhone / iPad</b>：没有 iOS 版本，已在 GitHub 发布页。":
      "<b>iPhone / iPad</b> détecté : pas de version iOS ; la page des versions est la voie d'entrée.",
    "检测到 <b>Linux</b>：没有打包好的 Linux 版本，但可以从源码构建（见发布页）。":
      "<b>Linux</b> détecté : pas de paquet, mais le projet se compile depuis les sources (voir la page des versions).",
    "没认出这个系统，带你去 GitHub 发布页自己挑。":
      "Impossible d'identifier votre système, voici la page des versions.",
    "试试看": "Essayez",
    "拖进来": "Déposez",
    "点这里选择文件": "Choisir un fichier",
    "无损模式不采样色度，也不量化。": "Le mode sans perte ne sous-échantillonne pas la chrominance et ne quantifie pas.",
    "画质": "Qualité",
    "体积": "Taille",
    "更快": "Plus rapide",
    "更小": "Plus petit",
    "原汁原味": "Comme à la prise de vue",
    "载入样图…": "Chargement de l'exemple…",
    "出错了：": "Erreur : ",
    "样图加载失败：": "Échec du chargement de l'exemple : ",
    "编解码器加载失败：": "Échec du chargement du codec : ",
    "样图没能在 3 秒内加载，用一张本地生成的图代替（编码器是真的）":
      "L'exemple n'est pas arrivé en 3 secondes : une image dessinée localement le remplace (l'encodeur est réel)",
    "自检图": "Image de test",
    "样图": "Exemple"
  };

  var DICTS = { zh: null, en: EN, ja: JA, ko: KO, ru: RU, es: ES, fr: FR };
  var NAMES = {
    zh: "中文",
    en: "English",
    ja: "日本語",
    ko: "한국어",
    ru: "Русский",
    es: "Español",
    fr: "Français"
  };
  var SHORT = { zh: "中", en: "EN", ja: "日", ko: "한", ru: "RU", es: "ES", fr: "FR" };

  /** Original text of every node and attribute touched, keyed by node/attribute. */
  var originals = null;

  function lookup(lang, text) {
    // Chinese is the source: the page is written in it, so there is nothing to do.
    if (lang === "zh") {
      return text;
    }
    var dict = DICTS[lang];
    if (dict && Object.prototype.hasOwnProperty.call(dict, text)) {
      return dict[text];
    }
    if (Object.prototype.hasOwnProperty.call(EN, text)) {
      return EN[text];
    }
    return text;
  }

  function walk(lang) {
    var root = document.body;
    if (!root) {
      return;
    }
    if (!originals) {
      originals = { text: new Map(), attr: new Map() };
    }
    var walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
      acceptNode: function (node) {
        var parent = node.parentNode;
        if (!parent) {
          return NodeFilter.FILTER_REJECT;
        }
        var tag = parent.nodeName;
        if (tag === "SCRIPT" || tag === "STYLE") {
          return NodeFilter.FILTER_REJECT;
        }
        return node.nodeValue && node.nodeValue.trim() ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_REJECT;
      }
    });
    var node;
    while ((node = walker.nextNode())) {
      if (!originals.text.has(node)) {
        originals.text.set(node, node.nodeValue);
      }
      var source = originals.text.get(node);
      var trimmed = source.trim();
      if (!Object.prototype.hasOwnProperty.call(originals.text, "x")) {
        // first pass over this node: keep the raw source for later languages
      }
      // Whitespace inside a paragraph is layout, not meaning: collapse it so a key does
      // not have to reproduce the exact indentation of the HTML.
      var replacement = lookup(lang, trimmed.replace(/\s+/g, " "));
      if (replacement !== trimmed) {
        var lead = source.match(/^\s*/)[0];
        var trail = source.match(/\s*$/)[0];
        node.nodeValue = lead + replacement + trail;
      } else {
        node.nodeValue = source;
      }
    }

    var attrs = ["placeholder", "title", "aria-label", "alt"];
    var elements = root.querySelectorAll("*");
    for (var i = 0; i < elements.length; i++) {
      for (var a = 0; a < attrs.length; a++) {
        var name = attrs[a];
        var value = elements[i].getAttribute(name);
        if (!value || !/\u4e00-\u9fff/.test(value)) {
          continue;
        }
        var key = elements[i].tagName + ":" + name;
        var store = originals.attr.get(elements[i]) || {};
        if (store[name] === undefined) {
          store[name] = value;
          originals.attr.set(elements[i], store);
        }
        var from = store[name].trim().replace(/\s+/g, " ");
        elements[i].setAttribute(name, lookup(lang, from));
      }
    }
  }

  function apply(lang) {
    if (!DICTS.hasOwnProperty(lang)) {
      lang = "en";
    }
    walk(lang);
    var originalTitle = document.documentElement.getAttribute("data-title");
    if (!originalTitle) {
      originalTitle = document.title;
      document.documentElement.setAttribute("data-title", originalTitle);
    }
    document.title = lookup(lang, originalTitle);
    document.documentElement.lang = lang === "zh" ? "zh-CN" : lang;
    try {
      localStorage.setItem("mtc.lang", lang);
    } catch (e) {
      /* private mode */
    }
    var button = document.getElementById("lang");
    if (button) {
      button.textContent = SHORT[lang] || lang.toUpperCase();
      button.setAttribute("aria-label", "Language / " + NAMES[lang]);
      button.title = NAMES[lang];
    }
  }

  function initial() {
    var params = new URLSearchParams(location.search);
    var forced = params.get("lang");
    if (forced && DICTS.hasOwnProperty(forced)) {
      return forced;
    }
    try {
      var saved = localStorage.getItem("mtc.lang");
      if (saved && DICTS.hasOwnProperty(saved)) {
        return saved;
      }
    } catch (e) {
      /* private mode */
    }
    var nav = (navigator.language || "en").toLowerCase();
    if (nav.indexOf("zh") === 0) {
      return "zh";
    }
    var short = nav.slice(0, 2);
    return DICTS.hasOwnProperty(short) ? short : "en";
  }

  var ORDER = ["zh", "en", "ja", "ko", "ru", "es", "fr"];
  var current = initial();

  function cycle() {
    var at = ORDER.indexOf(current);
    current = ORDER[(at + 1) % ORDER.length];
    apply(current);
  }

  /** Translate one string for code that builds text at runtime. */
  function str(text) {
    return lookup(current, text);
  }
  window.MTC_LANG = {
    str: str,
    apply: apply,
    cycle: cycle,
    current: function () {
      return current;
    }
  };

  apply(current);
})();
