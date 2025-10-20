#version 450

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec3 fragViewDir;
layout(location = 2) in vec2 fragTexCoord;
layout(location = 3) in vec3 fragWorldPos;

layout(location = 0) out vec4 outColor;

layout(push_constant) uniform PushConstants {
    mat4 model;
    vec4 color;          // RGB color + alpha
    float fresnelPower;  // Controls edge glow intensity
    float scanlineSpeed; // Animation speed for scanlines
    float time;          // Current time for animation
    float _padding;
} push;

void main() {
    // Normalize inputs
    vec3 normal = normalize(fragNormal);
    vec3 viewDir = normalize(fragViewDir);

    // Fresnel effect (edges glow more than center)
    float fresnel = pow(1.0 - max(dot(normal, viewDir), 0.0), push.fresnelPower);

    // Animated scanlines for holographic effect
    float scanline = sin(fragWorldPos.y * 10.0 + push.time * push.scanlineSpeed) * 0.5 + 0.5;
    scanline = pow(scanline, 3.0); // Sharpen the scanlines

    // Pulse effect (subtle breathing)
    float pulse = sin(push.time * 2.0) * 0.1 + 0.9;

    // Combine effects
    float intensity = fresnel * pulse + scanline * 0.3;

    // Apply color and transparency
    vec3 finalColor = push.color.rgb * intensity;
    float alpha = push.color.a * (fresnel * 0.7 + 0.3); // More transparent at center

    // Add edge glow
    finalColor += push.color.rgb * fresnel * 0.5;

    outColor = vec4(finalColor, alpha);
}
