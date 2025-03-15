use std::sync::Arc;

use ash::vk;
pub struct DescriptorLayoutBuilder<'a> {
    bindings: Vec<vk::DescriptorSetLayoutBinding<'a>>,
}

impl<'a> DescriptorLayoutBuilder<'a> {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn add_binding(mut self, binding: u32, kind: vk::DescriptorType) -> Self {
        self.bindings.push(
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(kind)
                .descriptor_count(1),
        );

        self
    }

    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    pub fn build(
        mut self,
        device: &ash::Device,
        shader_stages: vk::ShaderStageFlags,
        push_next: Option<&mut dyn vk::ExtendsDescriptorSetLayoutCreateInfo>,
        flags: vk::DescriptorSetLayoutCreateFlags,
    ) -> Result<vk::DescriptorSetLayout, vk::Result> {
        for binding in &mut self.bindings {
            binding.stage_flags |= shader_stages;
        }

        let mut info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&self.bindings)
            .flags(flags);

        if let Some(next) = push_next {
            info = info.push_next(next);
        }

        unsafe { device.create_descriptor_set_layout(&info, None) }
    }
}

#[derive(Debug, Default)]
pub struct PoolSizeRatio {
    pub ty: vk::DescriptorType,
    pub ratio: f32,
}

#[derive(Clone, Copy)]
pub struct DescriptorAllocator {
    pool: vk::DescriptorPool,
}

impl DescriptorAllocator {
    pub fn new(
        device: &ash::Device,
        max_sets: u32,
        pool_ratios: &[PoolSizeRatio],
    ) -> anyhow::Result<Self> {
        let pool_sizes = pool_ratios
            .iter()
            .map(|ratio| {
                vk::DescriptorPoolSize::default()
                    .ty(ratio.ty)
                    .descriptor_count(ratio.ratio.round() as u32 * max_sets)
            })
            .collect::<Vec<vk::DescriptorPoolSize>>();

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(max_sets)
            .pool_sizes(&pool_sizes);

        Ok(Self {
            pool: unsafe { device.create_descriptor_pool(&pool_info, None) }?,
        })
    }

    pub fn clear_descriptors(&self, device: &ash::Device) -> Result<(), vk::Result> {
        unsafe { device.reset_descriptor_pool(self.pool, vk::DescriptorPoolResetFlags::empty()) }
    }

    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_descriptor_pool(self.pool, None);
        }
    }

    pub fn allocate(
        &self,
        device: &ash::Device,
        layout: vk::DescriptorSetLayout,
    ) -> Result<vk::DescriptorSet, vk::Result> {
        let layout = [layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.pool)
            .set_layouts(&layout);

        Ok(unsafe { device.allocate_descriptor_sets(&alloc_info) }?[0])
    }
}
