struct UniformBufferObject {
    float4x4 view;
    float4x4 proj;
    uint2 screen_size;
};

ConstantBuffer<UniformBufferObject> ubo;

[[vk::binding(1, 0)]]
Sampler2D textureSampler[];

struct PushConstants {
    uint textureId;
};

struct VSInput {
    float2 position;
    float4 color;
    float2 uv;
};

struct VSOutput {
    float4 position : SV_POSITION;
    float4 color;
    float2 uv;
};

[shader("vertex")]
VSOutput main(VSInput input) {
    VSOutput output;

    float x = 2.0 * input.position.x / ubo.screen_size.x - 1.0;
    float y = 2.0 * input.position.y / ubo.screen_size.y - 1.0;
    output.position = float4(x, y, 0.0, 1.0);
    float3 gamma = pow(input.color.rgb, float3(2.2, 2.2, 2.2));
    output.color = float4(gamma, input.color.a);
    output.uv = input.uv;

    return output;
}

[shader("fragment")]
float4 main(VSOutput input, uniform PushConstants pc) {
    uint id = NonUniformResourceIndex(pc.textureId);
    float4 textureColor = textureSampler[id].Sample(input.uv);

    return input.color * textureColor;
}
