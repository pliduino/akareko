@echo off
REM Usage: launch-aurora.bat <index>
REM Example: launch-aurora.bat 3
REM This will open Aurora.exe in the folder test_3

if "%~1"=="" (
  echo Usage: %~nx0 ^<index^>
  exit /b 1
)

setlocal enabledelayedexpansion
set INDEX=%1
set BASEDIR=%~dp0
set EXE=%BASEDIR%target\debug\Aurora.exe

set FOLDER=%BASEDIR%test\%INDEX%
if not exist "%FOLDER%" (
  mkdir "%FOLDER%"
)

REM launch Aurora in its own window and keep cmd open
start "Aurora %INDEX%" /D "%FOLDER%" cmd /k "%EXE%"

endlocal