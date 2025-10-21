#version 450

// Color via push constants (after mat4 model at offset 64)
layout(push_constant) uniform PushConstants {
    layout(offset = 64) vec4 color;
} push;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = push.color;
}
