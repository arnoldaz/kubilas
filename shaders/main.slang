struct UniformBufferObject {
    float4x4 view;
    float4x4 proj;
    uint2 screen_size;
};

ConstantBuffer<UniformBufferObject> ubo;

[[vk::binding(1, 0)]]
Sampler2D textureSampler[];

struct PushConstants {
    float4x4 model;
    uint textureId;
};

struct VSInput {
    float3 position;
    float3 color;
    float2 uv;
};

struct VSOutput {
    float4 position : SV_POSITION;
    float3 color;
    float2 uv;
};

[shader("vertex")]
VSOutput main(VSInput input, uniform PushConstants pc) {
    VSOutput output;

    float4 pos = float4(input.position, 1.0);
    output.position = mul(ubo.proj, mul(ubo.view, mul(pc.model, pos)));
    output.color = input.color;
    output.uv = input.uv;

    return output;
}

[shader("fragment")]
float4 main(VSOutput input, uniform PushConstants pc) {
    uint id = NonUniformResourceIndex(pc.textureId);
    float3 texColor = textureSampler[id].Sample(input.uv).rgb;

    return float4(input.color * texColor, 1.0);
}

//asd