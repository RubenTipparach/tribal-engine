#version 450

// Simplified SSAO Fragment Shader
// Computes screen-space ambient occlusion from depth only with normal reconstruction

layout(binding = 0) uniform SSAOUniformBufferObject {
    mat4 proj;
    float ssaoRadius;
    float ssaoBias;
    float ssaoPower;
    uint ssaoKernelSize;
} ubo;

layout(binding = 1) uniform sampler2D depthTexture;

layout(location = 0) in vec2 fragTexCoord;
layout(location = 0) out float outAO;

// Linearize depth value
float linearizeDepth(float depth) {
    float near = 0.1;  // Must match your camera near plane
    float far = 100.0; // Must match your camera far plane
    float z = depth * 2.0 - 1.0; // Back to NDC
    return (2.0 * near * far) / (far + near - z * (far - near));
}

// Reconstruct view-space position from depth using inverse projection
vec3 reconstructViewPos(vec2 uv, float depth) {
    // Convert UV to NDC
    float x = uv.x * 2.0 - 1.0;
    float y = (1.0 - uv.y) * 2.0 - 1.0;  // Flip Y for Vulkan

    // Create clip-space position
    vec4 clipPos = vec4(x, y, depth, 1.0);

    // Transform to view space using inverse projection
    vec4 viewPos = inverse(ubo.proj) * clipPos;

    // Perspective divide
    return viewPos.xyz / viewPos.w;
}

// Reconstruct normal from depth using screen-space derivatives
vec3 reconstructNormal(vec2 uv) {
    vec2 texelSize = 1.0 / textureSize(depthTexture, 0);

    float depth = texture(depthTexture, uv).r;
    float depthRight = texture(depthTexture, uv + vec2(texelSize.x, 0.0)).r;
    float depthUp = texture(depthTexture, uv + vec2(0.0, texelSize.y)).r;

    vec3 pos = reconstructViewPos(uv, depth);
    vec3 posRight = reconstructViewPos(uv + vec2(texelSize.x, 0.0), depthRight);
    vec3 posUp = reconstructViewPos(uv + vec2(0.0, texelSize.y), depthUp);

    vec3 dx = posRight - pos;
    vec3 dy = posUp - pos;

    return normalize(cross(dx, dy));
}

// Simple hash for pseudo-random rotation
float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
}

// Kernel sample with spiral pattern
vec3 getKernelSample(int index, uint kernelSize) {
    float angle = float(index) * 2.399; // Better distribution
    float height = (float(index) + 0.5) / float(kernelSize);
    float radius = sqrt(height); // More samples close to center

    return vec3(
        cos(angle) * radius,
        sin(angle) * radius,
        height
    );
}

void main() {
    vec2 texCoord = fragTexCoord;
    float depth = texture(depthTexture, texCoord).r;

    // Early out for skybox/far plane
    if (depth >= 0.9999) {
        outAO = 1.0;
        return;
    }

    // Reconstruct position and normal from depth
    vec3 fragPos = reconstructViewPos(texCoord, depth);
    vec3 normal = reconstructNormal(texCoord);

    // Better noise: use larger tile size to reduce visible pattern
    vec2 noiseScale = vec2(textureSize(depthTexture, 0)) / 16.0; // 16x16 tiles
    float randomAngle = hash(floor(texCoord * noiseScale)) * 6.28318;
    vec3 randomVec = vec3(cos(randomAngle), sin(randomAngle), 0.0);

    // Create TBN matrix
    vec3 tangent = normalize(randomVec - normal * dot(randomVec, normal));
    vec3 bitangent = cross(normal, tangent);
    mat3 TBN = mat3(tangent, bitangent, normal);

    // Sample kernel
    float occlusion = 0.0;
    int sampleCount = 0;

    for (int i = 0; i < int(ubo.ssaoKernelSize); ++i) {
        // Get sample in tangent space hemisphere
        vec3 sampleTangent = getKernelSample(i, ubo.ssaoKernelSize);
        vec3 sampleView = TBN * sampleTangent;
        vec3 samplePos = fragPos + sampleView * ubo.ssaoRadius;

        // Convert view-space position into clip-space
        vec4 offset = vec4(samplePos, 1.0);
        offset = ubo.proj * offset;
        offset.xy /= offset.w;
        offset.xy = offset.xy * 0.5 + 0.5;
        offset.y = 1.0 - offset.y;  // Flip Y for Vulkan

        // Bounds check
        if (offset.x < 0.0 || offset.x > 1.0 || offset.y < 0.0 || offset.y > 1.0) {
            continue;
        }

        // Reconstruct view-space position at this sample location
        float sampleDepth = texture(depthTexture, offset.xy).r;
        if (sampleDepth >= 0.9999) {
            continue; // Skip skybox samples
        }

        vec3 reconstructedPos = reconstructViewPos(offset.xy, sampleDepth);
        vec3 sampledNormal = reconstructNormal(offset.xy);

        // Check if sampled normal is similar to our normal (facing same direction)
        // Skip samples that are facing away - they shouldn't contribute
        if (dot(sampledNormal, normal) > 0.99) {
            sampleCount++;
        }
        else {
            // Range check to avoid dark halos around objects
            float rangeCheck = smoothstep(0.0, 1.0, ubo.ssaoRadius / abs(reconstructedPos.z - samplePos.z - ubo.ssaoBias));

            // Count as occluder if reconstructed position is closer to camera (>= in view space where Z is negative)
            occlusion += (reconstructedPos.z >= samplePos.z - ubo.ssaoBias ? 1.0 : 0.0) * rangeCheck;
            sampleCount++;
        }
    }

    // Normalize and invert (1.0 = no occlusion, 0.0 = full occlusion)
    occlusion = 1.0 - (occlusion / float(max(sampleCount, 1)));

    // Apply power for contrast
    occlusion = pow(clamp(occlusion, 0.0, 1.0), ubo.ssaoPower);

    outAO = occlusion;
}
