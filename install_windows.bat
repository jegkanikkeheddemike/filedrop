mkdir %HOMEDRIVE%%HOMEPATH%\filedrop
cd %HOMEDRIVE%%HOMEPATH%\filedrop
curl -LO https://raw.githubusercontent.com/jegkanikkeheddemike/filedrop/main/binaries/filedrop.exe
curl -LO https://raw.githubusercontent.com/jegkanikkeheddemike/filedrop/main/binaries/filedrop_daemon.exe
curl -LO https://raw.githubusercontent.com/jegkanikkeheddemike/filedrop/main/startup_windows.bat
set mypath=%HOMEDRIVE%%HOMEPATH%\filedrop\
echo %mypath%
setx path /M "%PATH%%mypath%"
copy .\startup_windows.bat "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\filedrop_daemon.bat"
.\startup_windows.bat

timeout /t 30