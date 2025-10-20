#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    vec3 viewPos;
} ubo;

layout(location = 0) in vec3 fragPosition;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragUV;
layout(location = 3) in vec3 viewPos;

layout(location = 0) out vec4 outColor;

void main() {
    // TEST: Solid bright magenta
    outColor = vec4(1.0, 0.0, 1.0, 1.0);
}
