use gpu_allocator::vulkan::Allocator;
pub struct RenderContext {
    pub device: ash::Device,
    pub allocator: Allocator,
}

impl RenderContext {
    pub fn new(device: ash::Device, allocator: Allocator) -> Self {
        Self { device, allocator }
    }

    pub fn destroy(&mut self) {
        unsafe { self.device.destroy_device(None) };
    }
}
