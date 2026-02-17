@echo off
setlocal EnableDelayedExpansion

REM Check if PowerShell is available
where powershell >nul 2>&1
if errorlevel 1 (
    echo ERROR: PowerShell is required to run this script
    exit /b 1
)

REM Downloads sing-box and xray-core pcores and renames them
REM with the correct Tauri target-triple suffix.
REM
REM Usage:  ./cores.bat [target]
REM Example: ./cores.bat x86_64-pc-windows-msvc
REM
REM If no target is given, auto-detects from `rustc`.
REM
REM NOTE: For tar.gz archives (Linux/Mac targets), 7-Zip must be installed at 
REM C:\Program Files\7-Zip\7z.exe or C:\Program Files (x86)\7-Zip\7z.exe

set SINGBOX_VERSION=1.12.22
set XRAY_VERSION=26.2.6
set WINTUN_VERSION=0.14.1

REM Set pcores directory
set "BINDIR=%~dp0src-tauri\pcores"
if not exist "%BINDIR%" mkdir "%BINDIR%"

REM Auto-detect target triple
if "%~1"=="" (
    where rustc >nul 2>&1
    if errorlevel 1 (
        echo ERROR: rustc is not found in PATH and no target specified
        echo Please install Rust or specify a target explicitly
        exit /b 1
    )
    for /f "tokens=*" %%i in ('rustc -vV ^| findstr host') do (
        set "HOST_LINE=%%i"
        for /f "tokens=2" %%j in ("!HOST_LINE!") do set "TARGET=%%j"
    )
) else (
    set "TARGET=%~1"
)

echo Target: %TARGET%
echo Output: %BINDIR%
echo.

REM ── Map target triple → download arch ────────────────

if "%TARGET%"=="x86_64-pc-windows-msvc" (
    set SB_ARCH=windows-amd64
    set SB_EXT=zip
    set XRAY_ARCH=windows-64
    set XRAY_EXT=zip
    set EXE=.exe
) else if "%TARGET%"=="x86_64-pc-windows-gnu" (
    set SB_ARCH=windows-amd64
    set SB_EXT=zip
    set XRAY_ARCH=windows-64
    set XRAY_EXT=zip
    set EXE=.exe
) else if "%TARGET%"=="aarch64-pc-windows-msvc" (
    set SB_ARCH=windows-arm64
    set SB_EXT=zip
    set XRAY_ARCH=windows-arm64-v8a
    set XRAY_EXT=zip
    set EXE=.exe
) else if "%TARGET%"=="x86_64-unknown-linux-gnu" (
    set SB_ARCH=linux-amd64
    set SB_EXT=tar.gz
    set XRAY_ARCH=linux-64
    set XRAY_EXT=zip
    set EXE=
) else if "%TARGET%"=="x86_64-unknown-linux-musl" (
    set SB_ARCH=linux-amd64
    set SB_EXT=tar.gz
    set XRAY_ARCH=linux-64
    set XRAY_EXT=zip
    set EXE=
) else if "%TARGET%"=="aarch64-unknown-linux-gnu" (
    set SB_ARCH=linux-arm64
    set SB_EXT=tar.gz
    set XRAY_ARCH=linux-arm64-v8a
    set XRAY_EXT=zip
    set EXE=
) else if "%TARGET%"=="aarch64-apple-darwin" (
    set SB_ARCH=darwin-arm64
    set SB_EXT=tar.gz
    set XRAY_ARCH=macos-arm64-v8a
    set XRAY_EXT=zip
    set EXE=
) else if "%TARGET%"=="x86_64-apple-darwin" (
    set SB_ARCH=darwin-amd64
    set SB_EXT=tar.gz
    set XRAY_ARCH=macos-64
    set XRAY_EXT=zip
    set EXE=
) else (
    echo ERROR: Unsupported target: %TARGET%
    exit /b 1
)

REM Create temporary directory
set "TMPDIR=%TEMP%\download_cores_%RANDOM%"
mkdir "%TMPDIR%"

REM Cleanup function (called at exit)
set "CLEANUP_DIR=%TMPDIR%"

REM ── sing-box ─────────────────────────────────────────

echo === sing-box v%SINGBOX_VERSION% ===
set "SB_URL=https://github.com/SagerNet/sing-box/releases/download/v%SINGBOX_VERSION%/sing-box-%SINGBOX_VERSION%-%SB_ARCH%.%SB_EXT%"
set "SB_ARCHIVE=%TMPDIR%\singbox.%SB_EXT%"

call :download "!SB_URL!" "!SB_ARCHIVE!"
if errorlevel 1 goto cleanup_and_error

REM Extract the archive based on extension
if "%SB_EXT%"=="zip" (
    powershell -Command "Expand-Archive -Path '!SB_ARCHIVE!' -DestinationPath '!TMPDIR!\singbox' -Force"
) else (
    REM For tar.gz, use 7-Zip if available, otherwise fallback to PowerShell with external tools
    if exist "%ProgramFiles%\7-Zip\7z.exe" (
        "%ProgramFiles%\7-Zip\7z.exe" x -o"!TMPDIR!\temp_tar" "!SB_ARCHIVE!" >nul
        "%ProgramFiles%\7-Zip\7z.exe" x -o"!TMPDIR!\singbox" "!TMPDIR!\temp_tar\*" >nul
    ) else if exist "%ProgramFiles(x86)%\7-Zip\7z.exe" (
        "%ProgramFiles(x86)%\7-Zip\7z.exe" x -o"!TMPDIR!\temp_tar" "!SB_ARCHIVE!" >nul
        "%ProgramFiles(x86)%\7-Zip\7z.exe" x -o"!TMPDIR!\singbox" "!TMPDIR!\temp_tar\*" >nul
    ) else (
        echo ERROR: Cannot extract tar.gz files without 7-Zip installed
        echo Please install 7-Zip from https://www.7-zip.org/ to handle tar.gz archives
        exit /b 1
    )
)

