//====================================================================

use core::f32;
use std::{
    cell::RefCell,
    f32::consts::{FRAC_PI_2, PI, TAU},
};

use glam::{vec2, Vec2};

use crate::renderer::circles::RawInstance;

//====================================================================

pub struct Node {
    pub radius: f32,

    pub pos: Vec2,
    rotation: f32,
    pub max_rotation: f32,
    pub min_rotation: f32,
}

impl Node {
    const DEFAULT_ANGLE: f32 = 0.6981;

    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            ..Default::default()
        }
    }

    pub fn locked(radius: f32, rotation: f32) -> Self {
        Self {
            radius,
            max_rotation: rotation.to_radians(),
            min_rotation: rotation.to_radians(),
            ..Default::default()
        }
    }

    pub fn unlocked(radius: f32) -> Self {
        Self {
            radius,
            max_rotation: TAU,
            min_rotation: -TAU,
            ..Default::default()
        }
    }

    pub fn angle(radius: f32, angle: f32) -> Self {
        Self {
            radius,
            max_rotation: angle.to_radians(),
            min_rotation: angle.to_radians(),
            ..Default::default()
        }
    }

    pub fn angles(radius: f32, min: f32, max: f32) -> Self {
        Self {
            radius,
            max_rotation: max.to_radians(),
            min_rotation: min.to_radians(),
            ..Default::default()
        }
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    #[inline]
    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    pub fn attach_rot(&mut self, parent: &Node) {
        let vector = parent.pos - self.pos;
        self.set_rotation(vector.to_angle());

        let rotation_diff = (self.rotation - parent.rotation + TAU + PI) % TAU - PI;
        let rotation_diff = rotation_diff.clamp(self.min_rotation, self.max_rotation);
        self.rotation = parent.rotation + rotation_diff;

        let scaled = Vec2::from_angle(self.rotation) * parent.radius;
        self.pos = parent.pos - scaled;
    }

    pub fn attach(&mut self, parent: &Node) {
        let vector = parent.pos - self.pos;
        self.set_rotation(vector.to_angle());

        let scaled = Vec2::from_angle(self.rotation) * parent.radius;
        self.pos = parent.pos - scaled;
    }

    pub fn get_point(&self, angle: f32) -> Vec2 {
        let x = self.radius * angle.cos() + self.pos.x;
        let y = self.radius * angle.sin() + self.pos.y;

        vec2(x, y)
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            radius: 80.,
            pos: Vec2::ZERO,
            rotation: 0.,
            max_rotation: Self::DEFAULT_ANGLE,
            min_rotation: -Self::DEFAULT_ANGLE,
        }
    }
}

//====================================================================

pub struct ForwardKinematic {
    pub nodes: Vec<Node>,
}

impl ForwardKinematic {
    pub fn tick(&mut self) {
        if self.nodes.len() < 2 {
            return;
        }

        (1..self.nodes.len()).for_each(|index| {
            let (first, second) = self.nodes.split_at_mut(index);

            let first = first.last().unwrap();
            let second = &mut second[0];

            second.attach_rot(first);
        })
    }

    pub fn attach(&mut self, root: &Node) {
        if self.nodes.is_empty() {
            return;
        }

        self.nodes[0].attach_rot(root);
        self.tick();
    }
}

pub struct InverseKinematic {
    pub nodes: Vec<RefCell<Node>>,
    pub anchor: Vec2,
    pub target: Vec2,
    pub cycles: usize,
}

impl InverseKinematic {
    pub fn new(target: Vec2, anchor: Vec2) -> Self {
        Self {
            nodes: Vec::new(),
            anchor,
            target,
            cycles: 1,
        }
    }

    pub fn with_nodes<T: IntoIterator<Item = Node>>(mut self, nodes: T) -> Self {
        self.add_nodes(nodes);
        self
    }

    pub fn add_nodes<T: IntoIterator<Item = Node>>(&mut self, nodes: T) {
        nodes
            .into_iter()
            .for_each(|node| self.nodes.push(RefCell::new(node)));
    }

    pub fn circles(&self) -> Vec<RawInstance> {
        self.nodes
            .iter()
            .flat_map(|node| {
                let node = node.borrow();
                [
                    RawInstance::new(node.pos.to_array(), node.radius).hollow(),
                    RawInstance::new(node.get_point(node.rotation).to_array(), 5.)
                        .with_color([1., 0., 0., 1.]),
                ]
            })
            .collect()
    }

