#pragma once
#include "VulkanTypes.hpp"


constexpr uint32_t FRAMES_IN_FLIGHT = 3;
struct FrameData {
  VkCommandPool CommandPool{};
  VkCommandBuffer MainCommandBuffer{};
};

class VulkanEngine {
private:
  struct SDL_Window *Window_{nullptr};
  VkExtent2D WindowExtent_{800, 600};

  VkInstance Instance_{};
  VkPhysicalDevice PhysicalDevice_{};
  VkDevice Device_{};
  VkSurfaceKHR Surface_{};
  VkQueue GraphicsQueue_{};
  uint32_t GraphicsQueueFamilyIndex_{};
  FrameData FrameData_[FRAMES_IN_FLIGHT]{};
  uint32_t FrameNumber_{};


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

  FrameData& GetCurrentFrameData() {
    return FrameData_[FrameNumber_ % FRAMES_IN_FLIGHT];
  }

  void run();
  void draw();

private:
  void initializeVulkan();
  void inializeCommands();
  void createSwapchain(uint32_t width, uint32_t height);
};
