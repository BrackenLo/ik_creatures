use wgpu::util::DeviceExt;

use super::{
    tools::{self, Pipeline, PipelineUpdate, Vertex},
    uniques::Uniques,
    Core,
};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct RawVertex {
    pos: [f32; 2],
}

impl Vertex for RawVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
            0 => Float32x2
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RawVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &VERTEX_ATTRIBUTES,
        }
    }
}

const VERTICES: [RawVertex; 4] = [
    RawVertex { pos: [-0.5, 0.5] },
    RawVertex { pos: [-0.5, -0.5] },
    RawVertex { pos: [0.5, 0.5] },
    RawVertex { pos: [0.5, -0.5] },
];

pub const INDICES: [u16; 6] = [0, 1, 3, 0, 3, 2];

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct RawInstance {
    pub pos: [f32; 2],
    pub radius: f32,
    pub border_radius: f32,
    pub color: [f32; 4],
    pub border_color: [f32; 4],
    // hollow: bool, // TODO
}

impl Vertex for RawInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
            1 => Float32x2, 2 => Float32, 3 => Float32, 4 => Float32x4, 5 => Float32x4,
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RawInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &VERTEX_ATTRIBUTES,
        }
    }
}

pub struct CirclePipeline {
    pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,

    instance_buffer: wgpu::Buffer,
    instance_count: u32,
}

impl Pipeline for CirclePipeline {
    fn new(core: &Core, uniques: &mut Uniques) -> Self {
        let unique = uniques.insert_next(&core.device);

        let pipeline = tools::create_pipeline(
            &core.device,
            &core.config,
            "Circle Pipeline",
            &[&unique.camera_bind_group_layout],
            &[RawVertex::desc(), RawInstance::desc()],
            include_str!("circle_shader.wgsl").into(),
            // tools::RenderPipelineDescriptor {
            //     fragment_targets: Some(&[Some(wgpu::ColorTargetState {
            //         format: core.config.format,
            //         blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            //         write_mask: wgpu::ColorWrites::all(),
            //     })]),
            //     ..Default::default()
            // },
            tools::RenderPipelineDescriptor::default(),
        );

        let vertex_buffer = core
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Circle Pipeline Vertex Buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = core
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Circle Pipeline Index Buffer"),
                contents: bytemuck::cast_slice(&INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });
        let index_count = INDICES.len() as u32;

        let instance_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Circle Pipeline Instance Buffer"),
            size: 0,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let instance_count = 0 as u32;

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count,
            instance_buffer,
            instance_count,
        }
    }

    fn render(&mut self, pass: &mut wgpu::RenderPass, uniques: &Uniques) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &uniques.get(0).unwrap().camera_bind_group, &[]);

        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        pass.draw_indexed(0..self.index_count, 0, 0..self.instance_count);
    }
}

impl PipelineUpdate<&[RawInstance]> for CirclePipeline {
    fn update(&mut self, core: &Core, data: &[RawInstance]) {
        tools::update_instance_buffer(
            &core.device,
            &core.queue,
            "Circle Pipeline Instance Buffer",
            &mut self.instance_buffer,
            &mut self.instance_count,
            data,
        );
    }
}
