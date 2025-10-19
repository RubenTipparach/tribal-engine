#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec3 viewPos;
    vec3 color;     // Color from UBO - should be green
} ubo;

layout(location = 0) in vec3 fragPosition;
layout(location = 1) in vec3 fragNormal;

layout(location = 0) out vec4 outColor;

void main() {
    // Use color from UBO (should be green if working)
    outColor = vec4(ubo.color, 1.0);
}
