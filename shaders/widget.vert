#version 450

// Push constants for widget rendering
layout(push_constant) uniform PushConstants {
    mat4 model;
    mat4 view;
    mat4 projection;
    vec4 color;
} push;

layout(location = 0) in vec3 inPosition;

void main() {
    gl_Position = push.projection * push.view * push.model * vec4(inPosition, 1.0);
}
