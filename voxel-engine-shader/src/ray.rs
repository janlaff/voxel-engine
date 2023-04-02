use glam::Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn walk(&self, distance: f32) -> Vec3 {
        self.origin + distance * self.direction
    }
}
