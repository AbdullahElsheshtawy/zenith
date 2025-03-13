#[derive(Clone)]
pub struct Loaders {
    pub swapchain: ash::khr::swapchain::Device,
    pub surface: ash::khr::surface::Instance,
}

impl Loaders {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, device: &ash::Device) -> Self {
        Self {
            swapchain: ash::khr::swapchain::Device::new(instance, device),
            surface: ash::khr::surface::Instance::new(entry, instance),
        }
    }
}
