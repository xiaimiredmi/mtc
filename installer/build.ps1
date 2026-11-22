# Build the MTC Windows installer.
#
#   powershell -ExecutionPolicy Bypass -File installer\build.ps1
#   powershell -ExecutionPolicy Bypass -File installer\build.ps1 -Check   # + smoke test
#
# No Inno Setup, no WiX, no NSIS: the C# compiler that ships with .NET Framework
# (csc.exe) builds a single .exe and the binaries ride along as an embedded zip.
# Nothing has to be installed to build this, and nothing has to be installed to
# run it.
param(
    [switch]$Check,
    [string]$Version = ''
)

$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot
$root = Split-Path $here -Parent

if (-not $Version) {
    $line = Select-String -Path (Join-Path $root 'rust\Cargo.toml') -Pattern '^version = "(.*)"' |
        Select-Object -First 1
    if ($line) { $Version = $line.Matches[0].Groups[1].Value } else { $Version = '0.2.0' }
}
Write-Host "== MTC installer $Version"

$bin = Join-Path $root 'bin'
$exes = @('mtcodec.exe', 'mtbench.exe', 'mtview.exe')
foreach ($exe in $exes) {
    $path = Join-Path $bin $exe
    if (-not (Test-Path $path)) {
        throw "missing $path - build the Rust binaries first (cargo build --release --features viewer; copy to bin\)"
    }
}

# ---- payload -------------------------------------------------------------------
$stage = Join-Path $env:TEMP ('mtc-payload-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $stage -Force | Out-Null
try {
    Copy-Item (Join-Path $bin '*.exe') $stage -Force
    $icon = Join-Path $root 'deploy\mtc.ico'
    if (Test-Path $icon) { Copy-Item $icon (Join-Path $stage 'mtc.ico') -Force }
    $license = Join-Path $root 'LICENSE'
    if (Test-Path $license) { Copy-Item $license (Join-Path $stage 'LICENSE') -Force }

    # A short README inside the install folder, because the first thing anyone does
    # after installing a command line tool is wonder what the commands are.
    $readme = @"
MTC $Version - image codec, viewer and tools

  mtview image.mtc            open a picture (double-clicking a .mtc uses this too)
  mtcodec encode in.png out.mtc -q 85
  mtcodec decode in.mtc out.png
  mtcodec info image.mtc
  mtbench                      rate/distortion harness

The full documentation is in the repository: https://github.com/xiaimiredmi/mtc

Uninstall: Settings > Apps, or run uninstall.exe in this folder.
"@
    Set-Content -Path (Join-Path $stage 'README.txt') -Value $readme -Encoding UTF8

    # Compress each file on its own: DeflateStream lives in System.dll, so the
    # installer needs no reference to System.IO.Compression.dll, which csc cannot
    # always resolve by simple name.
    $packed = Join-Path $here 'payload'
    if (Test-Path $packed) { Remove-Item $packed -Recurse -Force }
    New-Item -ItemType Directory -Path $packed -Force | Out-Null
    $total = 0
    foreach ($file in (Get-ChildItem $stage -File)) {
        $out = Join-Path $packed ($file.Name + '.deflate')
        $bytes = [IO.File]::ReadAllBytes($file.FullName)
        $memory = New-Object IO.MemoryStream
        $deflate = New-Object IO.Compression.DeflateStream($memory, [IO.Compression.CompressionMode]::Compress, $true)
        $deflate.Write($bytes, 0, $bytes.Length)
        $deflate.Dispose()
        [IO.File]::WriteAllBytes($out, $memory.ToArray())
        $memory.Dispose()
        $total += (Get-Item $out).Length
    }
    Write-Host ("   payload {0:N0} KB ({1} files, was {2:N0} KB)" -f ($total / 1KB),
        (Get-ChildItem $stage -File).Count, ((Get-ChildItem $stage -File | Measure-Object Length -Sum).Sum / 1KB))
} finally {
    Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
}

# ---- compile -------------------------------------------------------------------
$csc = Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319\csc.exe'
if (-not (Test-Path $csc)) { $csc = Join-Path $env:WINDIR 'Microsoft.NET\Framework\v4.0.30319\csc.exe' }
if (-not (Test-Path $csc)) { throw 'csc.exe not found; .NET Framework 4 is required to build' }

$source = Join-Path $here 'MtcSetup.cs'
# csc reads a source file as ANSI unless it has a BOM; the file has Chinese strings,
# so the encoding is stated explicitly rather than hoped for.
$utf8Bom = New-Object Text.UTF8Encoding($true)
$text = [IO.File]::ReadAllText($source)
[IO.File]::WriteAllText($source, $text, $utf8Bom)

$out = Join-Path $here 'MtcSetup.exe'
if (Test-Path $out) { Remove-Item $out -Force }
$icon = Join-Path $root 'deploy\mtc.ico'
$args = @(
    '/nologo', '/target:winexe', '/platform:anycpu', '/optimize+',
    '/codepage:65001',
    "/out:$out"
)
foreach ($file in (Get-ChildItem $packed -File)) {
    $name = $file.Name -replace '\.deflate$', ''
    $args += "/resource:$($file.FullName),payload.$name"
}
if (Test-Path $icon) { $args += "/win32icon:$icon" }
$args += @('/r:System.dll', '/r:System.Drawing.dll', '/r:System.Windows.Forms.dll')
$args += $source

$previous = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
    & $csc @args 2>&1 | Tee-Object -FilePath (Join-Path $here 'csc.log') | Select-Object -Last 12
    $code = $LASTEXITCODE
} finally {
    $ErrorActionPreference = $previous
}
if ($code -ne 0) { throw "csc failed, see installer\csc.log" }

$installer = Get-Item $out
Write-Host ("built {0} ({1:N1} MB)" -f $installer.FullName, ($installer.Length / 1MB))

if ($Check) {
    Write-Host ''
    Write-Host '== smoke test: extract only, nothing registered'
    $probe = Join-Path $env:TEMP ('mtc-extract-' + [guid]::NewGuid().ToString('N'))
    try {
        & $out /extract $probe | Out-Null
        $wanted = @('mtcodec.exe', 'mtbench.exe', 'mtview.exe', 'README.txt')
        $bad = 0
        foreach ($file in $wanted) {
            $got = Join-Path $probe $file
            if (-not (Test-Path $got)) { Write-Host "  missing $file"; $bad++; continue }
            $source = Join-Path $bin $file
            if (Test-Path $source) {
                $same = (Get-FileHash $got -Algorithm SHA256).Hash -eq (Get-FileHash $source -Algorithm SHA256).Hash
                Write-Host ("  {0,-14} {1,10:N0} bytes  hash match: {2}" -f $file, (Get-Item $got).Length, $same)
                if (-not $same) { $bad++ }
            } else {
                Write-Host ("  {0,-14} {1,10:N0} bytes" -f $file, (Get-Item $got).Length)
            }
        }
        if ($bad -gt 0) { throw "$bad file(s) did not survive the round trip" }
        Write-Host '  payload round trip: ok'
    } finally {
        Remove-Item $probe -Recurse -Force -ErrorAction SilentlyContinue
    }
}
