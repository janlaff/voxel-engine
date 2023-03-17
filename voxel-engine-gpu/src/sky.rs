use glam::Vec3;
use crate::ray::Ray;

pub fn sky_color(ray: &Ray) -> Vec3 {
    let t = 0.5 * (ray.direction.y + 1.0);
    return (1.0 - t) * Vec3::splat(1.0) + t * Vec3::new(0.5, 0.7, 1.0);
}
