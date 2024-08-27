use std::sync::Arc;

use anyhow::Context;
use tools::{Pipeline, PipelineUpdate};
use uniques::{Camera, Uniques};

pub mod circles;
pub mod polygon;
pub mod text;
pub mod tools;
pub mod uniques;

pub struct Renderer {
    core: Core,

    uniques: Uniques,
}

pub struct Core {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}
impl Core {
    #[inline]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    #[inline]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    #[inline]
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}

impl Renderer {
    pub async fn new(window: Arc<winit::window::Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .context("Could not get wgpu adapter.")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await?;

        let capabilities = surface.get_capabilities(&adapter);

        let surface_format = capabilities
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let uniques = Uniques::default();

        Ok(Self {
            core: Core {
                device,
                queue,
                surface,
                config,
            },
            uniques,
        })
    }

    #[inline]
    pub fn create_pipeline<T: Pipeline>(&mut self) -> T {
        T::new(&self.core, &mut self.uniques)
    }

    #[inline]
    pub fn resize_pipeline<T: Pipeline>(&self, pipeline: &mut T, width: u32, height: u32) {
        pipeline.resize(&self.core, width, height);
    }

    #[inline]
    pub fn update_pipeline<D, T: PipelineUpdate<D>>(&self, pipeline: &mut T, data: D) {
        pipeline.update(&self.core, data);
    }

    #[inline]
    pub fn uniques(&self) -> &Uniques {
        &self.uniques
    }

    #[inline]
    pub fn update_camera(&mut self, slot: usize, data: &dyn Camera) {
        self.uniques.update_camera(&self.core.queue, slot, data);
    }

    pub fn render(&self, pipelines: &mut [&mut dyn Pipeline]) -> anyhow::Result<()> {
        let surface_texture = self.core.surface.get_current_texture()?;
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .core
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pipelines.into_iter().for_each(|pipeline| {
                pipeline.render(&mut pass, &self.uniques);
            });
        }

        self.core.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}
