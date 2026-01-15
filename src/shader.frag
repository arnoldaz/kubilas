#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
// layout(location = 2) in vec3 inPosition;

layout(binding = 1) uniform sampler2D texSampler[];

layout(push_constant) uniform PushConstants {
    layout(offset = 64) uint textureId;
} pcs;

layout(location = 0) out vec4 outColor;

void main() {
    // uint texIndex = uint(gl_FragCoord.x) % 10;
    outColor = vec4(fragColor * texture(texSampler[pcs.textureId], fragTexCoord).rgb, 1.0);
}