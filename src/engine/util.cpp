#include "util.hpp"
#include "fstream"

VkCommandPoolCreateInfo
util::commandPoolCreateInfo(uint32_t queueFamilyIdx,
                            VkCommandPoolCreateFlags flags) {
  return VkCommandPoolCreateInfo{.sType =
                                     VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
                                 .pNext = nullptr,
                                 .flags = flags,
                                 .queueFamilyIndex = queueFamilyIdx};
}

VkCommandBufferAllocateInfo
util::commandBufferAllocateInfo(VkCommandPool commandPool, uint32_t count) {
  return VkCommandBufferAllocateInfo{
      .sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
      .pNext = nullptr,
      .commandPool = commandPool,
      .level = VK_COMMAND_BUFFER_LEVEL_PRIMARY,
      .commandBufferCount = count,
  };
}

VkFenceCreateInfo util::fenceCreateInfo(VkFenceCreateFlags flags) {
  return VkFenceCreateInfo{.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
                           .pNext = nullptr,
                           .flags = flags};
}

VkSemaphoreCreateInfo util::semaphoreCreateInfo(VkSemaphoreCreateFlags flags) {
  return VkSemaphoreCreateInfo{.sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
                               .pNext = nullptr,
                               .flags = flags};
}

VkCommandBufferBeginInfo
util::commandBufferBeginInfo(VkCommandBufferUsageFlags flags) {
  return VkCommandBufferBeginInfo{
      .sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
      .pNext = nullptr,
      .flags = flags,
      .pInheritanceInfo = nullptr,
  };
}

void util::transitionImage(VkCommandBuffer cmd, VkImage image,
                           VkImageLayout currentLayout,
                           VkImageLayout newLayout) {
  const VkImageAspectFlags aspectMask =
      newLayout == VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_OPTIMAL
          ? VK_IMAGE_ASPECT_DEPTH_BIT
          : VK_IMAGE_ASPECT_COLOR_BIT;

  VkImageMemoryBarrier2 imageBarrier{};
  imageBarrier.sType = VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER_2;
  imageBarrier.pNext = nullptr;

  imageBarrier.image = image;
  imageBarrier.subresourceRange = util::imageSubresourceRange(aspectMask);

  imageBarrier.srcStageMask = VK_PIPELINE_STAGE_2_ALL_COMMANDS_BIT;
  imageBarrier.srcAccessMask = VK_ACCESS_2_MEMORY_WRITE_BIT;

  imageBarrier.dstStageMask = VK_PIPELINE_STAGE_2_ALL_COMMANDS_BIT;
  imageBarrier.dstAccessMask =
      VK_ACCESS_2_MEMORY_WRITE_BIT | VK_ACCESS_2_MEMORY_READ_BIT;

  imageBarrier.oldLayout = currentLayout;
  imageBarrier.newLayout = newLayout;

  VkDependencyInfo depInfo{};
  depInfo.sType = VK_STRUCTURE_TYPE_DEPENDENCY_INFO;
  depInfo.pNext = nullptr;
  depInfo.imageMemoryBarrierCount = 1;
  depInfo.pImageMemoryBarriers = &imageBarrier;

  vkCmdPipelineBarrier2(cmd, &depInfo);
}

VkImageSubresourceRange
util::imageSubresourceRange(VkImageAspectFlags aspectMask) {
  return VkImageSubresourceRange{
      .aspectMask = aspectMask,
      .baseMipLevel = 0,
      .levelCount = VK_REMAINING_MIP_LEVELS,
      .baseArrayLayer = 0,
      .layerCount = VK_REMAINING_ARRAY_LAYERS,
  };
}

VkSemaphoreSubmitInfo util::semaphoreSubmitInfo(VkPipelineStageFlags2 stageMask,
                                                VkSemaphore semaphore) {
  return VkSemaphoreSubmitInfo{
      .sType = VK_STRUCTURE_TYPE_SEMAPHORE_SUBMIT_INFO,
      .pNext = nullptr,
      .semaphore = semaphore,
      .value = 1,
      .stageMask = stageMask,
      .deviceIndex = 0,
  };
}

VkCommandBufferSubmitInfo util::commandBufferSubmitInfo(VkCommandBuffer cmd) {
  return VkCommandBufferSubmitInfo{
      .sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_SUBMIT_INFO,
      .pNext = nullptr,
      .commandBuffer = cmd,
      .deviceMask = 0,
  };
}

VkSubmitInfo2 util::submitInfo(const VkCommandBufferSubmitInfo *cmd,
                               const VkSemaphoreSubmitInfo *signalSemaphoreInfo,
                               const VkSemaphoreSubmitInfo *waitSemaphoreInfo) {
  return VkSubmitInfo2{
      .sType = VK_STRUCTURE_TYPE_SUBMIT_INFO_2,
      .pNext = nullptr,
      .waitSemaphoreInfoCount = waitSemaphoreInfo == nullptr ? 0u : 1u,
      .pWaitSemaphoreInfos = waitSemaphoreInfo,
      .commandBufferInfoCount = 1,
      .pCommandBufferInfos = cmd,
      .signalSemaphoreInfoCount = signalSemaphoreInfo == nullptr ? 0u : 1u,
      .pSignalSemaphoreInfos = signalSemaphoreInfo,
  };
}

