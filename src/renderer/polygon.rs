//====================================================================

use wgpu::util::DeviceExt;

use crate::renderer::tools;

use super::tools::{Pipeline, PipelineUpdate, Vertex};

//====================================================================

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

pub struct PolygonPipeline {
    pipeline: wgpu::RenderPipeline,

    vertex_instances: Vec<VertexInstance>,
}

struct VertexInstance {
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl Pipeline for PolygonPipeline {
    fn new(core: &super::Core, uniques: &mut super::uniques::Uniques) -> Self
    where
        Self: Sized,
    {
        let unique = uniques.first(core.device());

        let pipeline = tools::create_pipeline(
            core.device(),
            core.config(),
            "Polygon Pipeline",
            &[&unique.camera_bind_group_layout],
            &[RawVertex::desc()],
            include_str!("polygon_shader.wgsl").into(),
            tools::RenderPipelineDescriptor::default(),
        );

        Self {
            pipeline,
            vertex_instances: Vec::new(),
        }
    }

    fn render<'pass>(
        &'pass mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        uniques: &super::uniques::Uniques,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &uniques.get(0).unwrap().camera_bind_group, &[]);

        self.vertex_instances.iter().for_each(|instance| {
            pass.set_vertex_buffer(0, instance.vertex_buffer.slice(..));
            pass.set_index_buffer(instance.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..instance.index_count, 0, 0..1);
        });
    }
}

impl PipelineUpdate<&[(&[RawVertex], &[u16])]> for PolygonPipeline {
    fn update(&mut self, core: &super::Core, data: &[(&[RawVertex], &[u16])]) {
        //
        if data.len() > self.vertex_instances.len() {
            (0..data.len() - self.vertex_instances.len()).for_each(|_| {
                //
                self.vertex_instances.push(VertexInstance {
                    vertex_buffer: core.device().create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Polygon Vertices"),
                        size: 0,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                    vertex_count: 0,

                    index_buffer: core.device().create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Polygon Indices"),
                        size: 0,
                        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                    index_count: 0,
                })
            });
        }

        if data.len() < self.vertex_instances.len() {
            (0..self.vertex_instances.len() - data.len()).for_each(|_| {
                self.vertex_instances.pop();
            })
        }

        data.into_iter()
            .zip(self.vertex_instances.iter_mut())
            .for_each(|((vertices, indices), instance)| {
                if vertices.len() as u32 > instance.vertex_count {
                    instance.vertex_buffer =
                        core.device()
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Polygon Vertices"),
                                contents: bytemuck::cast_slice(vertices),
                                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                            });
                }
                //
                else {
                    core.queue().write_buffer(
                        &instance.vertex_buffer,
                        0,
                        bytemuck::cast_slice(vertices),
                    );
                }

                if indices.len() as u32 > instance.index_count {
                    instance.index_buffer =
                        core.device()
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Polygon Indices"),
                                contents: bytemuck::cast_slice(indices),
                                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                            });
                }
                //
                else {
                    core.queue().write_buffer(
                        &instance.index_buffer,
                        0,
                        bytemuck::cast_slice(indices),
                    );
                }

                instance.index_count = indices.len() as u32;
            });
    }
}

//====================================================================

pub fn calculate_strip(vertices: &[[f32; 2]]) -> (Vec<RawVertex>, Vec<u16>) {
    if vertices.len() < 4 {
        return (Vec::new(), Vec::new());
    }

    let vertices = vertices.into_iter().fold(Vec::new(), |mut acc, vertex| {
        acc.push(RawVertex { pos: *vertex });
        acc
    });

    let indices = (3..vertices.len())
        .step_by(2)
        .fold(Vec::new(), |mut acc, index| {
            acc.push(index as u16 - 3); // 0
            acc.push(index as u16 - 2); // 1
            acc.push(index as u16 - 1); // 2

            acc.push(index as u16 - 1); // 2
            acc.push(index as u16 - 2); // 1
            acc.push(index as u16); // 3

            acc
        });

    (vertices, indices)
}

//====================================================================
