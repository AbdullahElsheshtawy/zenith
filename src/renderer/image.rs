use ash::vk;
use gpu_allocator::vulkan::{AllocationCreateDesc, AllocationScheme};

use super::{
    context::RenderContext, deletion_queue::DeletionQueue, transitionable::Transitionable, util,
};
#[derive(Debug)]
pub struct Image {
    image: vk::Image,
    view: vk::ImageView,
    extent: vk::Extent3D,
    format: vk::Format,
}

impl Image {
    pub fn new(
        rcx: &mut RenderContext,
        deletion_queue: &mut DeletionQueue,
        extent: vk::Extent3D,
        format: vk::Format,
        usage_flags: vk::ImageUsageFlags,
    ) -> anyhow::Result<Self> {
        let image_info = util::image_create_info(format, usage_flags, extent);

        let image = unsafe { rcx.device.create_image(&image_info, None) }?;
        let reqs = unsafe { rcx.device.get_image_memory_requirements(image) };

        let allocation = rcx.allocator.allocate(&AllocationCreateDesc {
            name: "Image",
            requirements: reqs,
            location: gpu_allocator::MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: AllocationScheme::DedicatedImage(image),
        })?;

        unsafe {
            rcx.device
                .bind_image_memory(image, allocation.memory(), allocation.offset())
        }?;

        let view = unsafe {
            rcx.device.create_image_view(
                &util::image_view_create_info(format, image, vk::ImageAspectFlags::COLOR),
                None,
            )
        }?;

        deletion_queue.push(Box::new(move |rcx: &mut RenderContext| {
            unsafe {
                rcx.device.destroy_image_view(view, None);
                rcx.device.destroy_image(image, None);
            }
            rcx.allocator.free(allocation).unwrap();
        }));
        Ok(Self {
            image,
            view,
            extent,
            format,
        })
    }

    pub fn view(&self) -> vk::ImageView {
        self.view
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
        rcx: &RenderContext,
        cmd_buf: vk::CommandBuffer,
        current_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        self.image
            .transition(rcx, cmd_buf, current_layout, new_layout);
    }
}
