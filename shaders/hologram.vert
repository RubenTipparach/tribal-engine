#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) out vec3 fragNormal;
layout(location = 1) out vec3 fragViewDir;
layout(location = 2) out vec2 fragTexCoord;
layout(location = 3) out vec3 fragWorldPos;

layout(push_constant) uniform PushConstants {
    mat4 model;
    vec4 color;          // RGB color + alpha
    float fresnelPower;  // Controls edge glow intensity
    float scanlineSpeed; // Animation speed for scanlines
    float time;          // Current time for animation
    float _padding;
} push;

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    vec3 viewPos;
    float time;
    // ... other UBO fields
} ubo;

void main() {
    vec4 worldPos = push.model * vec4(inPosition, 1.0);
    fragWorldPos = worldPos.xyz;

    gl_Position = ubo.proj * ubo.view * worldPos;

    // Transform normal to world space
    mat3 normalMatrix = transpose(inverse(mat3(push.model)));
    fragNormal = normalize(normalMatrix * inNormal);

    // Calculate view direction
    fragViewDir = normalize(ubo.viewPos - worldPos.xyz);

    fragTexCoord = inTexCoord;
}
