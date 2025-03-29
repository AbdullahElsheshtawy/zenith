#include "descriptors.hpp"

void DescriptorLayoutBuilder::addBinding(uint32_t binding,
                                         VkDescriptorType type) {
  bindings.push_back({
      .binding = binding,
      .descriptorType = type,
      .descriptorCount = 1,
  });
}

void DescriptorLayoutBuilder::clear() { bindings.clear(); }

VkDescriptorSetLayout
DescriptorLayoutBuilder::build(VkDevice device, VkShaderStageFlags shaderStages,
                               void *pNext,
                               VkDescriptorSetLayoutCreateFlags flags) {
  for (auto &binding : bindings) {
    binding.stageFlags |= shaderStages;
  }

  const VkDescriptorSetLayoutCreateInfo info = {
      .sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
      .pNext = pNext,
      .flags = flags,
      .bindingCount = static_cast<uint32_t>(bindings.size()),
      .pBindings = bindings.data(),
  };
  VkDescriptorSetLayout setLayout;
  VK_CHECK(vkCreateDescriptorSetLayout(device, &info, nullptr, &setLayout));
  return setLayout;
}

void DescriptorAllocator::initializePool(
    VkDevice device, uint32_t maxSets,
    std::span<PoolSizeRatio> poolSizeRatios) {
  std::vector<VkDescriptorPoolSize> poolSizes;

  for (PoolSizeRatio ratio : poolSizeRatios) {
    poolSizes.emplace_back(VkDescriptorPoolSize{
        .type = ratio.type,
        .descriptorCount = static_cast<uint32_t>(ratio.ratio * maxSets)});
  }

  const VkDescriptorPoolCreateInfo info{
      .sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
      .pNext = nullptr,
      .flags = 0,
      .maxSets = maxSets,
      .poolSizeCount = static_cast<uint32_t>(poolSizes.size()),
      .pPoolSizes = poolSizes.data()};

  vkCreateDescriptorPool(device, &info, nullptr, &pool);
}

void DescriptorAllocator::clearDescriptors(VkDevice device) const {
  vkResetDescriptorPool(device, pool, 0);
}

void DescriptorAllocator::destroyPool(VkDevice device) const {
  vkDestroyDescriptorPool(device, pool, nullptr);
}

VkDescriptorSet
DescriptorAllocator::allocate(VkDevice device,
                              VkDescriptorSetLayout layout) const {
  const VkDescriptorSetAllocateInfo info = {
      .sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
      .pNext = nullptr,
      .descriptorPool = pool,
      .descriptorSetCount = 1,
      .pSetLayouts = &layout};

  VkDescriptorSet set;
  VK_CHECK(vkAllocateDescriptorSets(device, &info, &set));
  return set;
}
