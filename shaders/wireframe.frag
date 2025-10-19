#version 450

layout(location = 0) in vec3 fragNormal;

layout(location = 0) out vec4 outColor;

void main() {
    // Simple wireframe color with slight normal-based variation
    vec3 N = normalize(fragNormal);
    float intensity = abs(N.y) * 0.3 + 0.7; // Vary brightness based on normal

    // Yellow wireframe color
    vec3 wireColor = vec3(1.0, 1.0, 0.0) * intensity;
    outColor = vec4(wireColor, 1.0);
}
