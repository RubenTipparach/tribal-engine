#!/bin/bash

echo "========================================"
echo "   Tribal Engine - Build and Run"
echo "========================================"
echo ""

# Set Vulkan SDK path
GLSLC="/c/VulkanSDK/1.4.328.1/Bin/glslc.exe"

# Check if glslc exists
if [ ! -f "$GLSLC" ]; then
    echo "[X] Could not find glslc.exe at $GLSLC"
    echo "Please check your Vulkan SDK installation"
    exit 1
fi

echo "[OK] Found Vulkan SDK"
echo ""

# Check for cargo
if ! command -v cargo &> /dev/null; then
    echo "[X] Rust/Cargo not found!"
    echo "Please install Rust from: https://rustup.rs/"
    exit 1
fi

echo "[OK] Rust found"
echo ""

# Step 1: Compile Shaders
echo "========================================"
echo "[1/3] Compiling shaders..."
echo "========================================"
echo ""

"$GLSLC" shaders/mesh.vert -o shaders/mesh.vert.spv || exit 1
echo "  [OK] mesh.vert.spv"

"$GLSLC" shaders/mesh.frag -o shaders/mesh.frag.spv || exit 1
echo "  [OK] mesh.frag.spv"

"$GLSLC" shaders/skybox.vert -o shaders/skybox.vert.spv || exit 1
echo "  [OK] skybox.vert.spv"

"$GLSLC" shaders/skybox.frag -o shaders/skybox.frag.spv || exit 1
echo "  [OK] skybox.frag.spv"

echo ""
echo "All shaders compiled successfully!"
echo ""

# Step 2: Build
echo "========================================"
echo "[2/3] Building project (release mode)..."
echo "========================================"
echo "This may take a few minutes on first build..."
echo ""

cargo build --release || exit 1

echo ""
echo "Build completed!"
echo ""

# Step 3: Run
echo "========================================"
echo "[3/3] Running Tribal Engine..."
echo "========================================"
echo "Press Ctrl+C to exit"
echo ""

cargo run --release

echo ""
echo "========================================"
echo "Engine closed"
echo "========================================"
