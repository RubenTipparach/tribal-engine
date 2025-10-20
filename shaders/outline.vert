#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexCoord;

layout(push_constant) uniform PushConstants {
    mat4 model;
    vec4 outlineColor;   // RGB color + alpha
    float outlineWidth;  // How much to expand the mesh
    float _padding1;
    float _padding2;
    float _padding3;
} push;

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    vec3 viewPos;
    float time;
    // ... other UBO fields
} ubo;

void main() {
    // Expand vertex along normal for outline effect
    vec3 expandedPosition = inPosition + inNormal * push.outlineWidth;

    vec4 worldPos = push.model * vec4(expandedPosition, 1.0);
    gl_Position = ubo.proj * ubo.view * worldPos;
}
