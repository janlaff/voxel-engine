use std::f32::consts::PI;
use voxel_engine_gpu::glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use voxel_engine_gpu::InverseCamera;
use winit::dpi::{LogicalSize, PhysicalPosition};

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub view: Mat4,
    pub projection: Mat4,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, screen_size: LogicalSize<f32>) -> Self {
        let up = Vec3::new(0.0, -1.0, 0.0);
        let view = Mat4::look_at_rh(position, target, up);
        let projection = Mat4::perspective_rh(
            45.0_f32.to_radians(),
            screen_size.width / screen_size.height,
            0.1,
            100.0,
        );

        Self {
            position,
            target,
            up,
            view,
            projection,
        }
    }

    fn update_view(&mut self) {
        self.view = Mat4::look_at_rh(self.position, self.target, self.up);
    }

    pub fn update_projection(&mut self, screen_size: LogicalSize<f32>) {
        self.projection = Mat4::perspective_rh(
            45.0_f32.to_radians(),
            screen_size.width / screen_size.height,
            0.1,
            100.0,
        );
    }

    pub fn inverse(&self) -> InverseCamera {
        InverseCamera::new(&self.view, &self.projection)
    }

    pub fn arcball_rotate(&mut self, delta: PhysicalPosition<f32>, screen_size: LogicalSize<f32>) {
        let mut position = Vec4::from((self.position, 1.0));
        let pivot = Vec4::from((self.target, 1.0));
        let right = self.view.transpose().col(0).xyz();

        let delta_angle_x = 2.0 * PI / screen_size.width;
        let delta_angle_y = 2.0 * PI / screen_size.height;

        let x_angle = delta.x * delta_angle_x;
        let mut y_angle = delta.y * delta_angle_y;

        let view_dir = -self.view.transpose().col(2).xyz();
        let cos_angle = view_dir.dot(self.up);

        const MAX_ANGLE: f32 = 0.9;
        if cos_angle * y_angle.signum() > MAX_ANGLE {
            y_angle = (MAX_ANGLE - (cos_angle * y_angle.signum())) * y_angle.signum();
        }

        let rotation_x = Mat4::from_axis_angle(-self.up, x_angle);
        position = (rotation_x * (position - pivot)) + pivot;

        let rotation_y = Mat4::from_axis_angle(right, y_angle);
        position = (rotation_y * (position - pivot)) + pivot;

        self.position = position.xyz();
        self.update_view();
    }
}
