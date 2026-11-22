# Wait for GitHub Pages to issue the certificate, then turn on Enforce HTTPS.
#
#   powershell -ExecutionPolicy Bypass -File site\wait-for-https.ps1
#   powershell -ExecutionPolicy Bypass -File site\wait-for-https.ps1 -Minutes 30
#
# Reads a GitHub token from the credential helper `git credential` already has, so
# nothing secret is typed or printed. Exits 0 once https is trusted and enforced,
# 1 if it gave up waiting.
#
# ASCII-only: Windows PowerShell 5.1 reads a .ps1 as ANSI unless it has a BOM.
param(
    [string]$Domain = 'mtc-xiaimiredmi.top',
    [string]$Repo = 'xiaimiredmi/mtc',
    [int]$Minutes = 30,
    [int]$Every = 60
)

$ErrorActionPreference = 'Continue'
$deadline = (Get-Date).AddMinutes($Minutes)

function Get-Token {
    $cred = "protocol=https`nhost=github.com`n`n" | git credential fill 2>$null
    $line = ($cred | Select-String '^password=').Line
    if (-not $line) { return $null }
    return ($line -replace '^password=', '')
}

function Get-Pages($headers) {
    try {
        return Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/pages" -Headers $headers -TimeoutSec 30
    } catch {
        return $null
    }
}

function CertTrusted {
    # No -k: this only succeeds once the served certificate validates, which is
    # exactly the condition "the certificate has been issued".
    $code = & curl.exe -sS -o NUL -w '%{http_code}' --max-time 20 "https://$Domain/" 2>$null
    return ($LASTEXITCODE -eq 0 -and $code -eq '200')
}

$token = Get-Token
if (-not $token) { Write-Host 'no stored GitHub credential'; exit 1 }
$headers = @{ Authorization = "Bearer $token"; Accept = 'application/vnd.github+json'; 'User-Agent' = 'dsh' }

$pages = Get-Pages $headers
if ($pages) {
    Write-Host ("start: cname={0} https_enforced={1} certificate={2}" -f `
        $pages.cname, $pages.https_enforced, ($pages.https_certificate.state))
} else {
    Write-Host 'could not read the Pages settings'
}

while ((Get-Date) -lt $deadline) {
    $stamp = Get-Date -Format 'HH:mm:ss'
    $pages = Get-Pages $headers
    $certState = if ($pages -and $pages.https_certificate) { $pages.https_certificate.state } else { 'none' }
    $trusted = CertTrusted
    Write-Host ("  {0}  certificate={1,-10} https_works={2}" -f $stamp, $certState, $trusted)

    if ($trusted) {
        if ($pages -and -not $pages.https_enforced) {
            $body = @{
                cname          = $Domain
                build_type     = 'workflow'
                https_enforced = $true
            } | ConvertTo-Json
            try {
                Invoke-RestMethod -Method Put -Uri "https://api.github.com/repos/$Repo/pages" `
                    -Headers $headers -Body $body -ContentType 'application/json' -TimeoutSec 30 | Out-Null
                $after = Get-Pages $headers
                Write-Host ("  enforced https: now {0}" -f $after.https_enforced)
            } catch {
                Write-Host ('  could not enable enforcement: ' + $_.Exception.Message)
            }
        }
        Write-Host 'https is live'
        $token = $null
        exit 0
    }
    Start-Sleep -Seconds $Every
}

Write-Host 'gave up waiting; the certificate is still pending'
$token = $null
exit 1
