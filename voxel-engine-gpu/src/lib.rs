#![no_std]
#![feature(const_fn_floating_point_arithmetic)]

mod camera_matrices;
mod intersect;
mod octree;
mod ray;
mod sky;
mod stack;

pub use camera_matrices::*;
pub use glam;
pub use intersect::*;
pub use octree::*;
pub use ray::*;
pub use sky::*;
pub use stack::*;

use glam::{
    vec3, IVec2, Mat4, UVec2, UVec3, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles,
};
use spirv_std::num_traits::Float;
use spirv_std::{spirv, Image};

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
