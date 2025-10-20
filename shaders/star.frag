#version 450

layout(location = 0) in vec3 fragWorldPos;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform StarUniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec3 viewPos;
    float time;
    vec3 starColor;
    float gamma;
    float scale;
    float exposure;
    float speed_hi;
    float speed_low;
    float zoom;
    float _padding;
} ubo;

// Hash function for noise
vec4 hash4(vec4 n) {
    return fract(sin(n)*1399763.5453123);
}

// 4D noise function
float noise4q(vec4 x) {
    vec4 n3 = vec4(0, 0.25, 0.5, 0.75);
    vec4 p2 = floor(x.wwww + n3);
    vec4 b = floor(x.xxxx + n3) + floor(x.yyyy + n3) * 157.0 + floor(x.zzzz + n3) * 113.0;
    vec4 p1 = b + fract(p2 * 0.00390625) * vec4(164352.0, -164352.0, 163840.0, -163840.0);
    p2 = b + fract((p2 + 1.0) * 0.00390625) * vec4(164352.0, -164352.0, 163840.0, -163840.0);
    vec4 f1 = fract(x.xxxx + n3);
    vec4 f2 = fract(x.yyyy + n3);
    f1 = f1 * f1 * (3.0 - 2.0 * f1);
    f2 = f2 * f2 * (3.0 - 2.0 * f2);

    vec4 n1 = vec4(0, 1.0, 157.0, 158.0);
    vec4 n2 = vec4(113.0, 114.0, 270.0, 271.0);
    vec4 vs1 = mix(hash4(p1), hash4(n1.yyyy + p1), f1);
    vec4 vs2 = mix(hash4(n1.zzzz + p1), hash4(n1.wwww + p1), f1);
    vec4 vs3 = mix(hash4(p2), hash4(n1.yyyy + p2), f1);
    vec4 vs4 = mix(hash4(n1.zzzz + p2), hash4(n1.wwww + p2), f1);
    vs1 = mix(vs1, vs2, f2);
    vs3 = mix(vs3, vs4, f2);
    vs2 = mix(hash4(n2.xxxx + p1), hash4(n2.yyyy + p1), f1);
    vs4 = mix(hash4(n2.zzzz + p1), hash4(n2.wwww + p1), f1);
    vs2 = mix(vs2, vs4, f2);
    vs4 = mix(hash4(n2.xxxx + p2), hash4(n2.yyyy + p2), f1);
    vec4 vs5 = mix(hash4(n2.zzzz + p2), hash4(n2.wwww + p2), f1);
    vs4 = mix(vs4, vs5, f2);
    f1 = fract(x.zzzz + n3);
    f2 = fract(x.wwww + n3);
    f1 = f1 * f1 * (3.0 - 2.0 * f1);
    f2 = f2 * f2 * (3.0 - 2.0 * f2);
    vs1 = mix(vs1, vs2, f1);
    vs3 = mix(vs3, vs4, f1);
    vs1 = mix(vs1, vs3, f2);
    float r = dot(vs1, vec4(0.25));
    return r * r * (3.0 - 2.0 * r);
}

// Body of star with multiple detail levels
float noiseSpere(vec3 surfacePos, float zoom, vec3 subnoise, float anim, float speedHi, float speedLow) {
    float s = 0.0;

    // Detail level 1: Base turbulence
    s = noise4q(vec4(surfacePos * zoom * 3.6864 + subnoise, anim * speedHi)) * 0.625;

    // Detail level 2: Medium detail
    s = s * 0.85 + noise4q(vec4(surfacePos * zoom * 61.44 + subnoise * 3.0, anim * speedHi * 3.0)) * 0.125;

    // Detail level 3: Fine detail
    s = s * 0.94 + noise4q(vec4(surfacePos * zoom * 307.2 + subnoise * 5.0, anim * 5.0)) * 0.0625;

    // Detail level 4: Very fine detail
    s = s * 0.98 + noise4q(vec4(surfacePos * zoom * 600.0 + subnoise * 6.0, anim * speedLow * 6.0)) * 0.03125;

    return s;
}

