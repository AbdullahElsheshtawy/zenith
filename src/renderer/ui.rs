use std::collections::VecDeque;

use ash::vk;
use winit::window::Window;
use yakui::widgets::Pad;
use yakui::{Alignment, Color, use_state, widgets::Text};
use yakui_vulkan::{Options, YakuiVulkan};
use yakui_winit::YakuiWinit;

use crate::renderer::Renderer;

use super::context::RenderContext;
pub struct Ui {
    yak: yakui::Yakui,
    winit: YakuiWinit,
    renderer: YakuiVulkan,
    memory_props: vk::PhysicalDeviceMemoryProperties,
}

impl Ui {
    pub fn new(window: &Window, rcx: &RenderContext, format: vk::Format) -> Self {
        let window_size = window.inner_size();
        let mut yak = yakui::Yakui::new();
        yak.set_surface_size([window_size.width as f32, window_size.height as f32].into());
        yak.set_unscaled_viewport(yakui::Rect::from_pos_size(
            Default::default(),
            [window_size.width as f32, window_size.height as f32].into(),
        ));
        let winit = YakuiWinit::new(&window);
        let memory_props = unsafe {
            rcx.instance
                .get_physical_device_memory_properties(rcx.physical_device)
        };
        let yak_vk_ctx = yakui_vulkan::VulkanContext {
            device: &rcx.device,
            queue: rcx.gfx_queue,
            memory_properties: memory_props,
        };

        let mut options = Options::default();
        options.dynamic_rendering_format = Some(format);
        options.render_pass = vk::RenderPass::null();

        let mut renderer = YakuiVulkan::new(&yak_vk_ctx, options);
        for _ in 0..Renderer::FRAMES_IN_FLIGHT {
            renderer.transfers_submitted();
        }
        Self {
            yak,
            winit,
            renderer,
            memory_props,
        }
    }

    pub fn start(&mut self) {
        self.yak.start();
    }

    pub fn finish(&mut self, cmd_buf: vk::CommandBuffer, rcx: &RenderContext) {
        self.yak.finish();
        let paint = self.yak.paint();
        let yak_vk_ctx =
            yakui_vulkan::VulkanContext::new(&rcx.device, rcx.gfx_queue, self.memory_props);

        unsafe { self.renderer.transfers_finished(&yak_vk_ctx) };
        unsafe { self.renderer.transfer(&paint, &yak_vk_ctx, cmd_buf) };
        self.renderer.transfers_submitted();
    }

    pub fn render(
        &mut self,
        rcx: &RenderContext,
        cmd_buf: vk::CommandBuffer,
        extent: vk::Extent2D,
    ) {
        let paint = self.yak.paint();
        let yak_vk_ctx =
            yakui_vulkan::VulkanContext::new(&rcx.device, rcx.gfx_queue, self.memory_props);
        unsafe {
            self.renderer.paint(&paint, &yak_vk_ctx, cmd_buf, extent);
        }
    }

    pub fn window_event(&mut self, window_event: &winit::event::WindowEvent) {
        self.winit.handle_window_event(&mut self.yak, window_event);
    }

    pub fn destroy(&mut self, rcx: &RenderContext) {
        unsafe { self.renderer.cleanup(&rcx.device) };
    }
}

const FONT_SIZE: f32 = 16.0;
const TEXT_COLOR: Color = Color::CORNFLOWER_BLUE;

pub fn fps_counter() {
    let now = use_state(|| std::time::Instant::now());
    let new_now = std::time::Instant::now();
    let delta = new_now - now.get();
    now.set(new_now);
    let window = use_state(VecDeque::new);
    let avg = {
        let mut window = window.borrow_mut();
        window.push_back(delta.as_secs_f32());

        while window.len() > 120 {
            window.pop_front();
        }
        window.iter().sum::<f32>() / window.len() as f32
    };

    let fps = 1.0 / avg;
    let ms = avg * 1000.0;
    yakui::align(Alignment::TOP_LEFT, || {
        let mut text = Text::new(FONT_SIZE, format!("FPS: {fps:.0} ({ms:.2} ms)"));
        text.style.color = TEXT_COLOR;
        text.show();
    });
}
