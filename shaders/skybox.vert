#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec3 viewPos;
} ubo;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUV;

layout(location = 0) out vec3 fragPosition;
layout(location = 1) out vec3 fragNormal;

void main() {
    // Position skybox at camera location so it moves with the camera
    vec3 worldPos = inPosition + ubo.viewPos;
    gl_Position = ubo.proj * ubo.view * vec4(worldPos, 1.0);

    // Output for fragment shader
    fragPosition = inPosition; // Use local position for star field lookup
    fragNormal = inNormal;
}
