#version 450

layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;
layout(location = 2) in vec4 a_color;

layout(location = 0) out vec2 v_uv;
layout(location = 1) out vec4 v_color;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    uvec2 screen_size;
} ubo;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pcs;

void main() {
    gl_Position =
        vec4(2.0 * a_pos.x / ubo.screen_size.x - 1.0,
            2.0 * a_pos.y / ubo.screen_size.y - 1.0, 0.0, 1.0);

    v_uv = a_uv;
    v_color = a_color;
}
