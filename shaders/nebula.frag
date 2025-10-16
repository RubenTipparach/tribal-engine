#version 450

layout(location = 0) in vec2 inUV;
layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec3 viewPos;
    float time;
    vec2 resolution;
    vec2 mouse;
    float zoom;
    float density;
    float brightness;
    float scale;

    // Color parameters
    vec3 color_center;
    float _padding1;
    vec3 color_edge;
    float _padding2;
    vec3 color_density_low;
    float _padding3;
    vec3 color_density_high;
    float _padding4;

    // Light parameters
    vec3 light_color;
    float light_intensity;
} ubo;

layout(binding = 1) uniform sampler2D depthTexture;

// Port of "Dusty nebula 4" by Duke
// https://www.shadertoy.com/view/MsVXWW

// #define ROTATION  // Disabled - nebula is now fixed in space for camera orbit
#define DITHERING
#define BACKGROUND

#define pi 3.14159265

// 2D rotation helper
void rotate(inout vec2 p, float a) {
    p = cos(a) * p + sin(a) * vec2(p.y, -p.x);
}

// Simple hash-based noise (replacement for Shadertoy texture noise)
float hash(vec3 p) {
    p = fract(p * vec3(0.1031, 0.1030, 0.0973));
    p += dot(p, p.yxz + 33.33);
    return fract((p.x + p.y) * p.z);
}

float noise(vec3 x) {
    vec3 p = floor(x);
    vec3 f = fract(x);
    f = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(mix(hash(p + vec3(0, 0, 0)), hash(p + vec3(1, 0, 0)), f.x),
            mix(hash(p + vec3(0, 1, 0)), hash(p + vec3(1, 1, 0)), f.x), f.y),
        mix(mix(hash(p + vec3(0, 0, 1)), hash(p + vec3(1, 0, 1)), f.x),
            mix(hash(p + vec3(0, 1, 1)), hash(p + vec3(1, 1, 1)), f.x), f.y),
        f.z);
}

float rand(vec2 co) {
    return fract(sin(dot(co * 0.123, vec2(12.9898, 78.233))) * 43758.5453);
}

// Spiral noise from otaviogood
const float nudge = 0.739513;
float normalizer = 1.0 / sqrt(1.0 + nudge * nudge);

float SpiralNoiseC(vec3 p) {
    float n = 0.0;
    float iter = 1.0;
    for (int i = 0; i < 8; i++) {
        n += -abs(sin(p.y * iter) + cos(p.x * iter)) / iter;
        p.xy += vec2(p.y, -p.x) * nudge;
        p.xy *= normalizer;
        p.xz += vec2(p.z, -p.x) * nudge;
        p.xz *= normalizer;
        iter *= 1.733733;
    }
    return n;
}

float SpiralNoise3D(vec3 p) {
    float n = 0.0;
    float iter = 1.0;
    for (int i = 0; i < 5; i++) {
        n += (sin(p.y * iter) + cos(p.x * iter)) / iter;
        p.xz += vec2(p.z, -p.x) * nudge;
        p.xz *= normalizer;
        iter *= 1.33733;
    }
    return n;
}

float NebulaNoise(vec3 p) {
    float final = p.y + 4.5;
    final -= SpiralNoiseC(p.xyz);
    final += SpiralNoiseC(p.zxy * 0.5123 + 100.0) * 4.0;
    final -= SpiralNoise3D(p);
    return final;
}

float map(vec3 p) {
    #ifdef ROTATION
    rotate(p.xz, ubo.mouse.x * 0.008 * pi + ubo.time * 0.1);
    #endif

    float NebNoise = abs(NebulaNoise(p / 0.5) * 0.5);
    return NebNoise + 0.03;
}

vec3 computeColor(float density, float radius) {
    // Mix based on density using configurable colors
    vec3 result = mix(ubo.color_density_low, ubo.color_density_high, density);

    // Mix based on radius using configurable edge/center colors
    result *= mix(ubo.color_center, ubo.color_edge, min((radius + 0.05) / 0.9, 1.15));

    return result * ubo.brightness;
}

