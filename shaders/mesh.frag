#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    vec3 viewPos;
    vec3 dirLightDirection;
    vec3 dirLightColor;
    float dirLightIntensity;
    vec3 dirLightShadowColor;
    float starDensity;
    float starBrightness;
    float pad0;
    float pad1;
    float pad2;
    vec3 nebulaPrimaryColor;
    float nebulaIntensity;
    vec3 nebulaSecondaryColor;
    float backgroundBrightness;
    uint pointLightCount;
    uint ssaoEnabled;
} ubo;

// SSAO texture (blurred ambient occlusion)
layout(binding = 1) uniform sampler2D ssaoTexture;

// Material properties via push constants (after mat4 model at offset 64)
layout(push_constant) uniform MaterialPushConstants {
    layout(offset = 64) vec3 albedo;
    layout(offset = 76) float metallic;
    layout(offset = 80) float roughness;
    layout(offset = 84) float ambient_strength;
    layout(offset = 88) float gi_strength;
} material;

layout(location = 0) in vec3 fragPosition;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragUV;
layout(location = 3) in vec3 viewPos;

layout(location = 0) out vec4 outColor;

const float PI = 3.14159265359;

// Hash function for procedural skybox (matches skybox shader)
vec3 hash3(vec3 p) {
    p = vec3(
        dot(p, vec3(127.1, 311.7, 74.7)),
        dot(p, vec3(269.5, 183.3, 246.1)),
        dot(p, vec3(113.5, 271.9, 124.6))
    );
    return fract(sin(p) * 43758.5453123);
}

// Voronoi noise for star field
vec4 voronoi(vec3 p, float scale) {
    vec3 n = floor(p * scale);
    vec3 f = fract(p * scale);

    float minDist = 10.0;
    vec3 cellColor = vec3(1.0);

    for (int k = -1; k <= 1; k++) {
        for (int j = -1; j <= 1; j++) {
            for (int i = -1; i <= 1; i++) {
                vec3 neighbor = vec3(float(i), float(j), float(k));
                vec3 point = hash3(n + neighbor);
                vec3 diff = neighbor + point - f;
                float dist = length(diff);

                if (dist < minDist) {
                    minDist = dist;
                    cellColor = hash3(n + neighbor + vec3(12.34, 56.78, 90.12));
                }
            }
        }
    }

    return vec4(cellColor, minDist);
}

// Sample procedural skybox environment (simplified for GI)
vec3 sampleSkybox(vec3 dir) {
    // Simplified star field (single layer for performance)
    float densityScale = 1.0 / max(ubo.starDensity, 0.1);
    vec4 voronoi1 = voronoi(dir, 40.0 * ubo.starDensity);
    float star1 = pow(max(0.0, 1.0 - voronoi1.w * (25.0 * densityScale)), 10.0);
    vec3 stars = vec3(0.0);
    if (star1 > 0.01) {
        vec3 starColor = mix(vec3(0.8, 0.9, 1.0), vec3(1.0, 0.9, 0.8), voronoi1.x);
        stars = starColor * star1 * 2.0 * ubo.starBrightness;
    }

    // Nebula contribution
    vec3 spaceColor = vec3(ubo.backgroundBrightness);
    float nebulaFactor = abs(dir.y) * 0.3 + 0.2;
    vec3 nebulaColor = mix(
        ubo.nebulaPrimaryColor,
        ubo.nebulaSecondaryColor,
        (dir.x * 0.5 + 0.5)
    );
    spaceColor += nebulaColor * nebulaFactor * ubo.nebulaIntensity;

    return spaceColor + stars;
}

// PBR functions
float DistributionGGX(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float nom = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float nom = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}


vec3 calculateLight(vec3 N, vec3 V, vec3 L, vec3 lightColor, float lightIntensity, vec3 F0, vec3 albedo, float metallic, float roughness) {
    vec3 H = normalize(V + L);

    // Cook-Torrance BRDF
    float NDF = DistributionGGX(N, H, roughness);
    float G = GeometrySmith(N, V, L, roughness);
    vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);

    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
    vec3 specular = numerator / denominator;

    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - metallic;

    float NdotL = max(dot(N, L), 0.0);

    return (kD * albedo / PI + specular) * lightColor * lightIntensity * NdotL;
}

void main() {
    vec3 N = normalize(fragNormal);
    vec3 V = normalize(viewPos - fragPosition);

    // Base reflectivity (F0)
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, material.albedo, material.metallic);

    vec3 Lo = vec3(0.0);

    // Directional light
    vec3 L = normalize(-ubo.dirLightDirection);
    Lo += calculateLight(N, V, L, ubo.dirLightColor, ubo.dirLightIntensity, F0, material.albedo, material.metallic, material.roughness);

    // TODO: Add point lights (will need separate uniform buffer or storage buffer)

    // Global Illumination: Sample skybox environment based on surface normal
    vec3 giColor = vec3(0.0);
    if (material.gi_strength > 0.001) {
        // Sample skybox in the direction of the surface normal
        vec3 skyboxSample = sampleSkybox(N);

        // Mix between shadow color and skybox color based on light direction
        float NdotL = dot(N, normalize(-ubo.dirLightDirection));
        vec3 indirectLight = mix(ubo.dirLightShadowColor, skyboxSample, max(NdotL * 0.5 + 0.5, 0.0));

        giColor = indirectLight * material.albedo * material.gi_strength;
    }

    // Sample SSAO from screen-space coordinates (only if enabled)
    float ssaoValue = 1.0;
    if (ubo.ssaoEnabled != 0u) {
        vec2 screenUV = gl_FragCoord.xy / vec2(textureSize(ssaoTexture, 0));
        ssaoValue = texture(ssaoTexture, screenUV).r;
    }

    // Ambient lighting term (simple constant ambient)
    vec3 ambient = material.albedo * material.ambient_strength * 0.03;

    // Apply SSAO to ambient and GI terms (darker crevices get less indirect light)
    ambient *= ssaoValue;
    giColor *= ssaoValue;

    vec3 color = ambient + Lo + giColor;

    // HDR tonemapping and gamma correction
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));

    outColor = vec4(color, 1.0);
}
