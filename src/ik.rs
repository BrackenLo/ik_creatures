//====================================================================

use core::f32;
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
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

    pub fn attach(&mut self, parent: &Node) {
        let vector = parent.pos - self.pos;
        self.set_rotation(vector.to_angle());

        let rotation_diff = (self.rotation - parent.rotation + TAU + PI) % TAU - PI;
        let rotation_diff = rotation_diff.clamp(self.min_rotation, self.max_rotation);
        self.rotation = parent.rotation + rotation_diff;

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
            radius: 50.,
            pos: Vec2::ZERO,
            rotation: 0.,
            max_rotation: Self::DEFAULT_ANGLE,
            min_rotation: -Self::DEFAULT_ANGLE,
        }
    }
}

//====================================================================

pub type NodeID = u32;

#[derive(Default)]
pub struct Skeleton {
    current_id: NodeID,
    nodes: HashMap<NodeID, RefCell<Node>>,

    forward_kinematics: Vec<Vec<NodeID>>,
    inverse_kinematics: Vec<InverseKinematic>,
}

impl Skeleton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: Node) -> NodeID {
        let id = self.current_id;
        self.nodes.insert(id, RefCell::new(node));
        self.current_id += 1;

        id
    }

    pub fn add_fk(&mut self, fk: Vec<NodeID>) {
        self.forward_kinematics.push(fk);
    }

    pub fn add_ik(&mut self, ik: Vec<NodeID>) {
        let ik = InverseKinematic {
            nodes: ik,
            target: Vec2::ZERO,
        };
        self.inverse_kinematics.push(ik);
    }

    pub fn get_node_mut(&mut self, id: NodeID) -> Option<RefMut<Node>> {
        let node = self.nodes.get(&id)?;
        Some(node.borrow_mut())
    }

    pub fn get_ik(&mut self, index: usize) -> Option<&mut InverseKinematic> {
        self.inverse_kinematics.get_mut(index)
    }

    pub fn tick(&mut self) {
        self.forward_kinematics.iter().for_each(|fk| {
            if fk.len() < 2 {
                return;
            }

            (1..fk.len()).for_each(|index| {
                let first = &fk[index - 1];
                let second = &fk[index];

                let first = self.nodes.get(first).unwrap().borrow();
                let mut second = self.nodes.get(second).unwrap().borrow_mut();

                second.attach(&first);
            })
        });

        self.inverse_kinematics
            .iter()
            .for_each(|ik| fabrik(&self.nodes, ik));
    }
}

pub struct InverseKinematic {
    pub nodes: Vec<NodeID>,
    pub target: Vec2,
}

pub fn fabrik(nodes: &HashMap<NodeID, RefCell<Node>>, ik: &InverseKinematic) {
    if ik.nodes.len() < 3 {
        return;
    }

    let (anchor, anchor_rotation) = {
        let root = nodes.get(&ik.nodes[0]).unwrap().borrow();
        (root.pos, root.rotation)
    };

    for _ in 0..20 {
        nodes
            .get(ik.nodes.last().unwrap())
            .unwrap()
            .borrow_mut()
            .pos = ik.target;

        (0..ik.nodes.len() - 1).rev().for_each(|index| {
            let first = &ik.nodes[index + 1];
            let second = &ik.nodes[index];

            let first = nodes.get(first).unwrap().borrow();
            let mut second = nodes.get(second).unwrap().borrow_mut();

            second.attach(&first);
        });

        {
            let mut node = nodes.get(&ik.nodes[0]).unwrap().borrow_mut();
            node.pos = anchor;
            // node.rotation = anchor_rotation;
        }

        (1..ik.nodes.len()).for_each(|index| {
            let first = &ik.nodes[index - 1];
            let second = &ik.nodes[index];

            let first = nodes.get(first).unwrap().borrow();
            let mut second = nodes.get(second).unwrap().borrow_mut();

            second.attach(&first);
        });

        if nodes.get(ik.nodes.last().unwrap()).unwrap().borrow().pos == ik.target {
            break;
        }
    }
}

//====================================================================

impl Skeleton {
    pub fn triangle_list(&self) -> Vec<Vec<[f32; 2]>> {
        self.forward_kinematics
            .iter()
            .map(|skeleton| {
                skeleton
                    .iter()
                    .flat_map(|node| {
                        let node = self.nodes.get(node).unwrap().borrow();

                        let node_right = node.get_point(node.get_rotation() - FRAC_PI_2).to_array();
                        let node_left = node.get_point(node.get_rotation() + FRAC_PI_2).to_array();
                        vec![node_right, node_left]
                    })
                    .collect()
            })
            .collect()
    }

    pub fn circles(&self) -> Vec<RawInstance> {
        let circles = self
            .nodes
            .values()
            .map(|node| {
                let node = node.borrow();

                RawInstance::new(node.pos.to_array(), node.radius).hollow()
            })
            .collect::<Vec<_>>();

        self.forward_kinematics.iter().fold(circles, |mut acc, fk| {
            fk.iter().for_each(|node| {
                let node = self.nodes.get(node).unwrap().borrow();
                acc.push(
                    RawInstance::new(node.get_point(node.get_rotation()).to_array(), 5.)
                        .with_color([1., 0., 0., 1.]),
                );
            });

            acc
        })
    }

    // pub fn text(&self) -> Vec<TextData> {
    //     self.skeletons.iter()
    // }
}

//====================================================================

pub fn spawn_arm(skeleton: &mut Skeleton) {
    let nodes = [
        Node::unlocked(80.),
        Node::unlocked(80.),
        Node::unlocked(80.),
        Node::unlocked(80.),
        Node::unlocked(80.),
        Node::unlocked(80.),
        Node::unlocked(80.),
        Node::unlocked(80.),
    ]
    .into_iter()
    .map(|node| skeleton.add_node(node))
    .collect();

    skeleton.add_ik(nodes);
}

pub fn spawn_creature(skeleton: &mut Skeleton) {
    let nodes = [
        Node::new(30.),
        Node::locked(45., 0.),
        Node::locked(50., 0.),
        Node::new(40.),
        Node::new(40.),
        Node::unlocked(50.), // 5
        Node::new(60.),
        Node::new(63.),
        Node::new(65.),
        Node::new(63.),
        Node::new(60.),
        Node::new(40.),
        Node::new(30.),
        Node::new(20.),
        Node::new(20.),
        Node::new(20.),
        Node::new(20.),
        Node::new(20.),
        Node::new(10.),
        Node::new(10.),
    ]
    .into_iter()
    .map(|node| skeleton.add_node(node))
    .collect();

    skeleton.add_fk(nodes);

    let node = skeleton.add_node(Node::locked(40., 90.));
    skeleton.add_fk(vec![5, node]);

    let mut nodes = [
        // Node::angles(20., 90., 90.),
        // Node::locked(60., 0.),
        Node::default(),
        Node::default(),
        Node::default(),
        Node::default(),
        Node::default(),
    ]
    .into_iter()
    .map(|node| skeleton.add_node(node))
    .collect::<Vec<_>>();

    nodes.insert(0, node);

    skeleton.add_ik(nodes);
}

//====================================================================
