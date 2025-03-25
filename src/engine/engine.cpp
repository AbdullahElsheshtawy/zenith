#include "engine.hpp"
#include "SDL3/SDL.h"
#include "SDL3/SDL_vulkan.h"
#include "VkBootstrap.h"
#include "util.hpp"
#include "vma.hpp"

Engine::Engine(uint32_t width, uint32_t height) : WindowExtent_{width, height} {
  SDL_Init(SDL_INIT_VIDEO);
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
  InitializeCommands();

  VmaVulkanFunctions vulkanFunctions = {
      .vkGetInstanceProcAddr = vkGetInstanceProcAddr,
      .vkGetDeviceProcAddr = vkGetDeviceProcAddr,
      .vkGetPhysicalDeviceProperties = vkGetPhysicalDeviceProperties,
      .vkGetPhysicalDeviceMemoryProperties =
          vkGetPhysicalDeviceMemoryProperties,
      .vkAllocateMemory = vkAllocateMemory,
      .vkFreeMemory = vkFreeMemory,
      .vkMapMemory = vkMapMemory,
      .vkUnmapMemory = vkUnmapMemory,
      .vkFlushMappedMemoryRanges = vkFlushMappedMemoryRanges,
      .vkInvalidateMappedMemoryRanges = vkInvalidateMappedMemoryRanges,
      .vkBindBufferMemory = vkBindBufferMemory,
      .vkBindImageMemory = vkBindImageMemory,
      .vkGetBufferMemoryRequirements = vkGetBufferMemoryRequirements,
      .vkGetImageMemoryRequirements = vkGetImageMemoryRequirements,
      .vkCreateBuffer = vkCreateBuffer,
      .vkDestroyBuffer = vkDestroyBuffer,
      .vkCreateImage = vkCreateImage,
      .vkDestroyImage = vkDestroyImage,
      .vkCmdCopyBuffer = vkCmdCopyBuffer,
      .vkGetBufferMemoryRequirements2KHR = vkGetBufferMemoryRequirements2,
      .vkGetImageMemoryRequirements2KHR = vkGetImageMemoryRequirements2,
      .vkBindBufferMemory2KHR = vkBindBufferMemory2,
      .vkBindImageMemory2KHR = vkBindImageMemory2,
      .vkGetPhysicalDeviceMemoryProperties2KHR =
          vkGetPhysicalDeviceMemoryProperties2,
      .vkGetDeviceBufferMemoryRequirements =
          vkGetDeviceBufferMemoryRequirements,
  };
  VmaAllocatorCreateInfo allocatorInfo{};
  allocatorInfo.instance = Instance_;
  allocatorInfo.physicalDevice = physicalDevice_;
  allocatorInfo.device = Device_;
  allocatorInfo.flags = VMA_ALLOCATOR_CREATE_BUFFER_DEVICE_ADDRESS_BIT;
  allocatorInfo.pVulkanFunctions = &vulkanFunctions;
  allocatorInfo.vulkanApiVersion = VK_API_VERSION_1_3;
  vmaCreateAllocator(&allocatorInfo, &Allocator_);
  DeletionQueue_.Push([&]() { vmaDestroyAllocator(Allocator_); });

  DrawImage_.format = VK_FORMAT_R16G16B16A16_SFLOAT;
  DrawImage_.extent = {
      .width = WindowExtent_.width,
      .height = WindowExtent_.height,
      .depth = 1,
  };

  const VkImageUsageFlags drawImageUsages =
      VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT | VK_IMAGE_USAGE_TRANSFER_SRC_BIT |
      VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_STORAGE_BIT;

  const auto drawImageCreateInfo = util::imageCreateInfo(
      DrawImage_.format, drawImageUsages, DrawImage_.extent);

  const VmaAllocationCreateInfo drawImageAllocationInfo{
      .usage = VMA_MEMORY_USAGE_GPU_ONLY,
      .requiredFlags =
          VkMemoryPropertyFlags(VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT)};
  vmaCreateImage(Allocator_, &drawImageCreateInfo, &drawImageAllocationInfo,
                 &DrawImage_.handle, &DrawImage_.allocation, nullptr);
  const auto drawImageViewInfo = util::imageViewCreateInfo(
      DrawImage_.format, DrawImage_.handle, VK_IMAGE_ASPECT_COLOR_BIT);
  VK_CHECK(vkCreateImageView(Device_, &drawImageViewInfo, nullptr,
                             &DrawImage_.view));

  DeletionQueue_.Push([&]() {
    vkDestroyImageView(Device_, DrawImage_.view, nullptr);
    vmaDestroyImage(Allocator_, DrawImage_.handle, DrawImage_.allocation);
  });

  InitializeImgui();
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

      ImGui_ImplSDL3_ProcessEvent(&event);
    }

    if (stop_rendering) {
      std::this_thread::sleep_for(std::chrono::milliseconds(100));
      continue;
    }
    // imgui
    ImGui_ImplVulkan_NewFrame();
    ImGui_ImplSDL3_NewFrame();
    ImGui::NewFrame();

    ImGui::ShowDemoWindow();
    ImGui::Render();

    Draw();
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

    data.deletionQueue.Flush();
  }

  DeletionQueue_.Flush();
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

