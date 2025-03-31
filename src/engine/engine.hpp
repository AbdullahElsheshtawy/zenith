#pragma once
#include "SDL3/SDL.h"
#include "deletion_queue.hpp"
#include "descriptors.hpp"
#include "glm/vec4.hpp"
#include "imgui.h"
#include "imgui_impl_sdl3.h"
#include "imgui_impl_vulkan.h"
#include "types.hpp"
#include <array>

constexpr int FRAMES_IN_FLIGHT = 2;

struct FrameData {
  VkSemaphore swapchainSemaphore, renderSemaphore;
  VkFence renderFence;

  VkCommandPool commandPool;
  VkCommandBuffer commandBuffer;

  DeletionQueue deletionQueue;
};

struct Swapchain {
  VkSwapchainKHR handle;
  VkExtent2D extent;
  VkFormat format;
  std::vector<VkImage> images;
  std::vector<VkImageView> views;
};

struct Immediate {
  VkFence fence;
  VkCommandBuffer commandBuffer;
  VkCommandPool commandPool;
};

struct ComputePushConstants {
  glm::vec4 data1;
  glm::vec4 data2;
  glm::vec4 data3;
  glm::vec4 data4;
};

struct ComputeEffect {
  const char *name;
  VkPipelineLayout layout;
  VkPipeline pipeline;
  ComputePushConstants data;
};
class Engine {
public:
  Engine(uint32_t width, uint32_t height);
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
  VmaAllocator Allocator_;
  DeletionQueue DeletionQueue_;
  Image DrawImage_;
  VkDescriptorSet DrawImageDescriptors_;
  VkDescriptorSetLayout DrawImageDescriptorLayout_;
  DescriptorAllocator GlobalDescriptorAllocator_;
  VkExtent2D DrawExtent_;
  Immediate Immediate_;

  std::vector<ComputeEffect> BackgroundEffects_;
  int CurrentBackgroundEffect_{0};

  VkPipelineLayout TrianglePipelineLayout_;
  VkPipeline TrianglePipeline_;

private:
  void CreateSwapchain();
  void DestroySwapchain();
  void InitializeCommands();
  void InitializeDescriptors();
  void InitializePipelines();
  void InitializeImgui();
  void InitializeTrianglePipeline();
  FrameData &GetCurrentFrame() {
    return FrameData_[FrameNumber_ % FRAMES_IN_FLIGHT];
  };

  void Draw();
  void DrawBackground(VkCommandBuffer cmd) const;
  void DrawGeometry(VkCommandBuffer cmd) const;
  void DrawImgui(VkCommandBuffer cmd, VkImageView targetImageView) const;
  void
  ImmediateSubmit(const std::function<void(VkCommandBuffer cmd)> &&function) const;
};