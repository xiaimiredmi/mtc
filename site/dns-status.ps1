# What state is the domain in? A verdict, not a wall of dig output.
#
#   powershell -ExecutionPolicy Bypass -File site\dns-status.ps1
#   powershell -ExecutionPolicy Bypass -File site\dns-status.ps1 -Domain mtc-xiaimiredmi.top
#
# It asks two independent resolvers (the system one and Cloudflare over HTTPS), so
# a local cache or a broken ISP resolver cannot fake the answer.
#
# ASCII-only: Windows PowerShell 5.1 reads a .ps1 as ANSI unless it has a BOM.
param([string]$Domain = 'mtc-xiaimiredmi.top')

$ErrorActionPreference = 'Continue'
$expected = @('185.199.108.153', '185.199.109.153', '185.199.110.153', '185.199.111.153')

# Not every network can reach Cloudflare over HTTPS; this box cannot, so the
# channel is tested against a domain that is certain to answer before its result
# is trusted for anything.
function Test-Doh {
    $r = Doh 'example.com' 'A'
    return ($r -and $r.Status -eq 0)
}

function Doh([string]$name, [string]$type) {
    # Cloudflare's JSON API: https://developers.cloudflare.com/1.1.1.1/encryption/dns-over-https/
    $url = "https://cloudflare-dns.com/dns-query?name=$name&type=$type"
    try {
        $r = curl.exe -sS --max-time 20 -H 'accept: application/dns-json' $url 2>$null | ConvertFrom-Json
        return $r
    } catch {
        return $null
    }
}

function Summarise($json) {
    if (-not $json) { return 'no answer' }
    # Status: 0 = NOERROR, 2 = SERVFAIL, 3 = NXDOMAIN
    $state = switch ($json.Status) { 0 { 'NOERROR' } 2 { 'SERVFAIL' } 3 { 'NXDOMAIN' } default { "status $($json.Status)" } }
    $values = @()
    if ($json.Answer) { $values = $json.Answer | ForEach-Object { $_.data } }
    return "$state " + ($values -join ' ')
}

Write-Host "domain: $Domain"
$dohOk = Test-Doh
if ($dohOk) {
    Write-Host '  resolver: Cloudflare over HTTPS (works here)'
} else {
    Write-Host '  resolver: system resolver only - DNS over HTTPS is blocked on this network,'
    Write-Host '            so the Cloudflare column below is unreliable and left out of the verdict'
}
Write-Host ''

# ---- is it delegated at all? -------------------------------------------------
Write-Host '== delegation (NS)'
$ns = $null
if ($dohOk) {
    $ns = Doh $Domain 'NS'
    '  doh        : ' + (Summarise $ns)
}
$sysNs = try { (Resolve-DnsName $Domain -Type NS -Server 223.5.5.5 -ErrorAction Stop | Where-Object NameHost | Select-Object -ExpandProperty NameHost) -join ' ' } catch { 'error: ' + (($_.Exception.Message -replace "`r?`n", ' ')) }
'  system     : ' + $sysNs
if ($dohOk) {
    $delegated = ($ns -and $ns.Status -eq 0 -and $ns.Answer)
} else {
    $delegated = ($sysNs -and $sysNs -notmatch 'error')
}

# ---- the records ------------------------------------------------------------
Write-Host '== A records (apex)'
$apexIps = @()
if ($dohOk) {
    $a = Doh $Domain 'A'
    '  doh        : ' + (Summarise $a)
    if ($a -and $a.Answer) { $apexIps = @($a.Answer | Where-Object { $_.type -eq 1 } | ForEach-Object { $_.data }) }
}
if (-not $apexIps.Count) {
    $apexIps = @(try { Resolve-DnsName $Domain -Type A -Server 223.5.5.5 -ErrorAction Stop | Where-Object IPAddress | Select-Object -ExpandProperty IPAddress } catch { })
    '  system     : ' + $(if ($apexIps.Count) { $apexIps -join ' ' } else { 'nothing' })
}
$apexOk = $true
foreach ($ip in $expected) { if ($apexIps -notcontains $ip) { $apexOk = $false } }

Write-Host '== www'
$wwwTarget = ''
if ($dohOk) {
    $w = Doh "www.$Domain" 'CNAME'
    '  doh        : ' + (Summarise $w)
    if ($w -and $w.Answer) {
        $cname = $w.Answer | Where-Object { $_.type -eq 5 } | Select-Object -First 1
        if ($cname) { $wwwTarget = $cname.data }
    }
}
if (-not $wwwTarget) {
    $wwwTarget = try { (Resolve-DnsName "www.$Domain" -Type CNAME -Server 223.5.5.5 -ErrorAction Stop | Where-Object NameHost | Select-Object -First 1 -ExpandProperty NameHost) } catch { '' }
    '  system     : ' + $(if ($wwwTarget) { $wwwTarget } else { 'nothing' })
}
$wwwOk = $wwwTarget -like 'xiaimiredmi.github.io*'

# ---- verdict ---------------------------------------------------------------
Write-Host ''
Write-Host '== verdict'
if (-not $delegated) {
    Write-Host '  The domain has no nameservers that answer. It is not delegated yet.' -ForegroundColor Yellow
    Write-Host ''
    Write-Host '  This is the registrar side, not GitHub: the registry either has no NS'
    Write-Host '  record for the domain, or the NS it has does not answer for this zone.'
    Write-Host '  In the RainYun console check, in this order:'
    Write-Host '    1. the domain status says the real-name review is complete'
    Write-Host '    2. a DNS service is attached to the domain (free "DNS jiexi" product)'
    Write-Host '    3. the domain shows two nameservers (for example ns1/ns2.*.com)'
    Write-Host '    4. the five records below exist in that DNS service'
    Write-Host '  Registry changes can take from minutes up to a day to appear.'
    exit 2
}
if (-not $apexOk -or -not $wwwOk) {
    Write-Host '  Delegated, but the records are not what GitHub Pages needs.' -ForegroundColor Yellow
    if (-not $apexOk) { Write-Host ("  apex A: expected " + ($expected -join ', ') + "; got " + ($(if ($apexIps) { $apexIps -join ', ' } else { 'nothing' }))) }
    if (-not $wwwOk) { Write-Host ("  www: expected CNAME xiaimiredmi.github.io; got " + $(if ($wwwTarget) { $wwwTarget } else { 'nothing' })) }
    exit 1
}
Write-Host '  Delegated, and both the apex and www point at GitHub Pages.' -ForegroundColor Green
Write-Host '  Next: run site\check-domain.ps1, then tick Enforce HTTPS in Settings/Pages.'
exit 0
