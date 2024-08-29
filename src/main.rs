//====================================================================

use std::sync::Arc;

use glam::{vec2, Vec2};
use ik_creatures::{
    ik::{self, Skeleton},
    renderer::{
        circles::CirclePipeline, polygon::PolygonPipeline, text::TextPipeline,
        uniques::OrthographicCamera, Renderer,
    },
};
use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
};

//====================================================================

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

//====================================================================

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
    window: Arc<winit::window::Window>,
    renderer: Renderer,
    circles: CirclePipeline,
    text: TextPipeline,
    polygons: PolygonPipeline,

    camera: OrthographicCamera,
    mouse_pos: Vec2,
    mouse_vector: Vec2,
    mouse_down: bool,

    skeleton: Skeleton,
}

impl App {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(winit::window::Window::default_attributes())
                .unwrap(),
        );

        let mut renderer = Renderer::new(window.clone()).block_on().unwrap();
        let circles = renderer.create_pipeline();
        let text = renderer.create_pipeline();
        let polygons = renderer.create_pipeline();

        let camera = OrthographicCamera::default();
        renderer.update_camera(0, &camera);

        let mut skeleton = Skeleton::new();
        ik::spawn_creature(&mut skeleton);
        // ik::spawn_arm(&mut skeleton);

        Self {
            window,
            renderer,
            circles,
            text,
            polygons,
            camera,
            mouse_pos: Vec2::ZERO,
            mouse_vector: Vec2::ZERO,
            mouse_down: false,
            skeleton,
        }
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
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let size = self.window.inner_size();
                let pos = vec2(
                    position.x as f32,
                    (size.height as f32) - (position.y as f32),
                );

                let half_size = vec2(size.width as f32 / 2., size.height as f32 / 2.);
                let relative_pos = pos - half_size + self.camera.translation.truncate();

                self.mouse_vector = self.mouse_vector.lerp(relative_pos - self.mouse_pos, 0.5);
                self.mouse_pos = relative_pos;
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => match button {
                winit::event::MouseButton::Left => {
                    self.mouse_down = state.is_pressed();
                }
                _ => {}
            },

            _ => {}
        }
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        let half_width = size.width as f32 / 2.;
        let half_height = size.height as f32 / 2.;

        self.camera.left = -half_width;
        self.camera.right = half_width;

        self.camera.top = half_height;
        self.camera.bottom = -half_height;

        self.renderer.update_camera(0, &self.camera);

        self.renderer
            .resize_pipeline(&mut self.text, size.width, size.height);
    }

    fn tick(&mut self) {
        if self.mouse_down {
            if let Some(mut root) = self.skeleton.get_node_mut(0) {
                root.pos = self.mouse_pos;
                root.set_rotation(self.mouse_vector.to_angle());
            }
        }

        if let Some(ik) = self.skeleton.get_ik(0) {
            ik.target = self.mouse_pos;
        }

        self.skeleton.tick();

        let circle_instances = self.skeleton.circles();

        self.renderer
            .update_pipeline(&mut self.circles, circle_instances.as_slice());

        // let mesh_nodes = self.skeleton.triangle_list();
        // let (vertices, indices) = polygon::calculate_strip(&mesh_nodes[0]);

        // self.renderer.update_pipeline(
        //     &mut self.polygons,
        //     &[(vertices.as_slice(), indices.as_slice())],
        // );

        self.renderer
            .render(&mut [
                // -
                &mut self.polygons,
                &mut self.circles,
                &mut self.text,
            ])
            .unwrap();

        self.window.request_redraw();

        self.text.trim();
    }
}

//====================================================================
