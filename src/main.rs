use std::sync::Arc;

use ik_creatures::renderer::{
    circles::{CirclePipeline, RawInstance},
    Renderer,
};
use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
};

fn main() {
    println!("Hello, world!");

    env_logger::Builder::new()
        .filter_module("wgpu", log::LevelFilter::Warn)
        .filter_module("image_manager", log::LevelFilter::Trace)
        .format_timestamp(None)
        .init();

    let mut app = Runner::new();
    let event_loop = EventLoop::new().unwrap();
    match event_loop.run_app(&mut app) {
        Ok(_) => {}
        Err(e) => println!("Error on close: {}", e),
    };
}

struct Runner {
    inner: Option<App>,
}

impl Runner {
    pub fn new() -> Self {
        Self { inner: None }
    }
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.inner = Some(App::new(event_loop));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(inner) = &mut self.inner {
            inner.window_event(event_loop, window_id, event);
        }
    }

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        let _ = (event_loop, cause);
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }
}

//====================================================================

pub struct App {
    renderer: Renderer,
    circles: CirclePipeline,
}

impl App {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(winit::window::Window::default_attributes())
                .unwrap(),
        );

        let mut renderer = Renderer::new(window).block_on().unwrap();
        let circles = renderer.create_pipeline();

        Self { renderer, circles }
    }

    pub fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::Resized(new_size) => self.resize(new_size),

            winit::event::WindowEvent::Destroyed => log::error!("Window was destroyed"), // panic!("Window was destroyed"),
            winit::event::WindowEvent::CloseRequested => {
                log::info!("Close requested. Closing App.");
                event_loop.exit();
            }

            winit::event::WindowEvent::RedrawRequested => self.tick(),

            // winit::event::WindowEvent::KeyboardInput { event, .. } => {
            //     if let winit::keyboard::PhysicalKey::Code(key) = event.physical_key {
            //         self.world.run_with_data(
            //             tools::sys_process_keypress,
            //             (key, event.state.is_pressed()),
            //         );
            //     }
            // }

            // winit::event::WindowEvent::ActivationTokenDone { serial, token } => todo!(),
            // winit::event::WindowEvent::Moved(_) => todo!(),
            // winit::event::WindowEvent::DroppedFile(_) => todo!(),
            // winit::event::WindowEvent::HoveredFile(_) => todo!(),
            // winit::event::WindowEvent::HoveredFileCancelled => todo!(),
            // winit::event::WindowEvent::Focused(_) => todo!(),
            // winit::event::WindowEvent::ModifiersChanged(_) => todo!(),
            // winit::event::WindowEvent::Ime(_) => todo!(),
            // winit::event::WindowEvent::CursorMoved { device_id, position } => todo!(),
            // winit::event::WindowEvent::CursorEntered { device_id } => todo!(),
            // winit::event::WindowEvent::CursorLeft { device_id } => todo!(),
            // winit::event::WindowEvent::MouseWheel { device_id, delta, phase } => todo!(),
            // winit::event::WindowEvent::MouseInput { device_id, state, button } => todo!(),
            // winit::event::WindowEvent::PinchGesture { device_id, delta, phase } => todo!(),
            // winit::event::WindowEvent::PanGesture { device_id, delta, phase } => todo!(),
            // winit::event::WindowEvent::DoubleTapGesture { device_id } => todo!(),
            // winit::event::WindowEvent::RotationGesture { device_id, delta, phase } => todo!(),
            // winit::event::WindowEvent::TouchpadPressure { device_id, pressure, stage } => todo!(),
            // winit::event::WindowEvent::AxisMotion { device_id, axis, value } => todo!(),
            // winit::event::WindowEvent::Touch(_) => todo!(),
            // winit::event::WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => todo!(),
            // winit::event::WindowEvent::ThemeChanged(_) => todo!(),
            // winit::event::WindowEvent::Occluded(_) => todo!(),
            _ => {}
        }
    }

    fn resize(&mut self, _size: winit::dpi::PhysicalSize<u32>) {}

    fn tick(&mut self) {
        self.renderer.update_pipeline(
            &mut self.circles,
            &[RawInstance {
                pos: [0., 0.],
                radius: 500.,
                border_radius: 5.,
                color: [1., 1., 1., 1.],
                border_color: [0., 0., 0., 1.],
            }],
        );

        self.renderer.render(&mut [&mut self.circles]).unwrap();
    }
}

//====================================================================
