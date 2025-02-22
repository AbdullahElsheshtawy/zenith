#include "VulkanInitializers.hpp"

namespace VulkanInit {
VkCommandPoolCreateInfo commandPoolCreateInfo(uint32_t queueFamilyIndex,
                                              VkCommandPoolCreateFlags flags) {
  return VkCommandPoolCreateInfo{
      .sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
      .pNext = nullptr,
      .flags = flags,
      .queueFamilyIndex = queueFamilyIndex,
  };
}

VkCommandBufferAllocateInfo
commandBufferAllocateInfo(VkCommandPool pool, uint32_t commandBufferCount) {
  return VkCommandBufferAllocateInfo{
      .sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
      .pNext = nullptr,
      .commandPool = pool,
      .level = VK_COMMAND_BUFFER_LEVEL_PRIMARY,
      .commandBufferCount = commandBufferCount

  };
}

} // namespace VulkanInit