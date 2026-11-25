# Build the C library: capi\build\mtc.dll + mtc.h
#
#   powershell -ExecutionPolicy Bypass -File capi\build.ps1
#   powershell -ExecutionPolicy Bypass -File capi\build.ps1 -Check    # + load and run it
param(
    [switch]$Check
)

$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot

Write-Host '== cargo build --release (cdylib)'
Push-Location $here
try {
    $previous = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        & cargo build --release *> (Join-Path $env:TEMP 'mtc-capi-build.log')
        $code = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previous
    }
    if ($code -ne 0) {
        Get-Content (Join-Path $env:TEMP 'mtc-capi-build.log') | Select-Object -Last 25
        throw 'cargo build failed'
    }
} finally {
    Pop-Location
}

$dll = Join-Path $here 'target\release\mtc.dll'
if (-not (Test-Path $dll)) { throw "missing $dll" }

$out = Join-Path $here 'build'
New-Item -ItemType Directory -Path $out -Force | Out-Null
Copy-Item $dll (Join-Path $out 'mtc.dll') -Force
Copy-Item (Join-Path $here 'include\mtc.h') (Join-Path $out 'mtc.h') -Force
Write-Host ("built {0} ({1:N0} KB) and mtc.h" -f (Join-Path $out 'mtc.dll'), ((Get-Item (Join-Path $out 'mtc.dll')).Length / 1KB))

if (-not $Check) { return }

Write-Host ''
Write-Host '== loading it and decoding a file'
$env:PATH = "$out;$env:PATH"
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class MtcNative {
    [StructLayout(LayoutKind.Sequential)]
    public struct Info { public int width, height, channels, mode, quality, version, has_alpha, has_aq; }
    [DllImport("mtc", CallingConvention = CallingConvention.Cdecl)]
    public static extern int mtc_info(byte[] data, int len, out Info info);
    [DllImport("mtc", CallingConvention = CallingConvention.Cdecl)]
    public static extern int mtc_decode(byte[] data, int len, out IntPtr rgba, out int w, out int h);
    [DllImport("mtc", CallingConvention = CallingConvention.Cdecl)]
    public static extern int mtc_decode_scaled(byte[] data, int len, int maxSide, out IntPtr rgba, out int w, out int h);
    [DllImport("mtc", CallingConvention = CallingConvention.Cdecl)]
    public static extern void mtc_free(IntPtr p);
    [DllImport("mtc", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr mtc_last_error();
}
'@ -ReferencedAssemblies System.Runtime.InteropServices

# Encode something small with this very library, then read it back: a round trip that
# exercises both directions without needing a file from elsewhere.
$pixels = New-Object byte[] (64 * 48 * 4)
for ($i = 0; $i -lt 64 * 48; $i++) {
    $pixels[$i * 4 + 0] = ($i % 64) * 4
    $pixels[$i * 4 + 1] = [int]($i / 64) * 5
    $pixels[$i * 4 + 2] = 90
    $pixels[$i * 4 + 3] = 255
}
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class MtcEncode {
    [DllImport("mtc", CallingConvention = CallingConvention.Cdecl)]
    public static extern int mtc_encode(byte[] pixels, int w, int h, int channels, int quality, int lossless, out IntPtr output, out int outLen);
    [DllImport("mtc", CallingConvention = CallingConvention.Cdecl)]
    public static extern void mtc_free(IntPtr p);
}
'@ -ReferencedAssemblies System.Runtime.InteropServices

$ptr = [IntPtr]::Zero; $len = 0
$rc = [MtcEncode]::mtc_encode($pixels, 64, 48, 4, 85, 0, [ref]$ptr, [ref]$len)
if ($rc -ne 0) { throw ("encode failed: " + [Runtime.InteropServices.Marshal]::PtrToStringAnsi([MtcNative]::mtc_last_error())) }
$blob = New-Object byte[] $len
[Runtime.InteropServices.Marshal]::Copy($ptr, $blob, 0, $len)
[MtcEncode]::mtc_free($ptr)
Write-Host ("  encoded {0}x{1} -> {2} bytes" -f 64, 48, $len)

$info = New-Object MtcNative+Info
$rc = [MtcNative]::mtc_info($blob, $blob.Length, [ref]$info)
Write-Host ("  info rc={0} {1}x{2} mode={3} q={4} v={5}" -f $rc, $info.width, $info.height, $info.mode, $info.quality, $info.version)

$rgba = [IntPtr]::Zero; $w = 0; $h = 0
$rc = [MtcNative]::mtc_decode($blob, $blob.Length, [ref]$rgba, [ref]$w, [ref]$h)
if ($rc -ne 0) { throw ("decode failed: " + [Runtime.InteropServices.Marshal]::PtrToStringAnsi([MtcNative]::mtc_last_error())) }
$roundtrip = New-Object byte[] ($w * $h * 4)
[Runtime.InteropServices.Marshal]::Copy($rgba, $roundtrip, 0, $roundtrip.Length)
[MtcNative]::mtc_free($rgba)
$worst = 0
for ($i = 0; $i -lt 64 * 48 * 3; $i++) {
    $channel = $i % 4
    if ($channel -eq 3) { continue }
    $d = [Math]::Abs([int]$pixels[$i] - [int]$roundtrip[$i])
    if ($d -gt $worst) { $worst = $d }
}
Write-Host ("  round trip {0}x{1}, worst channel error {2}" -f $w, $h, $worst)
if ($w -ne 64 -or $h -ne 48) { throw 'decoded size does not match' }
if ($worst -gt 12) { throw "error too large for q85: $worst" }

$small = [IntPtr]::Zero; $sw = 0; $sh = 0
$rc = [MtcNative]::mtc_decode_scaled($blob, $blob.Length, 24, [ref]$small, [ref]$sw, [ref]$sh)
[MtcNative]::mtc_free($small)
Write-Host ("  max_side=24 -> {0}x{1} (rc={2})" -f $sw, $sh, $rc)
Write-Host 'all checks passed'
