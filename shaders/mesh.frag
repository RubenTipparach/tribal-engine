#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    vec3 viewPos;
    vec3 dirLightDirection;
    vec3 dirLightColor;
    float dirLightIntensity;
    uint pointLightCount;
} ubo;

// Material properties via push constants (after mat4 model at offset 64)
layout(push_constant) uniform MaterialPushConstants {
    layout(offset = 64) vec3 albedo;
    layout(offset = 76) float metallic;
    layout(offset = 80) float roughness;
    layout(offset = 84) float ao_intensity;
} material;

layout(location = 0) in vec3 fragPosition;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragUV;
layout(location = 3) in vec3 viewPos;

layout(location = 0) out vec4 outColor;

const float PI = 3.14159265359;

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

// Simple vertex-based ambient occlusion approximation
// Uses the idea that concave areas (facing away from view) are more occluded
float calculateSimpleAO(vec3 N, vec3 V) {
    float NdotV = max(dot(N, V), 0.0);
    // Areas facing away from camera get more occlusion
    float ao = mix(0.3, 1.0, NdotV);
    // Add some vertex-based cavity darkening
    float cavity = pow(NdotV, 2.0);
    return mix(ao, 1.0, cavity);
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

    // Calculate ambient occlusion
    float ao = calculateSimpleAO(N, V) * material.ao_intensity;

    // Ambient term with AO
    vec3 ambient = vec3(0.03) * material.albedo * ao;
    vec3 color = ambient + Lo;

    // HDR tonemapping and gamma correction
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));

    outColor = vec4(color, 1.0);
}
