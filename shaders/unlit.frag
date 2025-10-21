#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    vec3 viewPos;
    float time;
} ubo;

// Color, fresnel power, scanline speed via push constants (after mat4 model at offset 64)
layout(push_constant) uniform PushConstants {
    layout(offset = 64) vec4 color;
    layout(offset = 80) float fresnelPower;
    layout(offset = 84) float scanlineSpeed;
} push;

layout(location = 0) in vec3 fragPosition;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragUV;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 N = normalize(fragNormal);
    vec3 V = normalize(ubo.viewPos - fragPosition);

    // Fresnel effect (edge glow)
    float fresnel = pow(1.0 - max(dot(N, V), 0.0), push.fresnelPower);

    // Animated scanlines
    float scanline = sin(fragPosition.y * 20.0 + ubo.time * push.scanlineSpeed) * 0.5 + 0.5;
    scanline = smoothstep(0.3, 0.7, scanline);

    // Combine effects
    vec3 hologramColor = push.color.rgb * (0.3 + fresnel * 0.7 + scanline * 0.2);
    float alpha = push.color.a * (0.4 + fresnel * 0.6);

    outColor = vec4(hologramColor, alpha);
}
