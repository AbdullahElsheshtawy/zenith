#include "types.hpp"

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
}