VkShaderModule util::loadShaderModule(VkDevice device,
                                      const std::string_view filePath) {
  VkShaderModule shaderModule = VK_NULL_HANDLE;
  // open the file. With cursor at the end
  std::ifstream file(filePath.data(), std::ios::ate | std::ios::binary);

  if (!file.is_open()) {
    spdlog::error("Could not open shader: {}", filePath);
  }

  size_t fileSize = (size_t)file.tellg();

  std::vector<uint32_t> buffer(fileSize / sizeof(uint32_t));

  file.seekg(0);

  file.read((char *)buffer.data(), fileSize);
  file.close();

  VkShaderModuleCreateInfo createInfo = {};
  createInfo.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
  createInfo.pNext = nullptr;

  createInfo.codeSize = buffer.size() * sizeof(uint32_t);
  createInfo.pCode = buffer.data();

  const VkResult res =
      vkCreateShaderModule(device, &createInfo, nullptr, &shaderModule);
  if (res != VK_SUCCESS) {
    spdlog::error("Failed to create {} shader module: {}", filePath,
                  string_VkResult(res));
  }
  return shaderModule;
}

VkImageCreateInfo util::imageCreateInfo(VkFormat format,
                                        VkImageUsageFlags usageFlags,
                                        VkExtent3D extent) {
  return VkImageCreateInfo{
      .sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
      .pNext = nullptr,
      .imageType = VK_IMAGE_TYPE_2D,
      .format = format,
      .extent = extent,
      .mipLevels = 1,
      .arrayLayers = 1,
      .samples = VK_SAMPLE_COUNT_1_BIT,
      .tiling = VK_IMAGE_TILING_OPTIMAL,
      .usage = usageFlags,
  };
}

VkImageViewCreateInfo
util::imageViewCreateInfo(VkFormat format, VkImage image,
                          VkImageAspectFlags aspectFlags) {
  return VkImageViewCreateInfo{
      .sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
      .pNext = nullptr,
      .image = image,
      .viewType = VK_IMAGE_VIEW_TYPE_2D,
      .format = format,
      .subresourceRange = {.aspectMask = aspectFlags,
                           .baseMipLevel = 0,
                           .levelCount = 1,
                           .baseArrayLayer = 0,
                           .layerCount = 1},
  };
}

void util::copyImageToImage(VkCommandBuffer cmd, VkImage src, VkImage dst,
                            VkExtent2D srcSize, VkExtent2D dstSize) {
  VkImageBlit2 blitRegion{};
  blitRegion.sType = VK_STRUCTURE_TYPE_IMAGE_BLIT_2;
  blitRegion.pNext = nullptr;

  blitRegion.srcOffsets[1] = {.x = static_cast<int>(srcSize.width),
                              .y = static_cast<int>(srcSize.height),
                              .z = 1};

  blitRegion.dstOffsets[1] = {.x = static_cast<int>(dstSize.width),
                              .y = static_cast<int>(dstSize.height),
                              .z = 1};

  blitRegion.srcSubresource = {
      .aspectMask = VK_IMAGE_ASPECT_COLOR_BIT,
      .mipLevel = 0,
      .baseArrayLayer = 0,
      .layerCount = 1,
  };

  blitRegion.dstSubresource = {
      .aspectMask = VK_IMAGE_ASPECT_COLOR_BIT,
      .mipLevel = 0,
      .baseArrayLayer = 0,
      .layerCount = 1,
  };

  VkBlitImageInfo2 blitInfo{};
  blitInfo.sType = VK_STRUCTURE_TYPE_BLIT_IMAGE_INFO_2;
  blitInfo.pNext = nullptr;

  blitInfo.srcImage = src;
  blitInfo.srcImageLayout = VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL;

  blitInfo.dstImage = dst;
  blitInfo.dstImageLayout = VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL;

  blitInfo.filter = VK_FILTER_LINEAR;
  blitInfo.regionCount = 1;
  blitInfo.pRegions = &blitRegion;

  vkCmdBlitImage2(cmd, &blitInfo);
}

VkRenderingAttachmentInfo util::attachementInfo(const VkImageView view,
                                                const VkClearValue *clear,
                                                const VkImageLayout layout) {
  VkRenderingAttachmentInfo colorAttachment = {
      .sType = VK_STRUCTURE_TYPE_RENDERING_ATTACHMENT_INFO,
      .pNext = nullptr,
      .imageView = view,
      .imageLayout = layout,
      .loadOp =
          clear ? VK_ATTACHMENT_LOAD_OP_CLEAR : VK_ATTACHMENT_LOAD_OP_LOAD,
      .storeOp = VK_ATTACHMENT_STORE_OP_STORE,
  };

  if (clear) {
    colorAttachment.clearValue = *clear;
  }
  return colorAttachment;
}

VkRenderingInfo
util::renderingInfo(const VkExtent2D renderExtent,
                    const VkRenderingAttachmentInfo *colorAttachemnt,
                    const VkRenderingAttachmentInfo *depthAttachment) {
  return VkRenderingInfo{.sType = VK_STRUCTURE_TYPE_RENDERING_INFO,
                         .pNext = nullptr,
                         .renderArea = VkRect2D{VkOffset2D{0, 0}, renderExtent},
                         .layerCount = 1,
                         .colorAttachmentCount = 1,
                         .pColorAttachments = colorAttachemnt,
                         .pDepthAttachment = depthAttachment,
                         .pStencilAttachment = nullptr};
}