// Star corona rays
float ringRayNoise(vec3 surfaceDir, float dist, float anim, float speedRay, float speedRing) {
    vec3 pr = normalize(surfaceDir);

    // Base ray noise
    float nd = noise4q(vec4(pr * 1.0, -anim * speedRing + dist)) * 2.0;
    nd = pow(nd, 2.0);

    // Detailed rays
    float n = noise4q(vec4(pr * 10.0, -anim * speedRing + dist));

    // Ray streaks
    float ns = noise4q(vec4(pr * 50.0, -anim * speedRay + dist * 2.0)) * 2.0;

    // Extra detail
    ns = ns * 0.5 + noise4q(vec4(pr * 150.0, -anim * speedRay));

    n = pow(n, 2.0) * pow(nd, 1.0) * ns;

    return n;
}

void main() {
    // Get star center in world space
    vec3 starCenter = (ubo.model * vec4(0, 0, 0, 1)).xyz;

    // Surface position relative to star center
    vec3 surfacePos = fragWorldPos - starCenter;
    vec3 surfaceDir = normalize(surfacePos);

    // View direction
    vec3 viewDir = normalize(ubo.viewPos - fragWorldPos);

    // Distance from camera
    float dist = length(fragWorldPos - ubo.viewPos);

    // Animation time
    float animTime = ubo.time * 1.0;

    // === STAR SURFACE ===
    vec3 seed1 = vec3(45.78, 113.04, 28.957);
    vec3 seed2 = vec3(83.23, 34.34, 67.453);

    // Two layers of noise at different scales
    float s1 = noiseSpere(surfaceDir, ubo.zoom, seed1, animTime, ubo.speed_hi, ubo.speed_low);
    s1 = pow(min(s1 * 2.4, 1.0), 2.0);

    float s2 = noiseSpere(surfaceDir, ubo.zoom * 4.0, seed2, animTime, ubo.speed_hi * 1.5, ubo.speed_low * 2.0);
    s2 = min(s2 * 2.2, 1.0);

    // Color mixing
    vec3 lightColor = vec3(1.0, 1.0, 0.95);  // Bright white-yellow
    vec3 yellowColor = vec3(1.0, 0.85, 0.0); // Yellow
    vec3 orangeColor = vec3(1.0, 0.4, 0.0);  // Orange-red
    vec3 darkColor = vec3(0.8, 0.0, 0.2);    // Dark red-purple

    // Layer 1: Yellow to white gradient
    vec3 color = mix(yellowColor, lightColor, pow(s1, 60.0)) * s1;

    // Layer 2: Orange/red to white gradient
    color += mix(mix(orangeColor, darkColor, pow(s2, 2.0)), lightColor, pow(s2, 10.0)) * s2;

    // === CORONA RAYS ===
    float radius = 1.0;
    float distFromCenter = length(surfacePos);
    float rayString = 2.0;

    // Ray falloff from edge
    float rayFalloff = max(0.0, 1.0 - abs(radius - distFromCenter) / rayString);

    // Calculate ray intensity
    float rayIntensity = ringRayNoise(surfacePos, distFromCenter, animTime, 5.0, 2.0);

    // Combine ray falloff and intensity
    float rays = pow(rayFalloff, 4.0) + pow(rayFalloff, 2.0) * rayIntensity;

    // Ray colors
    vec3 rayColor = vec3(1.0, 0.6, 0.1);      // Orange rays
    vec3 rayLightColor = vec3(1.0, 0.95, 1.0); // White-pink rays

    // Add rays to color
    color += mix(rayColor, rayLightColor, pow(rays, 3.0)) * rays;

    // === EDGE GLOW ===
    float edge = 1.0 - abs(dot(surfaceDir, viewDir));
    edge = pow(edge, 3.0);
    color += vec3(1.0, 0.8, 0.4) * edge * 0.3;

    // === FINAL OUTPUT ===
    vec3 finalColor = color;

    // Apply exposure
    finalColor *= ubo.exposure * 0.8;

    // Tone mapping
    finalColor = finalColor / (finalColor + vec3(1.0));

    // Gamma correction
    finalColor = pow(finalColor, vec3(1.0 / ubo.gamma));

    // Apply star color tint
    finalColor *= ubo.starColor;

    // Clamp
    finalColor = clamp(finalColor, 0.0, 1.0);

    outColor = vec4(finalColor, 1.0);
}
