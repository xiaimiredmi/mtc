# Creates desktop and Start Menu shortcuts for the viewer, so .mtc files and the app have
# somewhere obvious to be clicked on Windows.
#
# ASCII only on purpose: PowerShell 5.1 reads a BOM-less .ps1 as ANSI, and Chinese string
# literals then arrive as mojibake. The Chinese name is built from code points instead.
param(
    [switch]$Remove,
    [string]$Bin = (Join-Path $PSScriptRoot '..\bin')
)

$ErrorActionPreference = 'Stop'

$Bin = (Resolve-Path $Bin).Path
$viewer = Join-Path $Bin 'mtview.exe'
if (-not (Test-Path $viewer)) {
    throw "mtview.exe not found in $Bin"
}
$iconFile = Join-Path $PSScriptRoot 'mtc.ico'
$icon = if (Test-Path $iconFile) { "$iconFile,0" } else { "$viewer,0" }

# MTC + picture + viewer
$name = 'MTC ' + [char]0x56FE + [char]0x7247 + [char]0x67E5 + [char]0x770B + [char]0x5668
$description = 'MTC ' + [char]0x56FE + [char]0x7247 + [char]0x67E5 + [char]0x770B + [char]0x5668 + ' - ' + [char]0x9009 + [char]0x62E9 + [char]0x4E00 + [char]0x5F20 + [char]0x56FE + [char]0x7247

$places = @([Environment]::GetFolderPath('Desktop'))
$programs = [Environment]::GetFolderPath('Programs')
if ($programs) {
    $places += (Join-Path $programs 'MTC')
}

$shell = New-Object -ComObject WScript.Shell
foreach ($place in $places) {
    $link = Join-Path $place "$name.lnk"
    if ($Remove) {
        if (Test-Path $link) {
            Remove-Item $link -Force
            Write-Host "removed  $link"
        }
        continue
    }
    if (-not (Test-Path $place)) {
        New-Item -ItemType Directory -Path $place -Force | Out-Null
    }
    $shortcut = $shell.CreateShortcut($link)
    $shortcut.TargetPath = $viewer
    $shortcut.WorkingDirectory = $Bin
    $shortcut.IconLocation = $icon
    $shortcut.Description = $description
    $shortcut.Save()
    Write-Host "created  $link"
}

if (-not $Remove) {
    Write-Host ''
    Write-Host 'Double-clicking a .mtc file works too: register.ps1 points the file type at the'
    Write-Host 'same viewer, so the shortcut is a convenience rather than the only way in.'
}
