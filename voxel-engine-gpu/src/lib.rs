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

use glam::{IVec2, Mat4, UVec2, UVec3, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
use spirv_std::num_traits::Float;
use spirv_std::{spirv, Image};

fn trace_ray(ray: &Ray) -> Vec3 {
    let mut hit = false;
    let mut distance = Float::infinity();
    let mut color = Vec3::default();

    let hit_sphere = intersect_sphere(ray, Vec3::new(1.0, 0.0, 0.0), 0.5);
    if hit_sphere.0 && hit_sphere.1 < distance {
        hit = true;
        distance = hit_sphere.1;
        color = Vec3::new(0.0, 1.0, 0.0);
    }

    let hit_sphere = intersect_sphere(ray, Vec3::new(-1.0, 0.0, 0.0), 0.5);
    if hit_sphere.0 && hit_sphere.1 < distance {
        hit = true;
        distance = hit_sphere.1;
        color = Vec3::new(0.0, 1.0, 0.0);
    }

    let hit_sphere = intersect_sphere(ray, Vec3::new(0.0, 0.0, 1.0), 0.5);
    if hit_sphere.0 && hit_sphere.1 < distance {
        hit = true;
        distance = hit_sphere.1;
        color = Vec3::new(0.0, 1.0, 0.0);
    }

    let hit_sphere = intersect_sphere(ray, Vec3::new(0.0, 0.0, -1.0), 0.5);
    if hit_sphere.0 && hit_sphere.1 < distance {
        hit = true;
        distance = hit_sphere.1;
        color = Vec3::new(0.0, 1.0, 0.0);
    }

    let hit_box = intersect_box(ray, Vec3::new(0.0, 0.0, 0.0), 0.5);
    if hit_box.0 && hit_box.1 < distance {
        hit = true;
        distance = hit_box.1;
        color = Vec3::new(1.0, 0.0, 0.0);
    }

    if hit {
        color * (distance / 20.0)
    } else {
        sky_color(ray)
    }
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

fn simple_octree(ray: &Ray, octree: &[OctreeNode]) -> Vec3 {
    #[derive(Default, Copy, Clone)]
    struct Stack {
        index: usize,
        center: Vec3,
        scale: f32,
    }

    let mut stack = [Stack::default(); 10];
    let mut stack_pos = 1;
    let mut center = Vec3::splat(0.0);
    let mut scale = 1.0;
    let mut index = 0usize;

    if !intersect_box(ray, center, scale).0 {
        return sky_color(ray);
    } else {
        return stack[9].center;
    }

    stack[0] = Stack {
        index,
        center,
        scale: scale * 0.5,
    };

    while stack_pos > 0 {
        stack_pos -= 1;

        center = stack[stack_pos].center;
        index = stack[stack_pos].index;
        scale = stack[stack_pos].scale;

        //uint node = 0x000003FF;
        let node = &octree[index];
        let child_ptr = node.child_ptr();
        let valid_mask = node.valid();
        let leaf_mask = node.leaf();

        for i in 0..8 {
            let valid = (valid_mask & (1 << i)) != 0;
            let leaf = (leaf_mask & (1 << i)) != 0;

            if !valid {
                continue;
            }

            let new_center = center + scale * POS[i];

            if !intersect_box(ray, new_center, scale).0 {
                continue;
            }

            if leaf {
                return Vec3::new(1.0, 0.0, 0.0); //vec3(hit.distance) / 10;
            } else {
                break;
                //stack[stack_pos] = Stack { index: child_ptr as usize, center: new_center, scale: scale * 0.5 };
                //stack_pos += 1;
            }
        }
    }

    sky_color(ray)
}

#[spirv(compute(threads(10, 10)))]
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
    let output_color = simple_octree(&camera.create_ray(screen_coords), octree);

    unsafe {
        image.write(output_coords, Vec4::from((output_color, 1.0)));
    }
}
