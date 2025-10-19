#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in float fragHighlight;

layout(location = 0) out vec4 outColor;

void main() {
    // Brighten the color when highlighted
    vec3 color = fragColor;
    if (fragHighlight > 0.5) {
        color = mix(fragColor, vec3(1.0), 0.5); // Brighten by 50%
    }
    outColor = vec4(color, 1.0);
}
