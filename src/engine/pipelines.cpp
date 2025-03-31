#include "pipelines.hpp"
#include "util.hpp"
#include <array>

void PipelineBuilder::Clear() noexcept {
  Layout_ = {};
  InputAssembly_ = {
      .sType = VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
      .pNext = nullptr,
  };
  Rasterizer_ = {
      .sType = VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
      .pNext = nullptr,
  };
  ColorBlendAttachment_ = {};
  Multisampling_ = {
      .sType = VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
      .pNext = nullptr,
  };
  DepthStencil_ = {
      .sType = VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
      .pNext = nullptr,
  };
  RenderInfo_ = {
      .sType = VK_STRUCTURE_TYPE_PIPELINE_RENDERING_CREATE_INFO,
      .pNext = nullptr,
  };
}

PipelineBuilder &
PipelineBuilder::Shaders(const VkShaderModule vertex,
                         const std::string_view vertexEntry,
                         const VkShaderModule fragment,
                         const std::string_view fragmentEntry) noexcept {
  ShaderStages_.clear();
  ShaderStages_.emplace_back(util::pipelineShaderStageCreateInfo(
      VK_SHADER_STAGE_VERTEX_BIT, vertex, vertexEntry));
  ShaderStages_.emplace_back(util::pipelineShaderStageCreateInfo(
      VK_SHADER_STAGE_FRAGMENT_BIT, fragment, fragmentEntry));
  return *this;
}

PipelineBuilder &
PipelineBuilder::InputTopology(const VkPrimitiveTopology topology) noexcept {
  InputAssembly_.topology = topology;
  InputAssembly_.primitiveRestartEnable = VK_FALSE;
  return *this;
}

PipelineBuilder &
PipelineBuilder::PolygonMode(const VkPolygonMode mode) noexcept {
  Rasterizer_.polygonMode = mode;
  Rasterizer_.lineWidth = 1.0;
  return *this;
}

PipelineBuilder &
PipelineBuilder::CullMode(const VkCullModeFlags Cullmode,
                          const VkFrontFace frontFace) noexcept {
  Rasterizer_.cullMode = Cullmode;
  Rasterizer_.frontFace = frontFace;
  return *this;
}

PipelineBuilder &PipelineBuilder::MultisamplingNone() noexcept {
  Multisampling_.sampleShadingEnable = VK_FALSE;
  Multisampling_.rasterizationSamples = VK_SAMPLE_COUNT_1_BIT;
  Multisampling_.minSampleShading = 1.0;
  Multisampling_.pSampleMask = nullptr;
  Multisampling_.alphaToCoverageEnable = VK_FALSE;
  Multisampling_.alphaToOneEnable = VK_FALSE;
  return *this;
}

PipelineBuilder &PipelineBuilder::DisableBlending() noexcept {
  // No blending
  ColorBlendAttachment_.colorWriteMask =
      VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT |
      VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT;
  ColorBlendAttachment_.blendEnable = VK_FALSE;
  return *this;
}

PipelineBuilder &
PipelineBuilder::ColorAttachmentFormat(const VkFormat format) noexcept {
  ColorAttachmentFormat_ = format;
  RenderInfo_.colorAttachmentCount = 1;
  RenderInfo_.pColorAttachmentFormats = &ColorAttachmentFormat_;
  return *this;
}
PipelineBuilder &PipelineBuilder::DepthFormat(const VkFormat format) noexcept {
  RenderInfo_.depthAttachmentFormat = format;
  return *this;
}
PipelineBuilder &PipelineBuilder::DisableDepthtest() noexcept {
  DepthStencil_.depthTestEnable = VK_FALSE;
  DepthStencil_.depthWriteEnable = VK_FALSE;
  DepthStencil_.depthBoundsTestEnable = VK_FALSE;
  DepthStencil_.depthCompareOp = VK_COMPARE_OP_NEVER;
  DepthStencil_.stencilTestEnable = VK_FALSE;
  DepthStencil_.front = {};
  DepthStencil_.back = {};
  DepthStencil_.minDepthBounds = 0.0;
  DepthStencil_.maxDepthBounds = 1.0;

  return *this;
}

VkPipeline PipelineBuilder::Build(VkDevice device) const noexcept {
  // Dynamic viewport and scissor
  const VkPipelineViewportStateCreateInfo viewportState{
      .sType = VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
      .pNext = nullptr,
      .viewportCount = 1,
      .scissorCount = 1,
  };

  // Dummy color blending
  const VkPipelineColorBlendStateCreateInfo colorBlending = {
      .sType = VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
      .pNext = nullptr,
      .logicOpEnable = VK_FALSE,
      .logicOp = VK_LOGIC_OP_COPY,
      .attachmentCount = 1,
      .pAttachments = &ColorBlendAttachment_,
  };

  const VkPipelineVertexInputStateCreateInfo vertexInputInfo = {
      .sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
      .pNext = nullptr,
  };

  const std::array<VkDynamicState, 2> dynamicState = {
      VK_DYNAMIC_STATE_VIEWPORT,
      VK_DYNAMIC_STATE_SCISSOR,
  };
  const VkPipelineDynamicStateCreateInfo dynamicInfo = {
      .sType = VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
      .pNext = nullptr,
      .dynamicStateCount = static_cast<uint32_t>(dynamicState.size()),
      .pDynamicStates = dynamicState.data(),
  };

  const VkGraphicsPipelineCreateInfo pipelineInfo = {
      .sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
      .pNext = &RenderInfo_,
      .stageCount = static_cast<uint32_t>(ShaderStages_.size()),
      .pStages = ShaderStages_.data(),
      .pVertexInputState = &vertexInputInfo,
      .pInputAssemblyState = &InputAssembly_,
      .pViewportState = &viewportState,
      .pRasterizationState = &Rasterizer_,
      .pMultisampleState = &Multisampling_,
      .pDepthStencilState = &DepthStencil_,
      .pColorBlendState = &colorBlending,
      .pDynamicState = &dynamicInfo,
      .layout = Layout_,
  };

  VkPipeline pipeline = VK_NULL_HANDLE;
  if (const auto res = vkCreateGraphicsPipelines(
          device, VK_NULL_HANDLE, 1, &pipelineInfo, nullptr, &pipeline);
      res != VK_SUCCESS) {
    spdlog::error("Failed to create graphics pipeline: {}",
                  string_VkResult(res));
  }
  return pipeline;
}
