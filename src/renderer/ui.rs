use std::mem::ManuallyDrop;

use ash::vk;
use egui::ViewportId;
use winit::window::Window;

use crate::renderer::Renderer;

use super::context::RenderContext;

pub struct Ui {
    pub ctx: egui::Context,
    pub egui_winit: egui_winit::State,
    pub renderer: ManuallyDrop<egui_ash_renderer::Renderer>,
}

impl Ui {
    pub fn new(window: &Window, rcx: &RenderContext, format: vk::Format) -> anyhow::Result<Self> {
        let ctx = egui::Context::default();
        let egui_winit =
            egui_winit::State::new(ctx.clone(), ViewportId::ROOT, &window, None, None, None);
        let renderer = ManuallyDrop::new(egui_ash_renderer::Renderer::with_default_allocator(
            &rcx.instance,
            rcx.physical_device,
            rcx.device.clone(),
            egui_ash_renderer::DynamicRendering {
                color_attachment_format: format,
                depth_attachment_format: None,
            },
            egui_ash_renderer::Options {
                in_flight_frames: Renderer::FRAMES_IN_FLIGHT,
                ..Default::default()
            },
        )?);
        Ok(Self {
            ctx,
            egui_winit,
            renderer,
        })
    }
}
