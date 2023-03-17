use bytemuck::{Zeroable, Pod};
use glam::{Mat4, Vec2, Vec4, Vec4Swizzles};
use crate::ray::Ray;

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
pub struct InverseCamera {
    pub inverse_view: Mat4,
    pub inverse_centered_view: Mat4,
    pub inverse_projection: Mat4,
}

impl InverseCamera {
    pub fn new(view: &Mat4, projection: &Mat4) -> Self {
        let inverse_view = view.inverse();
        let inverse_projection = projection.inverse();
        let mut inverse_centered_view = inverse_view.clone();

        inverse_centered_view.col_mut(3).x = 0.0;
        inverse_centered_view.col_mut(3).y = 0.0;
        inverse_centered_view.col_mut(3).z = 0.0;

        Self {
            inverse_view,
            inverse_centered_view,
            inverse_projection
        }
    }

    pub fn create_ray(&self, screen_coords: Vec2) -> Ray {
        Ray {
            origin: (self.inverse_view * Vec4::new(0.0, 0.0, 0.0, 1.0)).xyz(),
            direction: (self.inverse_centered_view * self.inverse_projection * Vec4::from((screen_coords, 0.0, 1.0))).xyz(),
        }
    }
}