    pub fn fabrik(&mut self) -> bool {
        if self.nodes.len() < 3 {
            return false;
        }

        let initial_rot = self.nodes[0].borrow().rotation;

        for _ in 0..self.cycles {
            self.nodes.last().unwrap().borrow_mut().pos = self.target;

            (0..self.nodes.len() - 1).rev().for_each(|index| {
                let first = self.nodes[index + 1].borrow();
                let mut second = self.nodes[index].borrow_mut();

                second.attach(&first);
            });

            // self.nodes[0].borrow_mut().pos = self.anchor;
            {
                let mut node = self.nodes[0].borrow_mut();
                node.pos = self.anchor;
                node.rotation = initial_rot;
            }

            (1..self.nodes.len()).for_each(|index| {
                let first = self.nodes[index - 1].borrow();
                let mut second = self.nodes[index].borrow_mut();

                second.attach_rot(&first);
            });

            if self.nodes.last().unwrap().borrow().pos == self.target {
                return true;
            }
        }

        return false;
    }
}
//====================================================================

pub fn triangle_list(nodes: &[Node]) -> Vec<[f32; 2]> {
    nodes
        .iter()
        .flat_map(|node| {
            let node_right = node.get_point(node.get_rotation() - FRAC_PI_2).to_array();
            let node_left = node.get_point(node.get_rotation() + FRAC_PI_2).to_array();
            vec![node_right, node_left]
        })
        .collect()
}

// impl Skeleton {
//     pub fn triangle_list(&self) -> Vec<Vec<[f32; 2]>> {
//         self.forward_kinematics
//             .iter()
//             .map(|skeleton| {
//                 skeleton
//                     .iter()
//                     .flat_map(|node| {
//                         let node = self.nodes.get(node).unwrap().borrow();

//                         let node_right = node.get_point(node.get_rotation() - FRAC_PI_2).to_array();
//                         let node_left = node.get_point(node.get_rotation() + FRAC_PI_2).to_array();
//                         vec![node_right, node_left]
//                     })
//                     .collect()
//             })
//             .collect()
//     }

//     pub fn circles(&self) -> Vec<RawInstance> {
//         let circles = self
//             .nodes
//             .values()
//             .map(|node| {
//                 let node = node.borrow();

//                 RawInstance::new(node.pos.to_array(), node.radius).hollow()
//             })
//             .collect::<Vec<_>>();

//         self.forward_kinematics.iter().fold(circles, |mut acc, fk| {
//             fk.iter().for_each(|node| {
//                 let node = self.nodes.get(node).unwrap().borrow();
//                 acc.push(
//                     RawInstance::new(node.get_point(node.get_rotation()).to_array(), 5.)
//                         .with_color([1., 0., 0., 1.]),
//                 );
//             });

//             acc
//         })
//     }

//     // pub fn text(&self) -> Vec<TextData> {
//     //     self.skeletons.iter()
//     // }
// }

//====================================================================

// pub fn spawn_creature(skeleton: &mut Skeleton) {
//     let nodes = [
//         Node::new(30.),
//         Node::locked(45., 0.),
//         Node::locked(50., 0.),
//         Node::new(40.),
//         Node::new(40.),
//         Node::unlocked(50.), // 5
//         Node::new(60.),
//         Node::new(63.),
//         Node::new(65.),
//         Node::new(63.),
//         Node::new(60.),
//         Node::new(40.),
//         Node::new(30.),
//         Node::new(20.),
//         Node::new(20.),
//         Node::new(20.),
//         Node::new(20.),
//         Node::new(20.),
//         Node::new(10.),
//         Node::new(10.),
//     ]
//     .into_iter()
//     .map(|node| skeleton.add_node(node))
//     .collect();

//     skeleton.add_fk(nodes);

//     let node = skeleton.add_node(Node::locked(40., 90.));
//     skeleton.add_fk(vec![5, node]);

//     let mut nodes = [
//         // Node::angles(20., 90., 90.),
//         // Node::locked(60., 0.),
//         Node::default(),
//         Node::default(),
//         Node::default(),
//         Node::default(),
//         Node::default(),
//     ]
//     .into_iter()
//     .map(|node| skeleton.add_node(node))
//     .collect::<Vec<_>>();

//     nodes.insert(0, node);

//     // skeleton.add_ik(nodes);
// }

//====================================================================
