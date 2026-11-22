# Check that the site is actually live, and that what is served is this repository.
#
#   powershell -ExecutionPolicy Bypass -File site\check-domain.ps1
#   powershell -ExecutionPolicy Bypass -File site\check-domain.ps1 -Base http://127.0.0.1:8765
#   # before DNS exists: talk to a Pages IP with the domain as the Host header
#   powershell -ExecutionPolicy Bypass -File site\check-domain.ps1 -Base http://185.199.108.153 -HostHeader mtc-xiaimiredmi.top -SkipDns
#
# Exit code 0 means every check passed.
#
# Kept ASCII-only on purpose: Windows PowerShell 5.1 reads a .ps1 as ANSI unless
# it has a BOM, and a BOM in turn upsets the first line.
param(
    [string]$Domain = 'mtc-xiaimiredmi.top',
    [string]$Base,
    [string]$HostHeader,
    [switch]$SkipDns
)

$ErrorActionPreference = 'Continue'
$failures = 0
function Pass($what, $detail = '') { Write-Host ("  ok    {0} {1}" -f $what, $detail) }
function Fail($what, $detail = '') {
    $script:failures++
    Write-Host ("  FAIL  {0} {1}" -f $what, $detail) -ForegroundColor Red
}
function Info($what, $detail = '') { Write-Host ("  ..    {0} {1}" -f $what, $detail) }

if (-not $Base) { $Base = "https://$Domain" }
Write-Host "checking $Base"
if ($HostHeader) { Write-Host "  with Host: $HostHeader" }

# ---- DNS --------------------------------------------------------------------
if (-not $HostHeader -and $Base.StartsWith('https://') -and -not $SkipDns) {
    Write-Host '== DNS'
    try {
        $answer = Resolve-DnsName $Domain -Type A -ErrorAction Stop |
            Where-Object { $_.IPAddress } | Select-Object -ExpandProperty IPAddress
        $expected = @('185.199.108.153', '185.199.109.153', '185.199.110.153', '185.199.111.153')
        $missing = @($expected | Where-Object { $answer -notcontains $_ })
        if ($missing.Count -eq 0) {
            Pass 'A records' ($answer -join ' ')
        } else {
            Fail 'A records' ("missing " + ($missing -join ', ') + "; got " + ($answer -join ' '))
        }
    } catch {
        Fail 'A records' $_.Exception.Message
    }
    try {
        $cname = Resolve-DnsName "www.$Domain" -Type CNAME -ErrorAction Stop |
            Where-Object { $_.NameHost } | Select-Object -First 1 -ExpandProperty NameHost
        if ($cname -like 'xiaimiredmi.github.io*') {
            Pass 'www CNAME' $cname
        } else {
            Fail 'www CNAME' "points at '$cname', expected xiaimiredmi.github.io"
        }
    } catch {
        Fail 'www CNAME' $_.Exception.Message
    }
}

# ---- fetch ----------------------------------------------------------------
# curl is used rather than Invoke-WebRequest because it can send a Host header
# that differs from the address, which is how a Pages site is reached before the
# domain resolves.
function Fetch([string]$path, [string]$outFile) {
    $args = @('-sS', '-L', '--max-time', '30', '-o', $outFile, '-w', '%{http_code}')
    if ($HostHeader) { $args += @('-H', "Host: $HostHeader") }
    $args += "$Base$path"
    $code = & curl.exe @args 2>$null
    return [int]$code
}

function Sha([string]$file) {
    if (-not (Test-Path $file)) { return $null }
    return (Get-FileHash -Path $file -Algorithm SHA256).Hash
}