REM Find the binary inside extracted dir
set SB_BIN=
for /f "usebackq" %%i in (`dir /s /b "%TMPDIR%\singbox\*sing-box%EXE%" 2^>nul`) do (
    set "SB_BIN=%%i"
    goto found_sb_bin
)
:found_sb_bin

if not defined SB_BIN (
    echo ERROR: sing-box binary not found in archive
    goto cleanup_and_error
)

copy "!SB_BIN!" "%BINDIR%\sing-box-%TARGET%%EXE%"
echo   ✓ %BINDIR%\sing-box-%TARGET%%EXE%
echo.

REM ── Xray-core ────────────────────────────────────────

echo === Xray-core v%XRAY_VERSION% ===
set "XRAY_URL=https://github.com/XTLS/Xray-core/releases/download/v%XRAY_VERSION%/Xray-%XRAY_ARCH%.%XRAY_EXT%"
set "XRAY_ARCHIVE=%TMPDIR%\xray.%XRAY_EXT%"

call :download "!XRAY_URL!" "!XRAY_ARCHIVE!"
if errorlevel 1 goto cleanup_and_error

REM Extract Xray archive (usually zip)
if "%XRAY_EXT%"=="zip" (
    if exist "%ProgramFiles%\7-Zip\7z.exe" (
        "%ProgramFiles%\7-Zip\7z.exe" x -o"!TMPDIR!\xray" "!XRAY_ARCHIVE!" >nul
    ) else if exist "%ProgramFiles(x86)%\7-Zip\7z.exe" (
        "%ProgramFiles(x86)%\7-Zip\7z.exe" x -o"!TMPDIR!\xray" "!XRAY_ARCHIVE!" >nul
    ) else (
        powershell -Command "Expand-Archive -Path '!XRAY_ARCHIVE!' -DestinationPath '!TMPDIR!\xray' -Force"
    )
) else (
    REM Handle other extensions (shouldn't occur for Xray but kept for consistency)
    echo ERROR: Unexpected Xray archive extension: %XRAY_EXT%
    exit /b 1
)

REM Find the Xray binary
set XRAY_BIN=
for /f "usebackq" %%i in (`dir /s /b "%TMPDIR%\xray\*xray%EXE%" 2^>nul`) do (
    set "XRAY_BIN=%%i"
    goto found_xray_bin
)
:found_xray_bin

if not defined XRAY_BIN (
    echo ERROR: xray binary not found in archive
    goto cleanup_and_error
)

copy "!XRAY_BIN!" "%BINDIR%\xray-%TARGET%%EXE%"
echo   ✓ %BINDIR%\xray-%TARGET%%EXE%
echo.

REM ── Download Xray GeoIP and Geosite databases ──────────────────────────────────────

echo === Xray GeoIP and Geosite databases ===

set "GEOIP_URL=https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geoip.dat"
set "GEOSITE_URL=https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geosite.dat"

call :download "!GEOIP_URL!" "%BINDIR%\geoip.dat"
call :download "!GEOSITE_URL!" "%BINDIR%\geosite.dat"

echo   ✓ %BINDIR%\geoip.dat
echo   ✓ %BINDIR%\geosite.dat
echo.

REM ── wintun.dll for TUN mode ──────────────────────────────

if "%EXE%"==".exe" (
    echo === wintun.dll v%WINTUN_VERSION% ===
    
    REM Determine architecture for wintun.dll
    if "%TARGET%"=="x86_64-pc-windows-msvc" (
        set "WINTUN_ARCH=amd64"
    ) else if "%TARGET%"=="x86_64-pc-windows-gnu" (
        set "WINTUN_ARCH=amd64"
    ) else if "%TARGET%"=="aarch64-pc-windows-msvc" (
        set "WINTUN_ARCH=arm64"
    ) else (
        set "WINTUN_ARCH=amd64"
    )
    
    set "WINTUN_URL=https://www.wintun.net/builds/wintun-%WINTUN_VERSION%.zip"
    set "WINTUN_ARCHIVE=%TMPDIR%\wintun.zip"
    
    call :download "!WINTUN_URL!" "!WINTUN_ARCHIVE!"
    if errorlevel 1 goto cleanup_and_error
    
    REM Extract wintun.dll
    powershell -Command "Expand-Archive -Path '!WINTUN_ARCHIVE!' -DestinationPath '!TMPDIR!\wintun' -Force"
    
    REM Copy the appropriate architecture version of wintun.dll
    copy "!TMPDIR!\wintun\wintun\bin\!WINTUN_ARCH!\wintun.dll" "%BINDIR%\wintun.dll"
    echo   ✓ %BINDIR%\wintun.dll
    echo.
)

echo === Done! ===
echo pcores ready in %BINDIR%:
dir "%BINDIR%"

REM Cleanup temp directory
rd /s /q "%CLEANUP_DIR%" 2>nul
goto :eof

REM ── Helper Functions ──────────────────────────────────────────

:download
set "url=%~1"
set "dest=%~2"
echo   Downloading !url!
powershell -Command "Invoke-WebRequest -Uri '%url%' -OutFile '%dest%' -UserAgent 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'"
if errorlevel 1 (
    echo ERROR: Failed to download file
    exit /b 1
)
goto :eof

:cleanup_and_error
rd /s /q "%CLEANUP_DIR%" 2>nul
exit /b 1