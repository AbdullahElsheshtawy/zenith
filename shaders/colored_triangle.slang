struct VertexOutput {
  float4 pos : SV_Position;
  float3 color : COLOR;
};

[shader("vertex")]
VertexOutput vertexMain(uint vertexID: SV_VertexID) {
  static const float3 positions[3] = {
    float3(1.0, 1.0, 0.0),
    float3(-1.0, 1.0, 0.0),
    float3(0.0, -1.0, 0.0),
  };
  static const float3 colors[3] = {
    float3(1.0, 0.0, 0.0),
    float3(0.0, 1.0, 0.0),
    float3(0.0, 0.0, 1.0),
  };

  VertexOutput output;
  output.pos = float4(positions[vertexID], 1.0);
  output.color = colors[vertexID];
  return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input) { return float4(input.color, 1.0); }
