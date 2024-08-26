use std::sync::Arc;

use glam::{vec2, Vec2};
use ik_creatures::{
    ik::Node,
    renderer::{
        circles::{CirclePipeline, RawInstance},
        uniques::OrthographicCamera,
        Renderer,
    },
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
    window: Arc<winit::window::Window>,
    renderer: Renderer,
    circles: CirclePipeline,

    camera: OrthographicCamera,
    mouse_pos: Vec2,
    mouse_vector: Vec2,

    nodes: Vec<Node>,
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

        let camera = OrthographicCamera::default();
        renderer.update_camera(0, &camera);

        // let nodes = [
        //     45, 50, 40, 40, 50, 60, 63, 65, 63, 60, 40, 30, 20, 20, 20, 20, 20, 10,
        // ];

        let nodes = [50; 2];

        let nodes = nodes.into_iter().map(|val| Node::new(val as f32)).collect();

        Self {
            window,
            renderer,
            circles,
            camera,
            mouse_pos: Vec2::ZERO,
            mouse_vector: Vec2::ZERO,
            nodes,
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

                self.mouse_vector = relative_pos - self.mouse_pos;
                self.mouse_pos = relative_pos;
            }

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
    }

    fn tick(&mut self) {
        if !self.nodes.is_empty() {
            self.nodes[0].pos = self.mouse_pos;
            self.nodes[0].rotation = self.mouse_vector.to_angle();

            // println!("Root rotation {}", self.nodes[0].rotation.to_degrees());
        }

        if self.nodes.len() > 1 {
            (1..self.nodes.len()).for_each(|index| {
                let (first, second) = self.nodes.split_at_mut(index);

                let first = first.last().unwrap();
                let second = &mut second[0];

                second.attach(first);

                // println!("Nexr rotation {}", second.rotation.to_degrees());
            });
        }

        let instances = self.nodes.iter().fold(Vec::new(), |mut acc, node| {
            acc.push(RawInstance::new(node.pos.to_array(), node.radius).hollow());

            acc.push(
                RawInstance::new(node.get_point(node.rotation).to_array(), 5.)
                    .with_color([1., 0., 0., 1.]),
            );

            acc
        });

        // let instances = self
        //     .nodes
        //     .iter()
        //     .map(|node| RawInstance {
        //         pos: node.pos.to_array(),
        //         radius: node.radius,
        //         border_radius: 6.,
        //         color: [1., 1., 1., 0.],
        //         border_color: [0., 0., 0., 1.],
        //     })
        //     .collect::<Vec<_>>();

        self.renderer.update_pipeline(
            &mut self.circles,
            &[
                instances.as_slice(),
                // &[RawInstance {
                //     pos: self.mouse_pos.to_array(),
                //     radius: 20.,
                //     border_radius: 3.,
                //     color: [0.95, 0., 0.8, 1.],
                //     border_color: [1., 0., 1., 1.],
                // }],
            ]
            .concat(),
        );

        self.renderer.render(&mut [&mut self.circles]).unwrap();

        self.window.request_redraw();
    }
}

//====================================================================
