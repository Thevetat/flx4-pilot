@echo off
cd /d "%~dp0"
flx4-pilot.exe %*
if errorlevel 1 pause
