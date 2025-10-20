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
    vec2 _padding;
} ubo;

// Hash functions from Shadertoy
#define HASHSCALE1 .1031
#define HASHSCALE3 vec3(.1031, .1030, .0973)
#define HASHSCALE4 vec4(1031, .1030, .0973, .1099)

float hash(float p) {
    vec3 p3  = fract(vec3(p) * HASHSCALE1);
    p3 += dot(p3, p3.yzx + 19.19);
    return fract((p3.x + p3.y) * p3.z);
}

vec4 hash4(const in vec4 n) {
    return fract(sin(n)*1399763.5453123);
}

// Noise function
float noise4q(vec4 x) {
    vec4 n3 = vec4(0,.25,.5,.75);
    vec4 p2 = floor(x.wwww+n3);
    vec4 b = floor(x.xxxx+n3) + floor(x.yyyy+n3)*157. + floor(x.zzzz +n3)*113.;
    vec4 p1 = b + fract(p2*.00390625)*vec4(164352., -164352., 163840., -163840.);
    p2 = b + fract((p2+1.0)*.00390625)*vec4(164352., -164352., 163840., -163840.);
    vec4 f1 = fract(x.xxxx+n3),f2 = fract(x.yyyy+n3);
    f1 *= f1*(3.0-f1-f1);
    f2 *= f2*(3.0-f2-f2);
    vec4 n1 = vec4(0,1.,157.,158.),
         n2 = vec4(113.,114.,270.0,271.);
    vec4 vs1 = mix(hash4(p1), hash4(n1.yyyy+p1), f1),
         vs2 = mix(hash4(n1.zzzz+p1), hash4(n1.wwww+p1), f1),
         vs3 = mix(hash4(p2), hash4(n1.yyyy+p2), f1),
         vs4 = mix(hash4(n1.zzzz+p2), hash4(n1.wwww+p2), f1);
    vs1 = mix(vs1, vs2, f2);
    vs3 = mix(vs3, vs4, f2);
    vs2 = mix(hash4(n2.xxxx+p1), hash4(n2.yyyy+p1), f1);
    vs4 = mix(hash4(n2.zzzz+p1), hash4(n2.wwww+p1), f1);
    vs2 = mix(vs2, vs4, f2);
    vs4 = mix(hash4(n2.xxxx+p2), hash4(n2.yyyy+p2), f1);
    vec4 vs5 = mix(hash4(n2.zzzz+p2), hash4(n2.wwww+p2), f1);
    vs4 = mix(vs4, vs5, f2);
    f1 = fract(x.zzzz+n3);
    f2 = fract(x.wwww+n3);
    f1 *= f1*(3.-f1-f1);
    f2 *= f2*(3.-f2-f2);
    vs1 = mix(vs1, vs2, f1);
    vs3 = mix(vs3, vs4, f1);
    vs1 = mix(vs1, vs3, f2);
    float r = dot(vs1,vec4(.25));
    return r*r*(3.-r-r);
}

// Black body color from Planck's law
vec3 blackBodyColor(float k) {
    float T = (k*2.)*16000.;
    vec3 c = vec3(1.,3.375,8.)/(exp((19e3*vec3(1.,1.5,2.)/T)) - 1.);
    return c / max(c.r,max(c.g,c.b));
}

// Smooth noise for star surface
const mat3 msun = mat3(0., .8, .6, -.8, .36, -.48, -.6, -.48, .64);

float smoothNoise(in vec3 q){
    // Simplified version - using hash instead of texture lookup
    vec3 p = floor(q);
    vec3 f = fract(q);
    f = f*f*(3.0-2.0*f);

    float n = p.x + p.y*57.0 + 113.0*p.z;
    float res = mix(mix(mix(hash(n+0.0), hash(n+1.0),f.x),
                        mix(hash(n+57.0), hash(n+58.0),f.x),f.y),
                    mix(mix(hash(n+113.0), hash(n+114.0),f.x),
                        mix(hash(n+170.0), hash(n+171.0),f.x),f.y),f.z);

    vec3 q2 = msun*q*2.01;
    n = q2.x + q2.y*57.0 + 113.0*q2.z;
    p = floor(q2);
    f = fract(q2);
    f = f*f*(3.0-2.0*f);
    res += 0.5 * mix(mix(mix(hash(n+0.0), hash(n+1.0),f.x),
                         mix(hash(n+57.0), hash(n+58.0),f.x),f.y),
                     mix(mix(hash(n+113.0), hash(n+114.0),f.x),
                         mix(hash(n+170.0), hash(n+171.0),f.x),f.y),f.z);

    return res;
}

// Star rays effect
float ringRayNoise(vec3 viewDir, vec3 surfacePos, float r, float size, float anim) {
    vec3 pr = surfacePos - dot(surfacePos, viewDir) * viewDir;
    float c = length(pr);
    float s = max(0.,(1.-size*abs(r-c)));
    pr = pr/max(c, 0.001);

    float n = 0.4;
    float nd = noise4q(vec4(pr*1.0,-anim+c))*2.0;
    if (c > r) {
        n = noise4q(vec4(pr*10.0,-anim+c));
        n *= noise4q(vec4(pr*50.0,-anim*2.5+ c+c))*2.;
    }
    n *= n*nd*nd;
    return s*s*(s*s+n);
}

void main() {
    // Get local position on sphere (normalized)
    vec3 localPos = normalize(fragNormal);

    // Surface lava effect
    float lava = smoothNoise((localPos + vec3(ubo.time*.03))*50.0);

    // Compute star surface color with black body radiation
    vec3 surfaceColor = blackBodyColor(.02 + 3.0 * clamp(0.7, .05, 1.) * (1. - sqrt(lava)));

    // View direction
    vec3 viewDir = normalize(ubo.viewPos - fragWorldPos);

    // Star rays effect
    float rays = ringRayNoise(viewDir, localPos, 1.0, 2.0, ubo.time);

    // Combine surface and rays
    vec3 finalColor = surfaceColor;
    finalColor += blackBodyColor(0.7) * rays * 0.5;

    // Apply twinkling
    finalColor *= 1.0 - 0.03*cos(5.0*ubo.time + 2.0*hash(ubo.time));

    // Tone mapping and gamma correction
    finalColor *= ubo.exposure;
    finalColor = finalColor / (finalColor + vec3(1.0)); // Reinhard tone mapping
    finalColor = pow(finalColor, vec3(1.0/ubo.gamma));

    // Apply star color tint
    finalColor *= ubo.starColor;

    // Edge glow
    float edge = 1.0 - abs(dot(normalize(fragNormal), viewDir));
    edge = pow(edge, 2.0);
    finalColor += ubo.starColor * edge * 0.3;

    outColor = vec4(finalColor, 1.0);
}
