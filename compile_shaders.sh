#!/bin/bash

echo "Compiling shaders..."

# Make sure glslc is in your PATH (comes with Vulkan SDK)

glslc shaders/mesh.vert -o shaders/mesh.vert.spv || exit 1
glslc shaders/mesh.frag -o shaders/mesh.frag.spv || exit 1
glslc shaders/skybox.vert -o shaders/skybox.vert.spv || exit 1
glslc shaders/skybox.frag -o shaders/skybox.frag.spv || exit 1
glslc shaders/imgui.vert -o shaders/imgui.vert.spv || exit 1
glslc shaders/imgui.frag -o shaders/imgui.frag.spv || exit 1

echo "All shaders compiled successfully!"
