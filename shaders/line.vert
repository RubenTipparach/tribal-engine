#version 450

// Input vertex data (positions)
layout(location = 0) in vec3 inPosition;

// Push constants for line rendering
layout(push_constant) uniform PushConstants {
    mat4 viewProj;    // Combined view-projection matrix
    vec4 color;       // Line color (RGBA)
} push;

void main() {
    gl_Position = push.viewProj * vec4(inPosition, 1.0);
}
