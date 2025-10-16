#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec3 viewPos;
    float starDensity;
    float starBrightness;
    float padding1;
    float padding2;
    vec3 nebulaPrimaryColor;
    float nebulaIntensity;
    vec3 nebulaSecondaryColor;
    float backgroundBrightness;
} ubo;

layout(location = 0) in vec3 fragPosition;
layout(location = 1) in vec3 fragNormal;

layout(location = 0) out vec4 outColor;

// Hash function for randomness
vec2 hash2(vec2 p) {
    p = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
    return fract(sin(p) * 43758.5453);
}

// 3D hash for better star distribution
vec3 hash3(vec3 p) {
    p = vec3(
        dot(p, vec3(127.1, 311.7, 74.7)),
        dot(p, vec3(269.5, 183.3, 246.1)),
        dot(p, vec3(113.5, 271.9, 124.6))
    );
    return fract(sin(p) * 43758.5453123);
}

// Voronoi noise - returns distance to nearest point and cell color
vec4 voronoi(vec3 p, float scale) {
    vec3 n = floor(p * scale);
    vec3 f = fract(p * scale);

    float minDist = 10.0;
    vec3 minPoint = vec3(0.0);
    vec3 cellColor = vec3(1.0);

    // Check neighboring cells
    for (int k = -1; k <= 1; k++) {
        for (int j = -1; j <= 1; j++) {
            for (int i = -1; i <= 1; i++) {
                vec3 neighbor = vec3(float(i), float(j), float(k));
                vec3 point = hash3(n + neighbor);
                vec3 diff = neighbor + point - f;
                float dist = length(diff);

                if (dist < minDist) {
                    minDist = dist;
                    minPoint = point;
                    cellColor = hash3(n + neighbor + vec3(12.34, 56.78, 90.12));
                }
            }
        }
    }

    return vec4(cellColor, minDist);
}

// Generate star field with multiple voronoi layers
vec3 starField(vec3 dir, float density, float brightness) {
    vec3 color = vec3(0.0);

    // Adjust scales based on density
    float densityScale = 1.0 / max(density, 0.1);

    // Layer 1: Small, frequent stars
    vec4 voronoi1 = voronoi(dir, 80.0 * density);
    float star1 = pow(max(0.0, 1.0 - voronoi1.w * (30.0 * densityScale)), 12.0);
    if (star1 > 0.01) {
        vec3 starColor1 = mix(vec3(0.8, 0.9, 1.0), vec3(1.0, 0.9, 0.8), voronoi1.x);
        color += starColor1 * star1 * 1.5 * brightness;
    }

    // Layer 2: Medium stars with color variation
    vec4 voronoi2 = voronoi(dir, 40.0 * density);
    float star2 = pow(max(0.0, 1.0 - voronoi2.w * (25.0 * densityScale)), 10.0);
    if (star2 > 0.01) {
        vec3 starColor2 = mix(
            vec3(0.6, 0.8, 1.0),  // Blue stars
            vec3(1.0, 0.7, 0.5),  // Orange stars
            voronoi2.y
        );
        color += starColor2 * star2 * 2.0 * brightness;
    }

    // Layer 3: Large, bright stars (rare)
    vec4 voronoi3 = voronoi(dir, 20.0 * density);
    float star3 = pow(max(0.0, 1.0 - voronoi3.w * (20.0 * densityScale)), 8.0);
    if (star3 > 0.01) {
        // More varied colors for large stars
        vec3 starColor3 = vec3(
            0.7 + voronoi3.x * 0.3,
            0.7 + voronoi3.y * 0.3,
            0.8 + voronoi3.z * 0.2
        );
        color += starColor3 * star3 * 3.0 * brightness;
    }

    // Layer 4: Tiny, very frequent background stars
    vec4 voronoi4 = voronoi(dir, 150.0 * density);
    float star4 = pow(max(0.0, 1.0 - voronoi4.w * (40.0 * densityScale)), 20.0);
    if (star4 > 0.01) {
        color += vec3(0.9, 0.95, 1.0) * star4 * 0.8 * brightness;
    }

    return color;
}

void main() {
    // Normalize direction for star field lookup
    vec3 dir = normalize(fragPosition);

    // Generate star field with configurable parameters
    vec3 stars = starField(dir, ubo.starDensity, ubo.starBrightness);

    // Background color with configurable brightness
    vec3 spaceColor = vec3(ubo.backgroundBrightness);

    // Add nebula colors based on direction
    float nebulaFactor = abs(dir.y) * 0.3 + 0.2;
    vec3 nebulaColor = mix(
        ubo.nebulaPrimaryColor,
        ubo.nebulaSecondaryColor,
        (dir.x * 0.5 + 0.5)
    );
    spaceColor += nebulaColor * nebulaFactor * ubo.nebulaIntensity;

    vec3 finalColor = spaceColor + stars;

    outColor = vec4(finalColor, 1.0);
}
