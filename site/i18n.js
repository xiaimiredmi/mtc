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
    "立刻压一张图": "画像を圧縮する",
    "看源代码": "ソースを見る",
    "同画质体积更小": "同じ画質でより小さい",
    "第三方编解码依赖": "外部コーデック依存",
    "在浏览器里压一张": "ブラウザで圧縮",
    "原图": "元画像",
    "MTC 解码": "MTC デコード",
    "把图片拖进来": "画像をドロップ",
    "选文件": "ファイルを選ぶ",
    "用样图试试": "サンプルで試す",
    "质量": "画質",
    "无损": "ロスレス",
    "原始像素": "元のピクセル",
    "每像素": "ピクセルあたり",
    "编码": "エンコード",
    "解码": "デコード",
    "下载 .mtc": ".mtc をダウンロード",
    "导出 PNG": "PNG を書き出す",
    "它到底怎么压的": "実際の仕組み",
    "YCoCg 换色": "YCoCg 色変換",
    "零区量化": "デッドゾーン量子化",
    "上下文算术编码": "文脈算術符号化",
    "容器": "コンテナ",
    "12 字节的头，就这么点": "12 バイトのヘッダ、これだけ",
    "测出来的，不是吹出来的": "測定値であり、誇張ではない",
    "交流零区": "AC デッドゾーン",
    "DC 二维预测": "DC 二次元予測",
    "last 上下文": "last 文脈",
    "频带上下文": "帯域文脈",
    "逐瓦片 AQ": "タイル単位 AQ",
    "图片": "画像",
    "变化": "変化",
    "合计（几何平均）": "合計（幾何平均）",
    "手机上：相机直接写 .mtc": "スマホ：カメラが直接 .mtc を書く",
    "三个界面": "3 つの画面",
    "相册": "アルバム",
    "相机": "カメラ",
    "查看器": "ビューア",
    "省内存的几件事": "メモリを節約する工夫",
    "拿走它": "持っていく",
    "全部版本": "すべてのリリース",
    "正在识别系统…": "システムを判定中…",
    "源代码": "ソースコード",
    "格式规范": "フォーマット仕様",
    "反馈 / 建议：": "ご意見："
  };

  var KO = {
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
    "立刻压一张图": "이미지 압축하기",
    "看源代码": "소스 보기",
    "同画质体积更小": "같은 화질에 더 작게",
    "第三方编解码依赖": "외부 코덱 의존",
    "在浏览器里压一张": "브라우저에서 압축",
    "原图": "원본",
    "MTC 解码": "MTC 디코드",
    "把图片拖进来": "이미지를 끌어다 놓기",
    "选文件": "파일 선택",
    "用样图试试": "샘플로 시험",
    "质量": "화질",
    "无损": "무손실",
    "原始像素": "원본 픽셀",
    "每像素": "픽셀당",
    "编码": "인코드",
    "解码": "디코드",
    "下载 .mtc": ".mtc 내려받기",
    "导出 PNG": "PNG 내보내기",
    "它到底怎么压的": "실제로 하는 일",
    "容器": "컨테이너",
    "图片": "이미지",
    "变化": "변화",
    "合计（几何平均）": "합계(기하평균)",
    "三个界面": "세 개의 화면",
    "相册": "앨범",
    "相机": "카메라",
    "查看器": "뷰어",
    "拿走它": "가져가기",
    "全部版本": "모든 릴리스",
    "正在识别系统…": "시스템 확인 중…",
    "源代码": "소스 코드",
    "格式规范": "포맷 규격",
    "反馈 / 建议：": "의견："
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
    "试一下": "Probar",
    "原理": "Cómo funciona",
    "数据": "Cifras",
    "安卓": "Android",
    "下载": "Descargar",
    "Rust · 自研格式 · 无第三方编解码库": "Rust · formato propio · sin códecs de terceros",
    "把照片压小": "Fotos más pequeñas,",
    "用的还是": "con un códec",
    "自己写的": "propio",
    "编解码器": "",
    "立刻压一张图": "Comprimir una imagen",
    "看源代码": "Ver el código",
    "同画质体积更小": "más pequeñas con la misma calidad",
    "第三方编解码依赖": "códecs de terceros",
    "在浏览器里压一张": "Comprime una en el navegador",
    "原图": "Original",
    "MTC 解码": "Decodificado con MTC",
    "把图片拖进来": "Arrastra una imagen",
    "选文件": "Elegir archivo",
    "用样图试试": "Probar una muestra",
    "质量": "Calidad",
    "无损": "Sin pérdida",
    "原始像素": "Píxeles originales",
    "每像素": "Por píxel",
    "编码": "Codificar",
    "解码": "Decodificar",
    "下载 .mtc": "Descargar .mtc",
    "导出 PNG": "Exportar PNG",
    "它到底怎么压的": "Qué hace en realidad",
    "容器": "Contenedor",
    "图片": "Imagen",
    "变化": "Cambio",
    "合计（几何平均）": "Total (media geométrica)",
    "三个界面": "Tres pantallas",
    "相册": "Álbum",
    "相机": "Cámara",
    "查看器": "Visor",
    "拿走它": "Llévatelo",
    "全部版本": "Todas las versiones",
    "正在识别系统…": "Detectando tu sistema…",
    "源代码": "Código",
    "格式规范": "Especificación",
    "反馈 / 建议：": "Comentarios: "
  };

  var FR = {
    "试一下": "Essayer",
    "原理": "Fonctionnement",
    "数据": "Chiffres",
    "安卓": "Android",
    "下载": "Télécharger",
    "Rust · 自研格式 · 无第三方编解码库": "Rust · format maison · sans codec tiers",
    "把照片压小": "Des photos plus petites,",
    "用的还是": "avec un codec",
    "自己写的": "maison",
    "编解码器": "",
    "立刻压一张图": "Compresser une image",
    "看源代码": "Voir le code",
    "同画质体积更小": "plus petites à qualité égale",
    "第三方编解码依赖": "codecs tiers",
    "在浏览器里压一张": "Compressez-en une ici",
    "原图": "Originale",
    "MTC 解码": "Décodé par MTC",
    "把图片拖进来": "Déposez une image",
    "选文件": "Choisir un fichier",
    "用样图试试": "Essayer un exemple",
    "质量": "Qualité",
    "无损": "Sans perte",
    "原始像素": "Pixels d'origine",
    "每像素": "Par pixel",
    "编码": "Encodage",
    "解码": "Décodage",
    "下载 .mtc": "Télécharger le .mtc",
    "导出 PNG": "Exporter en PNG",
    "它到底怎么压的": "Ce qu'il fait vraiment",
    "容器": "Conteneur",
    "图片": "Image",
    "变化": "Écart",
    "合计（几何平均）": "Total (moyenne géométrique)",
    "三个界面": "Trois écrans",
    "相册": "Album",
    "相机": "Appareil",
    "查看器": "Visionneuse",
    "拿走它": "Emporte-le",
    "全部版本": "Toutes les versions",
    "正在识别系统…": "Détection du système…",
    "源代码": "Code source",
    "格式规范": "Spécification",
    "反馈 / 建议：": "Retours : "
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
