#include "VulkanEngine.hpp"
#include "SDL3/SDL.h"
#include "SDL3/SDL_vulkan.h"
#include "VkBootstrap.h"
#include "VulkanInitializers.hpp"

VulkanEngine::VulkanEngine() {
  SDL_Init(SDL_INIT_VIDEO);
  Window_ = SDL_CreateWindow("zenith", WindowExtent_.width,
                             WindowExtent_.height, SDL_WINDOW_VULKAN);
  assert(Window_ && "Window cannot be null");

  initializeVulkan();
}

VulkanEngine::~VulkanEngine() {
  vkDestroySwapchainKHR(Device_, Swapchain_.swapchain, nullptr);
  for (const auto imageView : Swapchain_.imageViews) {
    vkDestroyImageView(Device_, imageView, nullptr);
  }

  for (uint32_t frame = 0; frame < FRAMES_IN_FLIGHT; frame++) {
    vkDestroyCommandPool(Device_, FrameData_[frame].CommandPool, nullptr);
  }

  SDL_Vulkan_DestroySurface(Instance_, Surface_, nullptr);
  vkDestroyDevice(Device_, nullptr);
  vkDestroyInstance(Instance_, nullptr);
  SDL_DestroyWindow(Window_);
}

void VulkanEngine::run() {
  SDL_Event event{};
  bool quit = false;
  bool stopRendering = false;
  while (!quit) {
    while (SDL_PollEvent(&event)) {
      if (event.type == SDL_EVENT_QUIT) {
        quit = true;
      }

      if (event.type == SDL_EVENT_WINDOW_MINIMIZED) {
        stopRendering = true;
        break;
      }
    }
    if (stopRendering) {
      std::this_thread::sleep_for(std::chrono::milliseconds(100));
      continue;
    }

    draw();
  }
}

void VulkanEngine::draw() {}

void VulkanEngine::initializeVulkan() {
  VK_CHECK(volkInitialize());
  auto vkbInstance =
      vkb::InstanceBuilder().require_api_version(1, 3).build().value();
  Instance_ = vkbInstance.instance;
  volkLoadInstance(Instance_);
  SDL_Vulkan_CreateSurface(Window_, Instance_, nullptr, &Surface_);

  VkPhysicalDeviceVulkan12Features features12{};
  features12.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_1_2_FEATURES;
  features12.bufferDeviceAddress = true;
  features12.descriptorIndexing = true;

  VkPhysicalDeviceVulkan13Features features13{};
  features12.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_1_3_FEATURES;
  features13.dynamicRendering = true;
  features13.synchronization2 = true;

  auto vkbPhysicalDevice = vkb::PhysicalDeviceSelector(vkbInstance, Surface_)
                               .set_minimum_version(1, 3)
                               .set_required_features_12(features12)
                               .set_required_features_13(features13)
                               .select()
                               .value();
  PhysicalDevice_ = vkbPhysicalDevice.physical_device;
  auto vkbDevice = vkb::DeviceBuilder(vkbPhysicalDevice).build().value();
  Device_ = vkbDevice.device;
  volkLoadDevice(Device_);

  GraphicsQueue_ = vkbDevice.get_queue(vkb::QueueType::graphics).value();
  GraphicsQueueFamilyIndex_ =
      vkbDevice.get_queue_index(vkb::QueueType::graphics).value();

  createSwapchain(WindowExtent_.width, WindowExtent_.height);
  inializeCommands();
}

void VulkanEngine::inializeCommands() {
  VkCommandPoolCreateInfo commandPoolInfo = VulkanInit::commandPoolCreateInfo(
      GraphicsQueueFamilyIndex_,
      VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT);
  for (uint32_t i = 0; i < FRAMES_IN_FLIGHT; i++) {
    VK_CHECK(vkCreateCommandPool(Device_, &commandPoolInfo, nullptr,
                                 &FrameData_[i].CommandPool));
    VkCommandBufferAllocateInfo commandBufferAllocateInfo =
        VulkanInit::commandBufferAllocateInfo(FrameData_[i].CommandPool, 1);

    VK_CHECK(vkAllocateCommandBuffers(Device_, &commandBufferAllocateInfo,
                                      &FrameData_[i].MainCommandBuffer));
  }
}

void VulkanEngine::createSwapchain(uint32_t width, uint32_t height) {
  auto vkbSwapchain =
      vkb::SwapchainBuilder(PhysicalDevice_, Device_, Surface_)
          .set_desired_format(VkSurfaceFormatKHR{
              .format = VK_FORMAT_B8G8R8A8_UNORM,
              .colorSpace = VK_COLORSPACE_SRGB_NONLINEAR_KHR})
          .set_desired_present_mode(VK_PRESENT_MODE_MAILBOX_KHR)
          .add_fallback_present_mode(VK_PRESENT_MODE_FIFO_KHR)
          .set_desired_extent(width, height)
          .add_image_usage_flags(VK_IMAGE_USAGE_TRANSFER_DST_BIT)
          .build()
          .value();

  Swapchain_.swapchain = vkbSwapchain.swapchain;
  Swapchain_.extent = vkbSwapchain.extent;
  Swapchain_.images = vkbSwapchain.get_images().value();
  Swapchain_.imageViews = vkbSwapchain.get_image_views().value();
  Swapchain_.format = vkbSwapchain.image_format;
}