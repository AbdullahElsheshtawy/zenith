#include "VulkanTypes.hpp"

namespace VulkanInit {
VkCommandPoolCreateInfo
commandPoolCreateInfo(uint32_t queueFamilyIndex,
                      VkCommandPoolCreateFlags flags = 0);

VkCommandBufferAllocateInfo
commandBufferAllocateInfo(VkCommandPool pool, uint32_t commandBufferCount);
} // namespace VulkanInit