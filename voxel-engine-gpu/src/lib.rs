#![no_std]

mod camera_matrices;
mod intersect;
mod octree;
mod ray;
mod sky;

pub use camera_matrices::*;
pub use glam;
pub use intersect::*;
pub use octree::*;
pub use ray::*;
pub use sky::*;

use glam::{
    vec3, IVec2, Mat4, UVec2, UVec3, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles,
};
use spirv_std::num_traits::Float;
use spirv_std::{spirv, Image};

const STACK_SIZE: usize = 8;

struct Stack {
    index: [usize; STACK_SIZE],
    center: [Vec3; STACK_SIZE],
    scale: [f32; STACK_SIZE],
    ptr: i32,
}

impl Stack {
    fn new() -> Self {
        Self {
            index: [0; STACK_SIZE],
            center: [Vec3::default(); STACK_SIZE],
            scale: [0.0; STACK_SIZE],
            ptr: -1,
        }
    }

    fn empty(&self) -> bool {
        self.ptr < 0
    }

    fn push(&mut self, index: usize, center: Vec3, scale: f32) {
        self.ptr += 1;
        self.index[self.ptr as usize] = index;
        self.center[self.ptr as usize] = center;
        self.scale[self.ptr as usize] = scale;
    }

    fn pop(&mut self) -> (usize, Vec3, f32) {
        self.ptr -= 1;
        (
            self.index[(self.ptr + 1) as usize],
            self.center[(self.ptr + 1) as usize],
            self.scale[(self.ptr + 1) as usize],
        )
    }
}

#[test]
fn test_stack() {
    let mut stack = Stack::new();

    assert!(stack.empty());
    stack.push(0, Vec3::default(), 0.0);
    assert!(!stack.empty());
    assert_eq!(stack.pop(), (0, Vec3::default(), 0.0));
    assert!(stack.empty());

    stack.push(0, Vec3::default(), 0.0);
    stack.push(1, Vec3::default(), 0.1);

    assert_eq!(stack.pop(), (1, Vec3::default(), 0.1));
    assert_eq!(stack.pop(), (0, Vec3::default(), 0.0));
}

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

pub fn trace_octree(ray: &Ray, octree: &[OctreeNode]) -> Vec3 {
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

#[spirv(compute(threads(16, 16)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image!(2D, format = rgba32f, sampled = false),
    #[spirv(descriptor_set = 0, binding = 1, uniform)] camera: &CameraMatrices,
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)] octree: &[OctreeNode],
) {
    let output_coords = id.xy();
    let screen_size: UVec2 = image.query_size();

    if output_coords.x >= screen_size.x || output_coords.y >= screen_size.y {
        return;
    }

    let screen_coords = output_coords.as_vec2() / screen_size.as_vec2() * 2.0 - 1.0;
    let camera_ray = camera.create_ray(screen_coords);
    let output_color = trace_octree(&camera_ray, octree);

    unsafe {
        image.write(output_coords, Vec4::from((output_color, 1.0)));
    }
}
