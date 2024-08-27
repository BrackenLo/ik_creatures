//====================================================================

use core::f32;
use std::f32::consts::{PI, TAU};

use glam::{vec2, Vec2};

//====================================================================

const ANGLE: f32 = TAU + PI;

pub struct Node {
    pub radius: f32,

    pub pos: Vec2,
    rotation: f32,
    pub max_rotation: f32,
}

impl Node {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            pos: Vec2::ZERO,
            rotation: 0.,
            max_rotation: 20_f32.to_radians(),
        }
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
        // self.rotation = rotation % TAU;
        // if self.rotation < 0. {
        //     self.rotation += TAU;
        // }
    }

    #[inline]
    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    pub fn attach(&mut self, parent: &Node) {
        let vector = parent.pos - self.pos;

        self.set_rotation(vector.to_angle());

        // 180 - abs(abs(a1 - a2) - 180);

        // delta = (targetAngle - myAngle + 540) % 360 - 180;

        let rotation_diff = (self.rotation - parent.rotation + ANGLE) % TAU - PI;

        // let rotation_diff = PI - ((self.rotation - parent.rotation).abs() - PI).abs();

        let rotation_diff = rotation_diff.clamp(-self.max_rotation, self.max_rotation);

        // let rotation_diff =
        //     (self.rotation - parent.rotation).clamp(-self.max_rotation, self.max_rotation);

        self.rotation = parent.rotation + rotation_diff;

        let scaled = Vec2::from_angle(self.rotation) * parent.radius;

        // let scaled = vector.normalize_or(Vec2::ONE) * parent.radius;
        // self.pos = scaled - parent.pos;
        self.pos = parent.pos - scaled;

        // self.pos = parent.get_point(-self.rotation);
    }

    pub fn get_point(&self, angle: f32) -> Vec2 {
        let x = self.radius * angle.cos() + self.pos.x;
        let y = self.radius * angle.sin() + self.pos.y;

        vec2(x, y)
    }
}

//====================================================================

pub fn get_rotation(point: &Vec2, rotation: f32, translation: &Vec2) -> Vec2 {
    let cos = rotation.cos();
    let sin = rotation.sin();

    vec2(
        (point.x * cos - point.y * sin) + translation.x,
        (point.y * cos + point.x * sin) + translation.y,
    )
}
