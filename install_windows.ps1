@echo off
setlocal enabledelayedexpansion

echo Installing discord_mpv_rpc...

:: Check if cargo is installed
where cargo >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Error: cargo is not installed. Please install Rust first.
    exit /b 1
)

:: Check for package managers and set MPV paths accordingly
set "MPV_BASE_PATH="
set "PACKAGE_MANAGER="

if exist "%APPDATA%\mpv" (
    set "MPV_BASE_PATH=%APPDATA%\mpv"
) else (
    :: Check Scoop
    where scoop >nul 2>&1
    if %ERRORLEVEL% equ 0 (
        for /f "tokens=*" %%i in ('scoop which mpv 2^>nul') do (
            set "MPV_BASE_PATH=%%~dpi.."
            set "PACKAGE_MANAGER=scoop"
        )
    )
    
    :: Check Chocolatey
    if not defined MPV_BASE_PATH (
        if exist "C:\ProgramData\chocolatey\bin\mpv.exe" (
            set "MPV_BASE_PATH=C:\ProgramData\chocolatey\lib\mpv"
            set "PACKAGE_MANAGER=chocolatey"
        )
    )
)

if not defined MPV_BASE_PATH (
    echo Error: Could not find MPV installation. Please install MPV first.
    exit /b 1
)

cargo build --release
if %ERRORLEVEL% neq 0 (
    echo Build failed. Please check the error messages above.
    exit /b 1
)

set "CONFIG_DIR=%APPDATA%\discord_mpv_rpc"
set "MPV_SCRIPTS_DIR=%MPV_BASE_PATH%\scripts"

mkdir "%CONFIG_DIR%" 2>nul
mkdir "%MPV_SCRIPTS_DIR%" 2>nul

echo Installing binary...
copy /Y "target\release\discord_mpv_rpc.exe" "%LOCALAPPDATA%\Programs\discord_mpv_rpc\" 
if %ERRORLEVEL% neq 0 (
    echo Failed to install binary. Please check permissions.
    exit /b 1
)

echo Installing MPV script...
copy /Y "discord-rpc.lua" "%MPV_SCRIPTS_DIR%\"
if %ERRORLEVEL% neq 0 (
    echo Failed to install MPV script.
    exit /b 1
)

set "CONFIG_FILE=%CONFIG_DIR%\config.toml"
if not exist "%CONFIG_FILE%" (
    echo Creating default config...
    (
        echo socket = "\\.\pipe\mpvsocket"
        echo.
        echo # Get this from https://discord.com/developers/applications if you want to have personal assets
        echo client_id = "1322011605432533082"
        echo.
        echo large_image = "mpv_large"
        echo small_image = "mpv_small"
        echo.
        echo # Optional: Custom status text
        echo status_text = "Watching Anime"
    ) > "%CONFIG_FILE%"
)

set "MPV_CONFIG=%MPV_BASE_PATH%\mpv.conf"
findstr /C:"input-ipc-server=\\.\pipe\mpvsocket" "%MPV_CONFIG%" >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Configuring MPV...
    echo.>> "%MPV_CONFIG%"
    echo # Added by discord_mpv_rpc>> "%MPV_CONFIG%"
    echo input-ipc-server=\\.\pipe\mpvsocket>> "%MPV_CONFIG%"
)

echo.
echo Installation complete!
echo Discord RPC will start automatically when you play videos in MPV
echo Press 'D' in MPV to toggle Discord RPC on/off
echo.
echo Don't forget to set your client_id in: %CONFIG_FILE%

if defined PACKAGE_MANAGER (
    echo Note: MPV was installed through %PACKAGE_MANAGER%. Configuration files are located accordingly.
)

endlocal
