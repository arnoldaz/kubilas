#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec4 v_color;

layout(binding = 1) uniform sampler2D texSampler[];

layout(push_constant) uniform PushConstants {
    layout(offset = 64) uint textureId;
} pcs;

layout(location = 0) out vec4 outColor;

void main() {
    vec4 color = texture(texSampler[nonuniformEXT(pcs.textureId)], v_uv);
    outColor = color * v_color;
}
