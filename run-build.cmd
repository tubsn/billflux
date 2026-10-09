@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build-portable.ps1" %*
exit /b %ERRORLEVEL%
