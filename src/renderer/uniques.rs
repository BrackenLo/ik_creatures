//====================================================================

//====================================================================

use wgpu::util::DeviceExt;

#[derive(Default)]
pub struct Uniques {
    profiles: Vec<UniqueProfile>,
}

pub struct UniqueProfile {
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
}

impl Uniques {
    pub fn get(&self, slot: usize) -> Option<&UniqueProfile> {
        self.profiles.get(slot)
    }

    pub fn first(&mut self, device: &wgpu::Device) -> &UniqueProfile {
        if self.profiles.is_empty() {
            self.insert(device);
        }

        self.profiles.first().unwrap()
    }

    fn insert(&mut self, device: &wgpu::Device) {
        let default_camera = OrthographicCamera::default();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Unique Profile camera buffer"),
            contents: bytemuck::cast_slice(&[default_camera.into_uniform()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Unique Profile Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Unique Profile Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(camera_buffer.as_entire_buffer_binding()),
            }],
        });

        let profile = UniqueProfile {
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        };

        self.profiles.push(profile);
    }

    pub fn insert_next_mut(&mut self, device: &wgpu::Device) -> &mut UniqueProfile {
        self.insert(device);
        self.profiles.last_mut().unwrap()
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue, slot: usize, data: &dyn Camera) {
        let profile = self.profiles.get(slot).unwrap();

        queue.write_buffer(
            &profile.camera_buffer,
            0,
            bytemuck::cast_slice(&[data.into_uniform()]),
        );
    }
}

//====================================================================

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct CameraUniform {
    view_projection: [f32; 16],
    camera_position: [f32; 3],
    _padding: u32,
}
impl CameraUniform {
    pub fn new(view_projection: [f32; 16], camera_position: [f32; 3]) -> Self {
        Self {
            view_projection,
            camera_position,
            _padding: 0,
        }
    }
}

pub trait Camera {
    fn into_uniform(&self) -> CameraUniform;
}

//--------------------------------------------------

pub struct PerspectiveCamera {
    pub up: glam::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub z_near: f32,
    pub z_far: f32,

    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
}
impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            up: glam::Vec3::Y,
            aspect: 1.7777777778,
            fovy: 45.,
            z_near: 0.1,
            z_far: 1000000.,

            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn into_uniform(&self) -> CameraUniform {
        CameraUniform::new(
            self.get_projection().to_cols_array(),
            self.translation.into(),
        )
    }
}

impl PerspectiveCamera {
    fn get_projection(&self) -> glam::Mat4 {
        let forward = (self.rotation * glam::Vec3::Z).normalize();

        let projection_matrix =
            glam::Mat4::perspective_lh(self.fovy, self.aspect, self.z_near, self.z_far);

        let view_matrix =
            glam::Mat4::look_at_lh(self.translation, self.translation + forward, self.up);

        projection_matrix * view_matrix
    }
}

//--------------------------------------------------

#[derive(Debug)]
pub struct OrthographicCamera {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub z_near: f32,
    pub z_far: f32,

    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Default for OrthographicCamera {
    fn default() -> Self {
        const DEFAULT_HALF_WIDTH: f32 = 1920. / 2.;
        const DEFAULT_HALF_HEIGHT: f32 = 1080. / 2.;

        Self {
            left: -DEFAULT_HALF_WIDTH,
            right: DEFAULT_HALF_WIDTH,
            bottom: -DEFAULT_HALF_HEIGHT,
            top: DEFAULT_HALF_HEIGHT,
            z_near: 0.,
            z_far: 1000000.,

            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
        }
    }
}

impl Camera for OrthographicCamera {
    fn into_uniform(&self) -> CameraUniform {
        CameraUniform::new(
            self.get_projection().to_cols_array(),
            self.translation.into(),
        )
    }
}

impl OrthographicCamera {
    fn get_projection(&self) -> glam::Mat4 {
        let projection_matrix = glam::Mat4::orthographic_lh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.z_near,
            self.z_far,
        );

        let transform_matrix =
            glam::Mat4::from_rotation_translation(self.rotation, self.translation);

        projection_matrix * transform_matrix
    }

    pub fn new_sized(width: f32, height: f32) -> Self {
        Self {
            left: 0.,
            right: width,
            bottom: 0.,
            top: height,
            ..Default::default()
        }
    }

    pub fn new_centered(half_width: f32, half_height: f32, x: f32, y: f32) -> Self {
        Self {
            left: -half_width,
            right: half_width,
            bottom: -half_height,
            top: half_height,
            translation: glam::Vec3::new(x, y, 0.),
            ..Default::default()
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        let half_width = width / 2.;
        let half_height = height / 2.;

        self.left = -half_width;
        self.right = half_width;
        self.bottom = -half_height;
        self.top = half_height;
    }
}

//====================================================================
