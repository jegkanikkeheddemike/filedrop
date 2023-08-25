cd %~dp0%
cargo build --release
set mypath=%~dp0%target\release\
setx path /M "%PATH%;%mypath%"
copy .\startup_windows.bat "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\filedrop_daemon.bat"

timeout /t 30