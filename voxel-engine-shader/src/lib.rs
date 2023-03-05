#![no_std]

// Export libraries
pub use bytemuck;
pub use glam;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, UVec2, UVec3, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
use spirv_std::{image::StorageImage2d, spirv};

#[repr(C)]
#[derive(Default, Copy, Clone, Pod, Zeroable)]
pub struct RayCamera {
    pub inverse_view: Mat4,
    pub inverse_centered_view: Mat4,
    pub inverse_projection: Mat4,
}

impl RayCamera {
    fn create_ray(&self, screen_coords: &Vec2) -> Ray {
        Ray {
            origin: (self.inverse_view * Vec4::new(0.0, 0.0, 0.0, 1.0)).xyz(),
            direction: (self.inverse_centered_view
                * self.inverse_projection
                * Vec4::from((*screen_coords, 0.0, 1.0)))
            .xyz(),
        }
    }
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

fn sky_color(ray: &Ray) -> Vec3 {
    let t = 0.5 * (ray.direction.y + 1.0);
    return (1.0 - t) * Vec3::splat(1.0) + t * Vec3::new(0.5, 0.7, 1.0);
}

fn intersect_box(ray: &Ray, center: &Vec3, extent: f32) -> bool {
    let box_min = *center - extent;
    let box_max = *center + extent;

    let inv_ray_dir = 1.0 / ray.direction;
    let t_min = (box_min - ray.origin) * inv_ray_dir;
    let t_max = (box_max - ray.origin) * inv_ray_dir;

    let t1 = t_min.min(t_max);
    let t2 = t_min.max(t_max);

    let t_near = t1.x.max(t1.y).max(t1.z);
    let t_far = t2.x.min(t2.y).min(t2.z);

    t_near <= t_far
}

fn trace_ray(ray: &Ray) -> Vec3 {
    if intersect_box(ray, &Vec3::splat(0.0), 0.5) {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        sky_color(ray)
    }
}

#[spirv(compute(threads(10, 10)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(binding = 0)] output: &mut StorageImage2d,
    #[spirv(uniform, binding = 1)] camera: &RayCamera,
    #[spirv(storage_buffer, binding = 2)] octree: &[u32],
) {
    let output_size: UVec2 = output.query_size();
    let output_coords = id.xy();

    // Check that coordinates do not exceed image boundaries
    if output_coords.x >= output_size.x && output_coords.y >= output_size.x {
        return;
    }

    let screen_coords = output_coords.as_vec2() / output_size.as_vec2() * 2.0 - 1.0;
    let output_color = trace_ray(&camera.create_ray(&screen_coords));

    unsafe {
        output.write(output_coords, Vec4::from((output_color, 1.0)));
    }
}
