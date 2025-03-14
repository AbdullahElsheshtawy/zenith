use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};

use super::{transitionable::Transitionable, util};
#[derive(Debug)]
pub struct Image {
    image: vk::Image,
    view: vk::ImageView,
    extent: vk::Extent3D,
    format: vk::Format,
    allocation: Option<Allocation>,
}

impl Image {
    pub fn new(
        device: &ash::Device,
        allocator: &mut Allocator,
        extent: vk::Extent3D,
        format: vk::Format,
        usage_flags: vk::ImageUsageFlags,
    ) -> anyhow::Result<Self> {
        let image_info = util::image_create_info(format, usage_flags, extent);

        let image = unsafe { device.create_image(&image_info, None) }?;
        let reqs = unsafe { device.get_image_memory_requirements(image) };

        let allocation = allocator.allocate(&AllocationCreateDesc {
            name: "Image",
            requirements: reqs,
            location: gpu_allocator::MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: AllocationScheme::DedicatedImage(image),
        })?;

        unsafe { device.bind_image_memory(image, allocation.memory(), allocation.offset()) }?;

        let view = unsafe {
            device.create_image_view(
                &util::image_view_create_info(format, image, vk::ImageAspectFlags::COLOR),
                None,
            )
        }?;

        Ok(Self {
            image,
            view,
            extent,
            format,
            allocation: Some(allocation),
        })
    }

    pub fn destroy(&mut self, device: &ash::Device, allocator: &mut Allocator) {
        unsafe {
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
        }
        if let Some(allocation) = self.allocation.take() {
            allocator.free(allocation).unwrap();
        }
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }
    pub fn extent(&self) -> vk::Extent3D {
        self.extent
    }

    pub fn image(&self) -> vk::Image {
        self.image
    }
}

impl Transitionable for Image {
    fn transition(
        &self,
        device: &ash::Device,
        cmd_buf: vk::CommandBuffer,
        current_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        self.image
            .transition(device, cmd_buf, current_layout, new_layout);
    }
}
