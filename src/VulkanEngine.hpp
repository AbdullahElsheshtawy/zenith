#pragma once
#include "VulkanTypes.hpp"

class VulkanEngine {
private:
  struct SDL_Window *Window_{nullptr};
  VkExtent2D WindowExtent_{800, 600};

  VkInstance Instance_{};
  VkPhysicalDevice PhysicalDevice_{};
  VkDevice Device_{};
  VkSurfaceKHR Surface_{};

  struct Swapchain {
    VkSwapchainKHR swapchain;
    VkFormat format{VK_FORMAT_B8G8R8A8_UNORM};
    VkExtent2D extent;
    std::vector<VkImage> images;
    std::vector<VkImageView> imageViews;
  } Swapchain_{};

public:
  VulkanEngine();
  ~VulkanEngine();

  void run();
  void draw();

private:
  void initializeVulkan();
  void createSwapchain(uint32_t width, uint32_t height);
};
