#version 450

// Push constants for widget rendering
layout(push_constant) uniform PushConstants {
    mat4 model;
    mat4 view;
    mat4 projection;
    vec4 color;
} push;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = push.color;
}
