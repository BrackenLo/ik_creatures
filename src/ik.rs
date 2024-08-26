//====================================================================

use glam::{vec2, Vec2};

//====================================================================

pub struct Node {
    pub radius: f32,

    pub pos: Vec2,
    pub rotation: f32,
    pub max_rotation: f32,
}

impl Node {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            pos: Vec2::ZERO,
            rotation: 0.,
            max_rotation: 45_f32.to_radians(),
        }
    }

    pub fn attach(&mut self, parent: &Node) {
        // let rotation_diff = self.rotation - parent.rotation;

        // if rotation_diff > self.max_rotation {
        //     // println!(
        //     //     "Diff = {}, old rot = {}, new rot = {}",
        //     //     rotation_diff.to_degrees(),
        //     //     self.rotation.to_degrees(),
        //     //     parent.rotation.to_degrees()
        //     // );

        //     self.rotation = parent.rotation + self.max_rotation;
        //     self.pos = self.get_point(self.rotation);
        //     return;
        // }
        // //
        // else if rotation_diff < -self.max_rotation {
        //     self.rotation = parent.rotation - self.max_rotation;
        //     self.pos = self.get_point(self.rotation);
        //     return;
        // }

        // println!("A");

        let vector = self.pos - parent.pos;
        self.rotation = (-vector).to_angle();

        let scaled = Vec2::from_angle(self.rotation) * parent.radius;

        // let scaled = vector.normalize_or(Vec2::ONE) * parent.radius;
        self.pos = scaled + parent.pos;

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
