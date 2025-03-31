#pragma once
#include "types.hpp"
#include <string_view>
#include <vector>

class PipelineBuilder {
  std::vector<VkPipelineShaderStageCreateInfo> ShaderStages_;

  VkPipelineLayout Layout_;
  VkPipelineInputAssemblyStateCreateInfo InputAssembly_;
  VkPipelineRasterizationStateCreateInfo Rasterizer_;
  VkPipelineColorBlendAttachmentState ColorBlendAttachment_;
  VkPipelineMultisampleStateCreateInfo Multisampling_;
  VkPipelineDepthStencilStateCreateInfo DepthStencil_;
  VkPipelineRenderingCreateInfo RenderInfo_;
  VkFormat ColorAttachmentFormat_;

public:
  [[nodiscard]] PipelineBuilder() noexcept { Clear(); };
  void Clear() noexcept;

  PipelineBuilder &Layout(const VkPipelineLayout layout) noexcept {
    Layout_ = layout;
    return *this;
  }
  PipelineBuilder &Shaders(const VkShaderModule vertex,
                           const std::string_view vertexEntry,
                           const VkShaderModule fragment,
                           const std::string_view fragmentEntry) noexcept;
  PipelineBuilder &InputTopology(const VkPrimitiveTopology topology) noexcept;
  PipelineBuilder &PolygonMode(const VkPolygonMode mode) noexcept;
  PipelineBuilder &CullMode(const VkCullModeFlags Cullmode,
                            const VkFrontFace frontFace) noexcept;
  PipelineBuilder &MultisamplingNone() noexcept;
  PipelineBuilder &DisableBlending() noexcept;
  PipelineBuilder &ColorAttachmentFormat(const VkFormat format) noexcept;
  PipelineBuilder &DepthFormat(const VkFormat format) noexcept;
  PipelineBuilder &DisableDepthtest() noexcept;
  VkPipeline Build(VkDevice device) const noexcept;
};