@echo off
REM Shader compilation script for Tribal Engine
REM Compiles all GLSL shaders to SPIR-V using glslc

setlocal enabledelayedexpansion

set GLSLC="C:\VulkanSDK\1.4.328.1\Bin\glslc.exe"
set SHADER_DIR=shaders
set ERROR_COUNT=0
set SUCCESS_COUNT=0

echo ========================================
echo Compiling Tribal Engine Shaders
echo ========================================
echo.

REM Check if glslc exists
if not exist %GLSLC% (
    echo ERROR: glslc not found at %GLSLC%
    echo Please update the GLSLC path in this script
    exit /b 1
)

REM Compile all vertex shaders
echo Compiling vertex shaders...
for %%f in (%SHADER_DIR%\*.vert) do (
    echo   Compiling %%f...
    %GLSLC% %%f -o %%f.spv
    if !errorlevel! neq 0 (
        echo     [FAILED]
        set /a ERROR_COUNT+=1
    ) else (
        echo     [OK]
        set /a SUCCESS_COUNT+=1
    )
)

echo.

REM Compile all fragment shaders
echo Compiling fragment shaders...
for %%f in (%SHADER_DIR%\*.frag) do (
    echo   Compiling %%f...
    %GLSLC% %%f -o %%f.spv
    if !errorlevel! neq 0 (
        echo     [FAILED]
        set /a ERROR_COUNT+=1
    ) else (
        echo     [OK]
        set /a SUCCESS_COUNT+=1
    )
)

echo.
echo ========================================
echo Compilation Summary
echo ========================================
echo Success: !SUCCESS_COUNT!
echo Errors:  !ERROR_COUNT!
echo ========================================

if !ERROR_COUNT! gtr 0 (
    exit /b 1
)

exit /b 0