void Engine::InitializeCommands() {
  const auto commandPoolInfo = util::commandPoolCreateInfo(
      GfxQueueFamilyIdx_, VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT);
  const auto fenceCreateInfo =
      util::fenceCreateInfo(VK_FENCE_CREATE_SIGNALED_BIT);
  const auto semaphoreCreateInfo = util::semaphoreCreateInfo();

  for (size_t i = 0; i < FRAMES_IN_FLIGHT; i++) {
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
  // Immediate
  VK_CHECK(vkCreateCommandPool(Device_, &commandPoolInfo, nullptr,
                               &Immediate_.commandPool));
  const auto immBufferAllocInfo =
      util::commandBufferAllocateInfo(Immediate_.commandPool, 1);
  VK_CHECK(vkAllocateCommandBuffers(Device_, &immBufferAllocInfo,
                                    &Immediate_.commandBuffer));
  VK_CHECK(
      vkCreateFence(Device_, &fenceCreateInfo, nullptr, &Immediate_.fence));
  DeletionQueue_.Push([&]() {
    vkDestroyCommandPool(Device_, Immediate_.commandPool, nullptr);
    vkDestroyFence(Device_, Immediate_.fence, nullptr);
  });
}

void Engine::InitializeImgui() {
  std::array<VkDescriptorPoolSize, 1> poolSizes = {
      VkDescriptorPoolSize{
          .type = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
          .descriptorCount =
              IMGUI_IMPL_VULKAN_MINIMUM_IMAGE_SAMPLER_POOL_SIZE + 10},
  };
  VkDescriptorPoolCreateInfo poolInfo = {
      .sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
      .pNext = nullptr,
      .flags = VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT,
      .maxSets = 0,
  };
  for (auto &poolSize : poolSizes) {
    poolInfo.maxSets += poolSize.descriptorCount;
  }
  poolInfo.poolSizeCount = static_cast<uint32_t>(poolSizes.size());
  poolInfo.pPoolSizes = poolSizes.data();

  VkDescriptorPool imguiPool;
  VK_CHECK(vkCreateDescriptorPool(Device_, &poolInfo, nullptr, &imguiPool));

  ImGui::CreateContext();
  ImGui_ImplSDL3_InitForVulkan(Window_);

  ImGui_ImplVulkan_InitInfo initInfo{};
  initInfo.Instance = Instance_;
  initInfo.PhysicalDevice = physicalDevice_;
  initInfo.Device = Device_;
  initInfo.Queue = GfxQueue_;
  initInfo.DescriptorPool = imguiPool;
  initInfo.MinImageCount = 3;
  initInfo.ImageCount = 3;
  initInfo.UseDynamicRendering = true;

  // Dynamic rendering parameters that imgui needs
  initInfo.PipelineRenderingCreateInfo = {
      .sType = VK_STRUCTURE_TYPE_PIPELINE_RENDERING_CREATE_INFO,
      .colorAttachmentCount = 1,
      .pColorAttachmentFormats = &Swapchain_.format};

  initInfo.MSAASamples = VK_SAMPLE_COUNT_1_BIT;

  ImGui_ImplVulkan_Init(&initInfo);
  ImGui_ImplVulkan_CreateFontsTexture();

  DeletionQueue_.Push([=, this]() {
    ImGui_ImplVulkan_Shutdown();
    vkDestroyDescriptorPool(Device_, imguiPool, nullptr);
  });
}

void Engine::Draw() {
  FrameData &frame = GetCurrentFrame();
  VK_CHECK(vkWaitForFences(Device_, 1, &frame.renderFence, VK_TRUE,
                           std::numeric_limits<uint64_t>::max()));
  frame.deletionQueue.Flush();
  VK_CHECK(vkResetFences(Device_, 1, &frame.renderFence));

  uint32_t swapchainImageIdx;
  VK_CHECK(vkAcquireNextImageKHR(
      Device_, Swapchain_.handle, std::numeric_limits<uint64_t>::max(),
      frame.swapchainSemaphore, VK_NULL_HANDLE, &swapchainImageIdx));
  const auto swapchainImage = Swapchain_.images.at(swapchainImageIdx);
  const auto swapchainImageView = Swapchain_.views.at(swapchainImageIdx);
  auto cmd = frame.commandBuffer;

  VK_CHECK(vkResetCommandBuffer(cmd, 0));

  const auto commandBufferBeginInfo =
      util::commandBufferBeginInfo(VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT);
  DrawExtent_ = {DrawImage_.extent.width, DrawImage_.extent.height};
  VK_CHECK(vkBeginCommandBuffer(cmd, &commandBufferBeginInfo));

  util::transitionImage(cmd, DrawImage_.handle, VK_IMAGE_LAYOUT_UNDEFINED,
                        VK_IMAGE_LAYOUT_GENERAL);
  DrawBackground(cmd);

  util::transitionImage(cmd, DrawImage_.handle, VK_IMAGE_LAYOUT_GENERAL,
                        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL);
  util::transitionImage(cmd, swapchainImage, VK_IMAGE_LAYOUT_UNDEFINED,
                        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL);

  util::copyImageToImage(cmd, DrawImage_.handle, swapchainImage, DrawExtent_,
                         Swapchain_.extent);

  DrawImgui(cmd, swapchainImageView);

  util::transitionImage(cmd, swapchainImage,
                        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
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

void Engine::DrawBackground(VkCommandBuffer cmd) const {
  const VkClearColorValue clearColor = {.float32{
      0.0, 0.0, std::abs(std::sin(static_cast<float>(FrameNumber_) / 120.0f)),
      1.0}};

  const auto clearRange =
      util::imageSubresourceRange(VK_IMAGE_ASPECT_COLOR_BIT);

  vkCmdClearColorImage(cmd, DrawImage_.handle, VK_IMAGE_LAYOUT_GENERAL,
                       &clearColor, 1, &clearRange);
}

void Engine::DrawImgui(VkCommandBuffer cmd, VkImageView targetImageView) const {
  const auto colorAttachment = util::attachementInfo(targetImageView, nullptr);
  const auto renderInfo =
      util::renderingInfo(Swapchain_.extent, &colorAttachment, nullptr);

  vkCmdBeginRendering(cmd, &renderInfo);
  ImGui_ImplVulkan_RenderDrawData(ImGui::GetDrawData(), cmd);
  vkCmdEndRendering(cmd);
}

void Engine::ImmediateSubmit(
    std::function<void(VkCommandBuffer cmd)> &&function) const {

  VK_CHECK(vkResetFences(Device_, 1, &Immediate_.fence));
  VK_CHECK(vkResetCommandBuffer(Immediate_.commandBuffer, 0));

  auto cmd = Immediate_.commandBuffer;
  const auto cmdBeginInfo =
      util::commandBufferBeginInfo(VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT);
  VK_CHECK(vkBeginCommandBuffer(cmd, &cmdBeginInfo));
  function(cmd);
  VK_CHECK(vkEndCommandBuffer(cmd));

  const auto cmdSubmitInfo = util::commandBufferSubmitInfo(cmd);
  const auto submitInfo = util::submitInfo(&cmdSubmitInfo, nullptr, nullptr);

  VK_CHECK(vkQueueSubmit2(GfxQueue_, 1, &submitInfo, Immediate_.fence));
  VK_CHECK(vkWaitForFences(Device_, 1, &Immediate_.fence, true,
                           std::numeric_limits<uint64_t>::max()));
}
