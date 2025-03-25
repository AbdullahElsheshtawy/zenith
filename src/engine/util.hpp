#include "types.hpp"
#include <filesystem>
#include <optional>

namespace util {
VkCommandPoolCreateInfo commandPoolCreateInfo(uint32_t queueFamilyIdx,
                                              VkCommandPoolCreateFlags flags);

VkCommandBufferAllocateInfo commandBufferAllocateInfo(VkCommandPool commandPool,
                                                      uint32_t count);

VkFenceCreateInfo fenceCreateInfo(VkFenceCreateFlags flags = 0);

VkSemaphoreCreateInfo semaphoreCreateInfo(VkSemaphoreCreateFlags flags = 0);

VkCommandBufferBeginInfo
commandBufferBeginInfo(VkCommandBufferUsageFlags flags = 0);

void transitionImage(VkCommandBuffer cmd, VkImage image,
                     VkImageLayout currentLayout, VkImageLayout newLayout);

VkImageSubresourceRange imageSubresourceRange(VkImageAspectFlags aspectMask);

VkSemaphoreSubmitInfo semaphoreSubmitInfo(VkPipelineStageFlags2 stageMask,
                                          VkSemaphore semaphore);

VkCommandBufferSubmitInfo commandBufferSubmitInfo(VkCommandBuffer cmd);

VkSubmitInfo2 submitInfo(const VkCommandBufferSubmitInfo *cmd,
                         const VkSemaphoreSubmitInfo *signalSemaphoreInfo,
                         const VkSemaphoreSubmitInfo *waitSemaphoreInfo);

std::optional<VkShaderModule> loadShaderModule(VkDevice device,
                                               const std::string_view path);

VkImageCreateInfo imageCreateInfo(VkFormat format, VkImageUsageFlags usageFlags,
                                  VkExtent3D extent);

VkImageViewCreateInfo imageViewCreateInfo(VkFormat format, VkImage image,
                                          VkImageAspectFlags aspectFlags);

void copyImageToImage(VkCommandBuffer cmd, VkImage src, VkImage dst,
                      VkExtent2D srcSize, VkExtent2D dstSize);

VkRenderingAttachmentInfo attachementInfo(
    const VkImageView view, const VkClearValue *clear,
    VkImageLayout layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL);

VkRenderingInfo renderingInfo(const VkExtent2D renderExtent,
                              const VkRenderingAttachmentInfo *colorAttachemnt,
                              const VkRenderingAttachmentInfo *depthAttachment);
} // namespace util