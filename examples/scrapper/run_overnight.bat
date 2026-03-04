@echo off
setlocal enabledelayedexpansion

:: Path to project and virtual environment
echo ========================================
echo  Starting the Overnight Scrapper Script
echo ========================================
set PROJECT_DIR=C:\Users\saran\Videos\Projects\FDM\scrapper
set SRC_DIR=%PROJECT_DIR%\src
set VENV_DIR=%PROJECT_DIR%\modern_env

:: Activate virtual environment
echo Activating virtual environment from: %VENV_DIR%
call "%VENV_DIR%\Scripts\activate.bat"
if %ERRORLEVEL% neq 0 (
    echo Failed to activate virtual environment. Exiting...
    pause
    exit /b 1
)
echo Virtual environment activated.

:loop
echo ----------------------------------------
echo Switching to source directory: %SRC_DIR%
cd /d "%SRC_DIR%"

echo Running scrapper.main using Python...
python -m scrapper.main

if %ERRORLEVEL% neq 0 (
    echo Scrapper exited with error code %ERRORLEVEL%.
) else (
    echo Scrapper run completed successfully.
)

echo ⏳ Restarting in 10 seconds...
timeout /t 10 /nobreak >nul
echo Restarting scrapper...
goto loop
