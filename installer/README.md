# MTC for Windows

A single `.exe` that installs the command line tools, the viewer, and the `.mtc`
file association. No Inno Setup, no WiX, no NSIS: it is C# compiled by the `csc.exe`
that ships with .NET Framework, and the binaries ride along as embedded resources,
so building it needs nothing installed and running it needs nothing installed
either.

## Build

```powershell
powershell -ExecutionPolicy Bypass -File installer\build.ps1          # -> installer\MtcSetup.exe
powershell -ExecutionPolicy Bypass -File installer\build.ps1 -Check   # + payload round trip
```

The payload is `bin\mtcodec.exe`, `bin\mtbench.exe`, `bin\mtview.exe`, plus the icon
and a short README, so build those first (`cargo build --release --features viewer`
in `rust/`, then copy them to `bin\`).

## Use

Double-click `MtcSetup.exe` for the wizard: pick a folder, tick at least "关联 .mtc",
press 安装. It is per-user — `%LOCALAPPDATA%\Programs\MTC` by default, `HKCU` only, so
Windows never asks for administrator rights.

Command line, for scripting or silent deployment:

```
MtcSetup.exe                                     the wizard
MtcSetup.exe /S                                  silent install with the defaults
MtcSetup.exe /S /dir "D:\Tools\MTC"              choose the folder
MtcSetup.exe /S /nopath /noassoc /nostartmenu    leave pieces out
MtcSetup.exe /extract "D:\tmp"                   unpack only, change nothing
MtcSetup.exe /uninstall [/S]                     remove it
```

## What it touches

| Where | What |
| --- | --- |
| `<dir>` | the three binaries, `mtc.ico`, `README.txt`, `uninstall.exe` |
| `HKCU\Software\Classes\.mtc` | → `MTC.Image`, content type `image/x-mtc` |
| `HKCU\Software\Classes\MTC.Image` | friendly name, icon, `shell\open\command` |
| `HKCU\Environment` `Path` | install folder, first, deduplicated |
| `HKCU\...\CurrentVersion\Uninstall\MTC` | Add/Remove Programs entry |
| Start menu | viewer, terminal, uninstall shortcuts |

Uninstalling removes all of it, including the folder. The folder is deleted by a
small batch file left in `%TEMP%`, because Windows will not delete a running image
and the uninstaller is one of the files; that is also why the uninstaller checks for
`mtview.exe` before removing anything.

Silent installs pick up `%LOCALAPPDATA%\Programs\MTC`; uninstalls read the install
location back from the registry, so `uninstall.exe /S` needs no arguments.

## Verified

Three consecutive install → check → uninstall cycles, each asserting: seven files in
place, association pointing at the new folder, the folder first on `PATH`, the
uninstall entry carrying `/uninstall`, the desktop shortcut created — then, after the
uninstall, the folder gone, the registry keys gone, `PATH` cleaned, shortcut gone, no
leftovers in `%TEMP%`.
