use glam::{Vec3, vec3};
use spirv_std::num_traits::Float;
use crate::{intersect_box, OctreeNode, Ray, sky_color, Stack};

const PPP: Vec3 = Vec3::new(1.0, 1.0, 1.0);
const PNP: Vec3 = Vec3::new(1.0, -1.0, 1.0);
const PNN: Vec3 = Vec3::new(1.0, -1.0, -1.0);
const NPN: Vec3 = Vec3::new(-1.0, 1.0, -1.0);
const NNN: Vec3 = Vec3::new(-1.0, -1.0, -1.0);
const NNP: Vec3 = Vec3::new(-1.0, -1.0, 1.0);
const NPP: Vec3 = Vec3::new(-1.0, 1.0, 1.0);
const PPN: Vec3 = Vec3::new(1.0, 1.0, -1.0);
const POS: [Vec3; 8] = [PNN, PNP, PPN, PPP, NNN, NNP, NPN, NPP];
const EPSILON: f32 = 0.000001;

pub fn trace_octree_top_down(ray: &Ray, octree: &[OctreeNode]) -> Vec3 {
    let center = vec3(0.0, 0.0, 0.0);
    let scale = 1.0;

    if !intersect_box(ray, center, scale).0 {
        return sky_color(ray);
    }

    let mut stack = Stack::new();
    stack.push(0, center, scale);

    let mut closest_hit = Float::infinity();
    let mut color = sky_color(ray);

    while !stack.empty() {
        let (index, center, scale) = stack.pop();
        let node = octree[index];

        for child_index in 0..8 {
            if !node.valid(child_index) {
                continue;
            }

            let new_center = center + POS[child_index] * scale * 0.5;

            if node.leaf(child_index) {
                let (hit, distance) = intersect_box(ray, new_center, scale * 0.5);

                if hit && distance < closest_hit {
                    closest_hit = distance;
                    color = (new_center + 1.0) / 2.0;
                }
            } else {
                stack.push(node.child_ptr() as usize, new_center, scale * 0.5);
            }
        }
    }

    color
}
