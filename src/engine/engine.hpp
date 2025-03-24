#pragma once
#include "SDL3/SDL.h"
#include "types.hpp"
#include <array>

constexpr int FRAMES_IN_FLIGHT = 2;

struct FrameData {
  VkSemaphore swapchainSemaphore, renderSemaphore;
  VkFence renderFence;

  VkCommandPool commandPool;
  VkCommandBuffer commandBuffer;
};

struct Swapchain {
  VkSwapchainKHR handle;
  VkExtent2D extent;
  VkFormat format;
  std::vector<VkImage> images;
  std::vector<VkImageView> views;
};

class Engine {
public:
  Engine();
  ~Engine();
  void run();

private:
  SDL_Window *Window_;
  VkInstance Instance_;
  VkSurfaceKHR Surface_;
  VkPhysicalDevice physicalDevice_;
  VkDevice Device_;
  uint32_t GfxQueueFamilyIdx_;
  VkQueue GfxQueue_;
  Swapchain Swapchain_;
  std::array<FrameData, FRAMES_IN_FLIGHT> FrameData_;
  size_t FrameNumber_{};
  VkExtent2D WindowExtent_;

private:
  void CreateSwapchain();
  void DestroySwapchain();
  void InitializeFrameData();
  FrameData &GetCurrentFrame() {
    return FrameData_[FrameNumber_ % FRAMES_IN_FLIGHT];
  };

  void draw();
};