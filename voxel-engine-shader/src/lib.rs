#![no_std]

use spirv_std::{glam::{UVec3, Vec3Swizzles, UVec2, Vec4}, spirv, image::StorageImage2d};

#[spirv(compute(threads(10, 10)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(binding = 0)] output: &mut StorageImage2d,
) {
    let output_size: UVec2 = output.query_size();
    let output_coords = id.xy();
    let output_color = Vec4::new(1.0, 0.0, 0.0, 1.0);

    if output_coords.x >= output_size.x && output_coords.y >= output_size.x {
        return;
    }

    unsafe {
        output.write(output_coords, output_color);
    }
}