$tmp = Join-Path $env:TEMP ("mtc-check-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp -Force | Out-Null

# ---- the page and its assets, compared byte for byte ------------------------
Write-Host '== page'
$localIndex = Join-Path $PSScriptRoot 'index.html'
$servedIndex = Join-Path $tmp 'index.html'
$code = Fetch '/' $servedIndex
if ($code -ne 200) {
    Fail 'GET /' "HTTP $code"
} else {
    $served = Sha $servedIndex
    $local = Sha $localIndex
    if ($served -eq $local) {
        Pass 'GET /' "HTTP 200, identical to site/index.html"
    } else {
        Fail 'GET /' "HTTP 200, but differs from site/index.html (deploy is stale, or the page was edited after publishing)"
    }
}

foreach ($asset in @('styles.css', 'codec.js', 'app.js')) {
    $file = Join-Path $tmp ($asset -replace '/', '_')
    $code = Fetch "/$asset" $file
    if ($code -ne 200) {
        Fail $asset "HTTP $code"
    } elseif ((Sha $file) -eq (Sha (Join-Path $PSScriptRoot $asset))) {
        Pass $asset 'identical'
    } else {
        Fail $asset 'served copy differs from the local file'
    }
}

# ---- the codec -----------------------------------------------------------------
Write-Host '== codec'
$wasm = Join-Path $tmp 'mtc.wasm'
$code = Fetch '/mtc.wasm' $wasm
if ($code -ne 200) {
    Fail 'GET /mtc.wasm' "HTTP $code"
} else {
    $bytes = [IO.File]::ReadAllBytes($wasm)
    $magic = ($bytes[0..3] | ForEach-Object { '{0:X2}' -f $_ }) -join ''
    $size = $bytes.Length
    $localSize = (Get-Item (Join-Path $PSScriptRoot 'mtc.wasm')).Length
    if ($magic -ne '0061736D') {
        Fail 'wasm magic' "$magic (expected 0061736D for \0asm)"
    } else {
        Pass 'wasm magic' $magic
        # The deploy workflow may rebuild the module on a different compiler, so
        # an exact size match is not required; a wide sanity band is.
        $ratio = $size / $localSize
        if ($ratio -lt 0.8 -or $ratio -gt 1.25) {
            Fail 'wasm size' "$size bytes vs $localSize locally - more than a rebuild should change"
        } elseif ($size -eq $localSize) {
            Pass 'wasm size' "$size bytes, identical to the local build"
        } else {
            Info 'wasm size' "$size bytes vs $localSize locally (rebuilt in CI, expected)"
        }
    }
}

# ---- other assets ---------------------------------------------------------------
Write-Host '== other assets'
foreach ($path in @('assets/sample.png', 'assets/legacy.mtc', '404.html', 'CNAME')) {
    $file = Join-Path $tmp ($path -replace '/', '_')
    $code = Fetch "/$path" $file
    if ($code -eq 200) { Pass $path 'HTTP 200' } else { Fail $path "HTTP $code" }
}

# ---- https ---------------------------------------------------------------------
if ($Base.StartsWith('https://')) {
    Write-Host '== tls'
    try {
        $head = Invoke-WebRequest $Base -Method Head -UseBasicParsing -TimeoutSec 25
        Pass 'https' ("HTTP " + $head.StatusCode)
    } catch {
        Fail 'https' $_.Exception.Message
    }
    # curl, not Invoke-WebRequest: with -MaximumRedirection 0 the .NET client throws
    # instead of handing back the 301, which made a working redirect look broken.
    $probe = curl.exe -sS -o NUL --max-time 25 -w '%{http_code} %{redirect_url}' ($Base -replace '^https://', 'http://') 2>$null
    $parts = $probe -split ' ', 2
    $code = [int]$parts[0]
    if ($code -ge 300 -and $code -lt 400) {
        Pass 'http redirects to https' ($probe.Trim())
    } else {
        Fail 'http does not redirect' ($probe.Trim())
    }
}

Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
Write-Host ''
if ($failures -eq 0) {
    Write-Host 'all checks passed - the domain is serving this repository' -ForegroundColor Green
    exit 0
}
Write-Host "$failures check(s) failed" -ForegroundColor Red
exit 1
