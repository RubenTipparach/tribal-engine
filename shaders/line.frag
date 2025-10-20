#version 450

// Push constants for line color
layout(push_constant) uniform PushConstants {
    mat4 viewProj;
    vec4 color;
} push;

// Output color
layout(location = 0) out vec4 outColor;

void main() {
    outColor = push.color;
}
