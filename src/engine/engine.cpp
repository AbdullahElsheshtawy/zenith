#include "engine.hpp"
#include "SDL3/SDL.h"
#include "SDL3/SDL_vulkan.h"
#include "VkBootstrap.h"
#include "engine.hpp"
#include "util.hpp"

Engine::Engine() {
  SDL_Init(SDL_INIT_VIDEO);
  WindowExtent_ = {800, 600};
  Window_ = SDL_CreateWindow("zenith", WindowExtent_.width,
                             WindowExtent_.height, SDL_WINDOW_VULKAN);
  VK_CHECK(volkInitialize());
  auto vkbInstance = vkb::InstanceBuilder()
                         .require_api_version(VK_API_VERSION_1_3)
                         .build()
                         .value();

  Instance_ = vkbInstance.instance;
  volkLoadInstance(Instance_);
  SDL_Vulkan_CreateSurface(Window_, Instance_, nullptr, &Surface_);

  auto vkbPhysicalDevice =
      vkb::PhysicalDeviceSelector(vkbInstance, Surface_)
          .set_minimum_version(1, 3)
          .set_required_features_12(
              {.descriptorIndexing = true, .bufferDeviceAddress = true})
          .set_required_features_13(
              {.synchronization2 = true, .dynamicRendering = true})
          .select()
          .value();

  auto vkbDevice = vkb::DeviceBuilder(vkbPhysicalDevice).build().value();
  Device_ = vkbDevice.device;

  physicalDevice_ = vkbPhysicalDevice.physical_device;
  GfxQueueFamilyIdx_ =
      vkbDevice.get_queue_index(vkb::QueueType::graphics).value();
  GfxQueue_ = vkbDevice.get_queue(vkb::QueueType::graphics).value();
  CreateSwapchain();
  InitializeFrameData();
}

void Engine::run() {
  bool quit = false;
  bool stop_rendering = false;
  SDL_Event event;

  while (!quit) {
    while (SDL_PollEvent(&event)) {
      if (event.type == SDL_EVENT_QUIT ||
          (event.key.down == true && event.key.key == SDLK_ESCAPE)) {
        quit = true;
      }

      if (event.type == SDL_EVENT_WINDOW_MINIMIZED) {
        stop_rendering = true;
        if (event.type == SDL_EVENT_WINDOW_RESTORED) {
          stop_rendering = false;
        }
      }
    }

    if (stop_rendering) {
      std::this_thread::sleep_for(std::chrono::milliseconds(100));
    } else {
      draw();
    }
  }
}

Engine::~Engine() {
  vkDeviceWaitIdle(Device_);
  DestroySwapchain();

  for (size_t i = 0; i < FrameData_.size(); i++) {
    FrameData &data = FrameData_[i];
    vkDestroyCommandPool(Device_, data.commandPool, nullptr);

    vkDestroySemaphore(Device_, data.renderSemaphore, nullptr);
    vkDestroySemaphore(Device_, data.swapchainSemaphore, nullptr);
    vkDestroyFence(Device_, data.renderFence, nullptr);
  }

  SDL_Vulkan_DestroySurface(Instance_, Surface_, nullptr);
  vkDestroyDevice(Device_, nullptr);
  vkDestroyInstance(Instance_, nullptr);
  SDL_DestroyWindow(Window_);
}

void Engine::CreateSwapchain() {
  constexpr auto format = VK_FORMAT_B8G8R8A8_UNORM;
  auto vkbSwapchain =
      vkb::SwapchainBuilder(physicalDevice_, Device_, Surface_)
          .set_desired_format(VkSurfaceFormatKHR{
              .format = format, .colorSpace = VK_COLORSPACE_SRGB_NONLINEAR_KHR})
          .set_desired_extent(WindowExtent_.width, WindowExtent_.height)
          .set_desired_present_mode(VK_PRESENT_MODE_FIFO_KHR)
          .add_image_usage_flags(VK_IMAGE_USAGE_TRANSFER_DST_BIT)
          .build()
          .value();

  Swapchain_ = Swapchain{.handle = vkbSwapchain.swapchain,
                         .extent = vkbSwapchain.extent,
                         .format = vkbSwapchain.image_format,
                         .images = vkbSwapchain.get_images().value(),
                         .views = vkbSwapchain.get_image_views().value()};
}

void Engine::DestroySwapchain() {
  vkDestroySwapchainKHR(Device_, Swapchain_.handle, nullptr);

  for (const auto view : Swapchain_.views) {
    vkDestroyImageView(Device_, view, nullptr);
  }
}