bool RaySphereIntersect(vec3 org, vec3 dir, out float near, out float far) {
    float b = dot(dir, org);
    float c = dot(org, org) - 8.0;
    float delta = b * b - c;
    if (delta < 0.0)
        return false;
    float deltasqrt = sqrt(delta);
    near = -b - deltasqrt;
    far = -b + deltasqrt;
    return far > 0.0;
}

void main() {
    // Compute NDC coordinates from UV
    vec2 ndc = inUV * 2.0 - 1.0;

    // Compute inverse view-projection matrix
    mat4 invViewProj = inverse(ubo.proj * ubo.view);

    // Ray origin is camera position
    vec3 ro = ubo.viewPos;

    // Compute far point in world space
    vec4 farPoint = invViewProj * vec4(ndc, 1.0, 1.0);
    farPoint /= farPoint.w;

    // Ray direction from camera to far point
    vec3 rd = normalize(farPoint.xyz - ro);

    // Apply scale by moving camera position away from origin
    ro /= ubo.scale;

    // Apply zoom by moving camera along view direction
    ro += rd * ubo.zoom * 1.6;

    #ifdef DITHERING
    vec2 seed = inUV + fract(ubo.time);
    #endif

    // Sample depth buffer once before raymarch
    float scene_depth = texture(depthTexture, inUV).r;

    float ld = 0.0, td = 0.0, w = 0.0;
    float d = 1.0, t = 0.0;
    const float h = 0.1;
    vec4 sum = vec4(0.0);

    float min_dist = 0.0, max_dist = 0.0;

    if (RaySphereIntersect(ro, rd, min_dist, max_dist)) {
        t = min_dist * step(t, min_dist);

        // Raymarch loop - sample the volumetric nebula
        for (int i = 0; i < 56; i++) {
            vec3 pos = ro + t * rd;

            // Check if we've reached geometry depth
            // Linearize both depths for comparison
            // We need to convert raymarch distance to depth buffer value
            // Project current raymarch position to clip space
            vec4 clipPos = ubo.proj * ubo.view * vec4(pos * ubo.scale, 1.0);
            float raymarch_depth = clipPos.z / clipPos.w;

            // If we've passed the geometry, stop raymarching
            if (raymarch_depth >= scene_depth) {
                break;
            }

            if (td > 0.9 * ubo.density || d < 0.1 * t || t > 10.0 || sum.a > 0.99 || t > max_dist)
                break;

            float d = map(pos);
            d = max(d, 0.08);

            // Point light
            vec3 ldst = vec3(0.0) - pos;
            float lDist = max(length(ldst), 0.001);

            sum.rgb += (ubo.light_color / (lDist * lDist) * ubo.light_intensity);

            if (d < h) {
                ld = h - d;
                w = (1.0 - td) * ld;
                td += w + 1.0 / 200.0;

                vec4 col = vec4(computeColor(td, lDist), td);
                col.a *= 0.185 * ubo.density;
                col.rgb *= col.a;
                sum = sum + col * (1.0 - sum.a);
            }

            td += 1.0 / 70.0;
            d = max(d, 0.04);

            #ifdef DITHERING
            d = abs(d) * (0.8 + 0.2 * rand(seed * vec2(float(i))));
            #endif

            t += max(d * 0.1 * max(min(length(ldst), length(ro)), 1.0), 0.02);
        }

        sum *= 1.0 / exp(ld * 0.2) * 0.6;
        sum = clamp(sum, 0.0, 1.0);
        sum.xyz = sum.xyz * sum.xyz * (3.0 - 2.0 * sum.xyz);
    }

    #ifdef BACKGROUND
    if (td < 0.8) {
        vec3 stars = vec3(noise(rd * 500.0) * 0.5 + 0.5);
        vec3 starbg = vec3(0.0);
        starbg = mix(starbg, vec3(0.8, 0.9, 1.0),
                    smoothstep(0.99, 1.0, stars) * clamp(dot(vec3(0.0), rd) + 0.75, 0.0, 1.0));
        starbg = clamp(starbg, 0.0, 1.0);
        sum.xyz += starbg;
    }
    #endif

    outColor = vec4(sum.xyz, sum.a);
}
