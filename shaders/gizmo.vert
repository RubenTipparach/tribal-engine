#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(push_constant) uniform PushConstants {
    int hoveredAxis; // 0=none, 1=X, 2=Y, 3=Z
} push;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUV;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out float fragHighlight;

void main() {
    // Use the normal as color (X=red, Y=green, Z=blue)
    vec3 baseColor = abs(inNormal);
    fragColor = baseColor;

    // Check if this vertex's normal matches the hovered axis
    float highlight = 0.0;
    if (push.hoveredAxis == 1 && baseColor.r > 0.9) { // X axis (red)
        highlight = 1.0;
    } else if (push.hoveredAxis == 2 && baseColor.g > 0.9) { // Y axis (green)
        highlight = 1.0;
    } else if (push.hoveredAxis == 3 && baseColor.b > 0.9) { // Z axis (blue)
        highlight = 1.0;
    }
    fragHighlight = highlight;

    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(inPosition, 1.0);
}
