#version 450

// Push constants for per-object model matrix
layout(push_constant) uniform PushConstants {
    mat4 model;
} push;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    vec3 viewPos;
    vec3 dirLightDirection;
    vec3 dirLightColor;
    float dirLightIntensity;
    uint pointLightCount;
} ubo;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUV;

layout(location = 0) out vec3 fragPosition;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out vec2 fragUV;
layout(location = 3) out vec3 viewPos;

void main() {
    vec4 worldPosition = push.model * vec4(inPosition, 1.0);
    fragPosition = worldPosition.xyz;
    fragNormal = mat3(transpose(inverse(push.model))) * inNormal;
    fragUV = inUV;
    viewPos = ubo.viewPos;

    gl_Position = ubo.proj * ubo.view * worldPosition;
}
