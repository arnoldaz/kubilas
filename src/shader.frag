#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(set = 0, binding = 1) uniform sampler2D texSampler[2];

layout(push_constant) uniform PushConstants {
    layout(offset = 64) uint objectIndex;
} pcs;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragColor * texture(texSampler[pcs.objectIndex], fragTexCoord).rgb, 1.0);
}