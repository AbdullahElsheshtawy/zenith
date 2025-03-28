struct PushConstants {
  float4 data1;
  float4 data2;
  float4 data3;
  float4 data4;
};
[[vk::push_constant]]
ConstantBuffer<PushConstants> pc;

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 threadId: SV_DispatchThreadID,
                 RWTexture2D<float4> image) {
    let texelCoord = int2(threadId.xy);
    uint width, height;
    image.GetDimensions(width, height);
    let size = uint2(width, height);

  if (texelCoord.x < size.x && texelCoord.y < size.y) {
      let blend = float(texelCoord.y) / size.y;
      image[texelCoord] = lerp(pc.data1, pc.data2, blend);
  }
}
