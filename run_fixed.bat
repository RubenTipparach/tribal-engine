@echo off
echo ========================================
echo    Tribal Engine - Build and Run
echo ========================================
echo.

REM Auto-detect Vulkan SDK
set "GLSLC_PATH=C:\VulkanSDK\1.4.328.1\Bin\glslc.exe"

if exist "%GLSLC_PATH%" (
    echo [OK] Found Vulkan SDK 1.4.328.1
) else (
    echo [X] Could not find glslc.exe at expected location
    echo Please check Vulkan SDK installation
    pause
    exit /b 1
)
echo.

REM Check for Rust/Cargo
echo Checking for Rust...
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [X] Rust is not installed!
    echo Please install Rust from: https://rustup.rs/
    pause
    exit /b 1
)
echo [OK] Rust found
echo.

REM Step 1: Compile Shaders
echo ========================================
echo [1/3] Compiling shaders...
echo ========================================
echo.

"%GLSLC_PATH%" shaders/mesh.vert -o shaders/mesh.vert.spv
if %errorlevel% neq 0 (
    echo ERROR: Failed to compile mesh.vert
    pause
    exit /b 1
)
echo   [OK] mesh.vert.spv

"%GLSLC_PATH%" shaders/mesh.frag -o shaders/mesh.frag.spv
if %errorlevel% neq 0 (
    echo ERROR: Failed to compile mesh.frag
    pause
    exit /b 1
)
echo   [OK] mesh.frag.spv

"%GLSLC_PATH%" shaders/skybox.vert -o shaders/skybox.vert.spv
if %errorlevel% neq 0 (
    echo ERROR: Failed to compile skybox.vert
    pause
    exit /b 1
)
echo   [OK] skybox.vert.spv

"%GLSLC_PATH%" shaders/skybox.frag -o shaders/skybox.frag.spv
if %errorlevel% neq 0 (
    echo ERROR: Failed to compile skybox.frag
    pause
    exit /b 1
)
echo   [OK] skybox.frag.spv

echo.
echo All shaders compiled successfully!
echo.

REM Step 2: Build
echo ========================================
echo [2/3] Building project (release mode)...
echo ========================================
echo This may take a few minutes on first build...
echo.

cargo build --release
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Build failed!
    pause
    exit /b 1
)

echo.
echo Build completed!
echo.

REM Step 3: Run
echo ========================================
echo [3/3] Running Tribal Engine...
echo ========================================
echo Close the window or press Ctrl+C to exit
echo.

cargo run --release

echo.
echo ========================================
echo Engine closed
echo ========================================
pause