void Engine::InitializeFrameData() {
  const auto commandPoolInfo = util::commandPoolCreateInfo(
      GfxQueueFamilyIdx_, VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT);
  const auto fenceCreateInfo =
      util::fenceCreateInfo(VK_FENCE_CREATE_SIGNALED_BIT);
  const auto semaphoreCreateInfo = util::semaphoreCreateInfo();

  for (size_t i = 0; i < FrameData_.size(); i++) {
    FrameData &data = FrameData_[i];
    VK_CHECK(vkCreateCommandPool(Device_, &commandPoolInfo, nullptr,
                                 &data.commandPool));

    auto commandBufferAllocInfo =
        util::commandBufferAllocateInfo(data.commandPool, 1);
    VK_CHECK(vkAllocateCommandBuffers(Device_, &commandBufferAllocInfo,
                                      &data.commandBuffer));

    VK_CHECK(vkCreateSemaphore(Device_, &semaphoreCreateInfo, nullptr,
                               &data.renderSemaphore));
    VK_CHECK(vkCreateSemaphore(Device_, &semaphoreCreateInfo, nullptr,
                               &data.swapchainSemaphore));
    VK_CHECK(
        vkCreateFence(Device_, &fenceCreateInfo, nullptr, &data.renderFence));
  }
}

void Engine::draw() {
  FrameData &frame = GetCurrentFrame();
  VK_CHECK(vkWaitForFences(Device_, 1, &frame.renderFence, VK_TRUE,
                           std::numeric_limits<uint64_t>::max()));
  VK_CHECK(vkResetFences(Device_, 1, &frame.renderFence));

  uint32_t swapchainImageIdx;
  VK_CHECK(vkAcquireNextImageKHR(
      Device_, Swapchain_.handle, std::numeric_limits<uint64_t>::max(),
      frame.swapchainSemaphore, VK_NULL_HANDLE, &swapchainImageIdx));
  auto swapchainImage = Swapchain_.images.at(swapchainImageIdx);
  auto cmd = frame.commandBuffer;

  VK_CHECK(vkResetCommandBuffer(cmd, 0));

  const auto commandBufferBeginInfo =
      util::commandBufferBeginInfo(VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT);
  VK_CHECK(vkBeginCommandBuffer(cmd, &commandBufferBeginInfo));

  util::transitionImage(cmd, swapchainImage, VK_IMAGE_LAYOUT_UNDEFINED,
                        VK_IMAGE_LAYOUT_GENERAL);

  const VkClearColorValue clearColor = {.float32{
      0.0, 0.0, std::abs(std::sin(static_cast<float>(FrameNumber_) / 120.0f)),
      1.0}};

  const auto clearRange =
      util::imageSubresourceRange(VK_IMAGE_ASPECT_COLOR_BIT);

  vkCmdClearColorImage(cmd, swapchainImage, VK_IMAGE_LAYOUT_GENERAL,
                       &clearColor, 1, &clearRange);
  util::transitionImage(cmd, swapchainImage, VK_IMAGE_LAYOUT_GENERAL,
                        VK_IMAGE_LAYOUT_PRESENT_SRC_KHR);
  VK_CHECK(vkEndCommandBuffer(cmd));

  // Preperare to submit the command buffer to the queue.
  // Wait on frame.swapchainSemaphore because it is signaled when the swapchain
  // is ready.
  // To signal that rendering has finished we signal frame.renderSemaphore.

  const auto cmdInfo = util::commandBufferSubmitInfo(cmd);
  const auto waitInfo = util::semaphoreSubmitInfo(
      VK_PIPELINE_STAGE_2_COLOR_ATTACHMENT_OUTPUT_BIT_KHR,
      frame.swapchainSemaphore);
  const auto signalInfo = util::semaphoreSubmitInfo(
      VK_PIPELINE_STAGE_2_ALL_GRAPHICS_BIT, frame.renderSemaphore);

  const auto submitInfo = util::submitInfo(&cmdInfo, &signalInfo, &waitInfo);

  VK_CHECK(vkQueueSubmit2(GfxQueue_, 1, &submitInfo, frame.renderFence));

  // Wait on frame.renderSemaphore because it is signaled when all the drawing
  // commands are finished before presenting to the screen.
  VkPresentInfoKHR presentInfo{};
  presentInfo.sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR;
  presentInfo.pNext = nullptr;
  presentInfo.swapchainCount = 1;
  presentInfo.pSwapchains = &Swapchain_.handle;

  presentInfo.waitSemaphoreCount = 1;
  presentInfo.pWaitSemaphores = &frame.renderSemaphore;

  presentInfo.pImageIndices = &swapchainImageIdx;

  VK_CHECK(vkQueuePresentKHR(GfxQueue_, &presentInfo));
  FrameNumber_++;
}
