#pragma once
#include "spdlog/spdlog.h"
#include "volk.h"
#include <vulkan/vk_enum_string_helper.h>
#include "vma.hpp"

#define VK_CHECK(x)                                                            \
  do {                                                                         \
    VkResult result = x;                                                       \
    if (result) {                                                              \
      spdlog::error("Vulkan: {}", string_VkResult(result));                    \
    }                                                                          \
  } while (0)

struct Image {
  VkImage handle;
  VkImageView view;
  VkFormat format;
  VkExtent3D extent;
  VmaAllocation allocation;
};