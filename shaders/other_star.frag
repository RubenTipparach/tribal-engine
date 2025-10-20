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

// 2D hash for noise texture replacement
vec2 hash2(vec2 p) {
    p = vec2(dot(p, vec2(127.1, 311.7)),
             dot(p, vec2(269.5, 183.3)));
    return fract(sin(p) * 43758.5453123);
}

// IQ's noise - simulates texture(iChannel0, uv).yx
float pn(in vec3 p) {
    vec3 ip = floor(p);
    vec3 fp = fract(p);
    fp = fp * fp * (3.0 - 2.0 * fp);

    // Simulate 2D noise texture lookup
    vec2 uv = (ip.xy + vec2(37.0, 17.0) * ip.z) + fp.xy;
    vec2 noise2d = hash2((uv + 0.5) / 256.0);

    return mix(noise2d.x, noise2d.y, fp.z);
}

// FBM
float fpn(vec3 p) {
    return pn(p * 0.06125) * 0.57 + pn(p * 0.125) * 0.28 + pn(p * 0.25) * 0.15;
}

// Rotation matrix
mat2 rot2D(float angle) {
    float cs = cos(angle);
    float si = sin(angle);
    return mat2(cs, si, -si, cs);
}

// Spikeball distance function - MODIFIED to be less spiky
float spikeball(vec3 p) {
    // Ball
    float d = length(p) - 1.6;

    // Spikes - reduced influence by using lower power
    p = normalize(p);
    vec4 b = max(max(max(
        abs(vec4(dot(p, vec3(0.526, 0.000, 0.851)), dot(p, vec3(-0.526, 0.000, 0.851)), dot(p, vec3(0.851, 0.526, 0.000)), dot(p, vec3(-0.851, 0.526, 0.000)))),
        abs(vec4(dot(p, vec3(0.357, 0.934, 0.000)), dot(p, vec3(-0.357, 0.934, 0.000)), dot(p, vec3(0.000, 0.851, 0.526)), dot(p, vec3(0.000, -0.851, 0.526))))),
        abs(vec4(dot(p, vec3(0.000, 0.357, 0.934)), dot(p, vec3(0.000, -0.357, 0.934)), dot(p, vec3(0.934, 0.000, 0.357)), dot(p, vec3(-0.934, 0.000, 0.357))))),
        abs(vec4(dot(p, vec3(0.577, 0.577, 0.577)), dot(p, vec3(-0.577, 0.577, 0.577)), dot(p, vec3(0.577, -0.577, 0.577)), dot(p, vec3(0.577, 0.577, -0.577)))));

    b.xy = max(b.xy, b.zw);
    // REDUCED spike sharpness from 64 to 16, and reduced amplitude
    b.x = pow(max(b.x, b.y), 16.0);

    return d - exp2(b.x * (sin(ubo.time) * 0.15 + 0.4)) * 0.3;
}

// Distance function
float map(vec3 p) {
    mat2 rM = rot2D(ubo.time * ubo.speed_low);
    p.xy = p.xy * rM;
    p.xz = p.xz * rM;

    return spikeball(p) + fpn(p * 50.0 + ubo.time * ubo.speed_hi * 3.0) * 0.5;
}

// Fire palette
vec3 firePalette(float i) {
    float T = 1400.0 + 1300.0 * i; // Temperature range (in Kelvin)
    vec3 L = vec3(7.4, 5.6, 4.4); // Red, green, blue wavelengths
    L = pow(L, vec3(5.0)) * (exp(1.43876719683e5 / (T * L)) - 1.0);
    return 1.0 - exp(-5e8 / L);
}

void main() {
    // Ray direction from camera to fragment
    vec3 rayDir = normalize(fragWorldPos - ubo.viewPos);

    // Ray origin (camera position)
    vec3 rayOrigin = ubo.viewPos;

    // Transform to local space (sphere center is at fragWorldPos origin)
    vec3 sphereCenter = (ubo.model * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
    vec3 ro = (rayOrigin - sphereCenter) * 0.02; // Scale down for raymarch
    vec3 rd = rayDir;

    // Raymarch variables
    float ld = 0.0, td = 0.0, w;
    float d = 1.0, t = 0.0;
    const float h = 0.1;
    vec3 tc = vec3(0.0);

    // Raymarch loop
    for (int i = 0; i < 64; i++) {
        if (td > (1.0 - 1.0/200.0) || d < 0.001 * t || t > 12.0) break;

        d = map(ro + t * rd);

        ld = (h - d) * step(d, h);
        w = (1.0 - td) * ld;

        tc += w * w + 1.0/50.0;
        td += w + 1.0/200.0;

        d = max(d, 0.04);
        t += d * 0.5;
    }

    // Apply fire palette
    vec3 color = firePalette(tc.x);

    // Apply user color tint
    color *= ubo.starColor;

    // Exposure and gamma correction
    color *= ubo.exposure * 0.01;
    color = pow(color, vec3(1.0 / ubo.gamma));

    outColor = vec4(color, 1.0);
}
