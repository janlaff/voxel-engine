use crate::ray::Ray;
use glam::{Vec2, Vec3};
use spirv_std::num_traits::Float;

pub fn intersect_sphere(ray: &Ray, center: Vec3, radius: f32) -> (bool, f32) {
    let oc = ray.origin - center;
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        (false, -1.0)
    } else {
        (true, (-b - discriminant.sqrt()) / (2.0 * a))
    }
}

pub fn intersect_box(ray: &Ray, center: Vec3, extent: f32) -> (bool, f32) {
    let inv_ray_dir = 1.0 / ray.direction;

    let box_min = center - extent;
    let box_max = center + extent;

    let t_min = (box_min - ray.origin) * inv_ray_dir;
    let t_max = (box_max - ray.origin) * inv_ray_dir;

    let t1 = t_min.min(t_max);
    let t2 = t_max.max(t_min);

    let t_near = (t1.x.max(t1.y)).max(t1.z);
    let t_far = (t2.x.min(t2.y)).min(t2.z);

    if t_near <= t_far {
        (true, t_near)
    } else {
        (false, -1.0)
    }
}
