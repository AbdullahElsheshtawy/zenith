mod renderer;
use renderer::Renderer;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes},
};

pub struct App<'a> {
    window: Arc<Window>,
    renderer: Renderer<'a>,
}

impl App<'_> {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> anyhow::Result<Self> {
        let window =
            Arc::new(event_loop.create_window(WindowAttributes::default().with_title("Zenith"))?);
        let renderer = Renderer::new(window.clone())?;
        Ok(App { window, renderer })
    }

    fn window_event(&mut self, event: winit::event::WindowEvent) -> bool {
        use winit::event::WindowEvent;
        match event {
            WindowEvent::CloseRequested => false,
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                use winit::keyboard::KeyCode;
                match key {
                    KeyCode::Escape => false,
                    _ => true,
                }
            }
            WindowEvent::RedrawRequested => {
                self.renderer.draw();
                true
            }
            _ => true,
        }
    }
}
pub enum AppState<'a> {
    Initializing,
    Running(App<'a>),
    Closing,
}

impl AppState<'_> {
    pub fn new() -> Self {
        AppState::Initializing
    }
}

impl Default for AppState<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler for AppState<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        *self = AppState::Running(App::new(event_loop).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let AppState::Running(app) = self else {
            return;
        };

        match app.window_event(event) {
            true => app.window.request_redraw(),
            false => {
                *self = AppState::Closing;
                event_loop.exit();
            }
        };
    }
}
