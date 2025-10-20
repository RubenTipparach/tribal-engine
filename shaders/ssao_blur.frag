#version 450

// SSAO Separable Bilateral Blur Fragment Shader
// Two-pass edge-preserving blur using depth to avoid blurring across edges
// Prevents blurring across large depth discontinuities

layout(binding = 0) uniform sampler2D ssaoInput;
layout(binding = 1) uniform sampler2D depthTexture;  // For edge detection

// Push constant to control blur direction (0 = horizontal, 1 = vertical)
layout(push_constant) uniform BlurDirection {
    int direction; // 0 = horizontal, 1 = vertical
} blur;

layout(location = 0) in vec2 fragTexCoord;
layout(location = 0) out float outAO;

void main() {
    vec2 texelSize = 1.0 / vec2(textureSize(ssaoInput, 0));

    // Get center sample depth
    float ourDepth = texture(depthTexture, fragTexCoord).r;

    int sampleCount = 0;
    float sum = 0.0;

    // Larger blur radius (13-tap filter) for smoother results
    const int radius = 6;
    // Even more lenient depth threshold for aggressive smoothing
    // This allows blur on similar depths while preserving edges
    const float depthThreshold = 0.02;

    // Determine blur direction
    vec2 blurDir = blur.direction == 0
        ? vec2(1.0, 0.0)  // Horizontal
        : vec2(0.0, 1.0); // Vertical

    // Edge-preserving blur along chosen axis
    for (int i = -radius; i <= radius; ++i) {
        vec2 offset = blurDir * float(i) * texelSize;
        vec2 sampleUV = fragTexCoord + offset;

        float depth = texture(depthTexture, sampleUV).r;

        // Only include sample if depth is similar (not across a large depth discontinuity)
        // This prevents bleeding AO across object boundaries
        if (abs(ourDepth - depth) < depthThreshold) {
            sum += texture(ssaoInput, sampleUV).r;
            sampleCount++;
        }
    }

    // If no samples passed (shouldn't happen), use center value
    outAO = sampleCount > 0 ? (sum / float(sampleCount)) : texture(ssaoInput, fragTexCoord).r;
}
