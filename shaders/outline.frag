#version 450

layout(location = 0) out vec4 outColor;

layout(push_constant) uniform PushConstants {
    mat4 model;
    vec4 outlineColor;   // RGB color + alpha
    float outlineWidth;  // How much to expand the mesh
    float _padding1;
    float _padding2;
    float _padding3;
} push;

void main() {
    // Simple solid color for outline
    outColor = push.outlineColor;
}
