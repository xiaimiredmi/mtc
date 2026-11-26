# Register the MTC viewer for .mtc files and put the codec on PATH.
#
# Everything here is per-user: HKCU and the user Path, no administrator needed.
#   powershell -ExecutionPolicy Bypass -File deploy\register.ps1
#   powershell -ExecutionPolicy Bypass -File deploy\register.ps1 -Unregister
#
# ASCII-only on purpose: Windows PowerShell 5.1 reads a .ps1 as ANSI unless it has
# a BOM, so the one Chinese label is built from code points instead of written
# literally (U+56FE U+7247 is "tu pian", picture).
param(
    [switch]$Unregister,
    [string]$Bin = (Join-Path $PSScriptRoot '..\bin')
)

$ErrorActionPreference = 'Stop'
$Bin = (Resolve-Path $Bin).Path
$viewer = Join-Path $Bin 'mtview.exe'
$iconFile = Join-Path $PSScriptRoot 'mtc.ico'
$icon = if (Test-Path $iconFile) { "$iconFile,0" } else { "$viewer,0" }
$friendly = 'MTC ' + [char]0x56FE + [char]0x7247

function Set-Default($path, $value) {
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-Item -Path $path -Value $value -Force
}

function Set-String($path, $name, $value) {
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    New-ItemProperty -Path $path -Name $name -Value $value -PropertyType String -Force | Out-Null
}

if ($Unregister) {
    foreach ($k in 'HKCU:\Software\Classes\.mtc', 'HKCU:\Software\Classes\MTC.Image',
                   'HKCU:\Software\Classes\.mtb', 'HKCU:\Software\Classes\MTB.Image',
                   'HKCU:\Software\Classes\MTB.Video') {
        if (Test-Path $k) { Remove-Item -Path $k -Recurse -Force }
    }
    $raw = (Get-Item HKCU:\Environment).GetValue('Path', $null, 'DoNotExpandEnvironmentNames')
    $kept = ($raw -split ';') | Where-Object { $_ -ne '' -and $_ -ne $Bin }
    Set-ItemProperty -Path HKCU:\Environment -Name Path -Value ($kept -join ';') -Type ExpandString
    Write-Host "unregistered .mtc and removed $Bin from the user Path"
    exit 0
}

if (-not (Test-Path $viewer)) {
    throw "mtview.exe is not in $Bin. Build it with 'cargo build --release --features viewer' and copy the binaries there."
}

# .mtc -> MTC.Image -> mtview
Set-Default 'HKCU:\Software\Classes\.mtc' 'MTC.Image'
Set-String 'HKCU:\Software\Classes\.mtc' 'Content Type' 'image/x-mtc'
Set-Default 'HKCU:\Software\Classes\MTC.Image' $friendly
Set-String 'HKCU:\Software\Classes\MTC.Image' 'FriendlyTypeName' $friendly
Set-String 'HKCU:\Software\Classes\MTC.Image' 'PerceivedType' 'image'
Set-Default 'HKCU:\Software\Classes\MTC.Image\DefaultIcon' $icon
Set-Default 'HKCU:\Software\Classes\MTC.Image\shell\open\command' ('"{0}" "%1"' -f $viewer)

# The video container is gone; drop its registrations if an older install left them.
foreach ($k in 'HKCU:\Software\Classes\.mtb', 'HKCU:\Software\Classes\MTB.Image', 'HKCU:\Software\Classes\MTB.Video') {
    if (Test-Path $k) { Remove-Item -Path $k -Recurse -Force }
}

# Put the binaries on PATH, ahead of anything else that answers to mtcodec.
$raw = (Get-Item HKCU:\Environment).GetValue('Path', $null, 'DoNotExpandEnvironmentNames')
$kept = ($raw -split ';') | Where-Object { $_ -ne '' -and $_ -ne $Bin }
Set-ItemProperty -Path HKCU:\Environment -Name Path -Value ((@($Bin) + $kept) -join ';') -Type ExpandString

# Tell running shells and Explorer that the environment changed.
if (-not ('Win32.Env' -as [type])) {
    Add-Type -Namespace Win32 -Name Env -MemberDefinition @'
[System.Runtime.InteropServices.DllImport("user32.dll", SetLastError = true, CharSet = System.Runtime.InteropServices.CharSet.Auto)]
public static extern System.IntPtr SendMessageTimeout(System.IntPtr hWnd, uint Msg, System.UIntPtr wParam, string lParam, uint fuFlags, uint uTimeout, out System.UIntPtr lpdwResult);
'@
}
$result = [UIntPtr]::Zero
[void][Win32.Env]::SendMessageTimeout([IntPtr]0xffff, 0x1A, [UIntPtr]::Zero, 'Environment', 2, 3000, [ref]$result)

# Explorer caches file-type icons hard. Tell it the associations changed and rebuild the
# cache, or a freshly registered icon still shows as a blank page until the next logon.
if (-not ('Win32.Shell' -as [type])) {
    Add-Type -Namespace Win32 -Name Shell -MemberDefinition @'
[System.Runtime.InteropServices.DllImport("shell32.dll")]
public static extern void SHChangeNotify(int eventId, uint flags, System.IntPtr item1, System.IntPtr item2);
'@
}
[Win32.Shell]::SHChangeNotify(0x08000000, 0x0000, [IntPtr]::Zero, [IntPtr]::Zero)
$ie4uinit = Join-Path $env:SystemRoot 'System32\ie4uinit.exe'
if (Test-Path $ie4uinit) { Start-Process -FilePath $ie4uinit -ArgumentList '-show' -WindowStyle Hidden }
Write-Host "registered: .mtc opens with $viewer"
Write-Host "icon:       $icon"
Write-Host "PATH:       $Bin (first)"
Write-Host ""
Write-Host "Explorer caches associations; sign out once if a double-click still opens an old viewer."
