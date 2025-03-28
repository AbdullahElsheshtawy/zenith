// From
// https://github.com/vblanco20-1/vulkan-guide/blob/all-chapters-2/shaders/sky.comp

struct PushConstants {
  float4 data1;
  float4 data2;
  float4 data3;
  float4 data4;
};
[[vk::push_constant]]
ConstantBuffer<PushConstants> pc;

// Return random noise in the range [0.0, 1.0] as a function of x.
// f(x) = fract(415.92653 * (cos(x.x * 37.0) + cos(x.y * 57.0))
float Noise2d(float2 x) {
  float xhash = cos(x.x * 37.0);
  float yhash = cos(x.y * 57.0);
  return fract(415.92653 * (xhash + yhash));
}

// Convert [`Noise2d()`] into a "star field" by stomping everything below
// `threshhold` to zero.
float NoisyStarField(float2 samplePos, float threshhold) {
  float starVal = Noise2d(samplePos);
  if (starVal >= threshhold) {
    starVal = pow((starVal - threshhold) / (1.0 - threshhold), 6.0);
  } else {
    starVal = 0.0;
  }

  return starVal;
}

// Stabalize the [`NoiseStarField()`] by only sampling at integer values.
float StableStarField(float2 samplePos, float threshhold) {
  // Linear interpolation between four samples.
  // NOTE: This approach has some visual artifacts.
  float fractX = fract(samplePos.x);
  float fractY = fract(samplePos.y);
  float2 floorSample = floor(samplePos);
  float v1 = NoisyStarField(floorSample, threshhold);
  float v2 = NoisyStarField(floorSample + float2(0.0, 1.0), threshhold);
  float v3 = NoisyStarField(floorSample + float2(1.0, 0.0), threshhold);
  float v4 = NoisyStarField(floorSample + float2(1.0, 1.0), threshhold);

  float StarVal = v1 * (1.0 - fractX) * (1.0 - fractY) +
                  v2 * (1.0 - fractX) * fractY + v3 * fractX * (1.0 - fractY) +
                  v4 * fractX * fractY;
  return StarVal;
}

float4 mainImage(RWTexture2D<float4> image, float2 fragCoord) {
  uint width, height;
  image.GetDimensions(width, height);
  uint2 resolution = uint2(width, height);
  // Sky background Color
  float3 color = pc.data1.xyz * fragCoord.y / resolution.y;

  // NOTE: Choose threshhold in the range of [0.99, 0.9999].
  // Higher values yield a sparser starfield.
  float starFieldThreshhold = pc.data1.w;

  // Stars with a slow crawl.
  float xRate = 0.2;
  float yRate = -0.06;
  float2 samplePos = fragCoord.xy + float2(xRate, yRate);
  float starVal = StableStarField(samplePos, starFieldThreshhold);

  color += float3(starVal);
  return float4(color, 1.0);
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 threadId: SV_DispatchThreadID, RWTexture2D<float4> image) {
  let texelCoord = int2(threadId.xy);
  uint width, height;
  image.GetDimensions(width, height);
  uint2 size = uint2(width, height);
  if (texelCoord.x < size.x && texelCoord.y < size.y) {
    image[texelCoord] = mainImage(image, texelCoord);
  }
}
