#pragma once

#include <array>
#include <expected>
#include <filesystem>
#include <functional>
#include <memory>
#include <optional>
#include <ranges>
#include <span>
#include <string>
#include <string_view>
#include <vector>

#include "Volk/volk.h"
#include "glm/glm.hpp"
#include "spdlog/spdlog.h"
#include "vk_mem_alloc.h"
#include <vulkan/vk_enum_string_helper.h>

#define VK_CHECK(x)                                                            \
  do {                                                                         \
    VkResult err = x;                                                          \
    if (err) {                                                                 \
      spdlog::error("Vulkan Error: {}", string_VkResult(err));                 \
    }                                                                          \
  } while (0)