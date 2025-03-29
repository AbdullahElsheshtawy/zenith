#pragma once
#include "types.hpp"
#include <span>

class DescriptorLayoutBuilder {
  std::vector<VkDescriptorSetLayoutBinding> bindings;

public:
  void addBinding(uint32_t binding, VkDescriptorType type);
  void clear();
  VkDescriptorSetLayout build(VkDevice device, VkShaderStageFlags shaderStages,
                              void *pNext = nullptr,
                              VkDescriptorSetLayoutCreateFlags flags = 0);
};

struct DescriptorAllocator {
  struct PoolSizeRatio {
    VkDescriptorType type;
    float ratio;
  };
  VkDescriptorPool pool;

  void initializePool(VkDevice device, uint32_t maxSets,
                      std::span<PoolSizeRatio> poolSizeRatios);
  void clearDescriptors(VkDevice device) const;
  void destroyPool(VkDevice device) const;

  VkDescriptorSet allocate(VkDevice device, VkDescriptorSetLayout layout) const;
};