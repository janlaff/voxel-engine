use crate::{intersect_box, sky_color, OctreeNode, Ray, Stack};
use glam::{uvec2, vec2, vec3, IVec2, UVec2, Vec2, Vec3};
use spirv_std::num_traits::Float;

#[inline(always)]
pub fn next_child_index(prev_idx: usize, t1_child: &Vec3, t_exit_child: f32) -> (usize, bool) {
    let mut idx = prev_idx;
    let mut exit_node = false;

    if t1_child.x == t_exit_child {
        if idx & 1 != 0 {
            exit_node = true;
        }
        idx |= 1;
    } else if t1_child.y == t_exit_child {
        if idx & 2 != 0 {
            exit_node = true;
        }
        idx |= 2;
    } else {
        if idx & 4 != 0 {
            exit_node = true;
        }
        idx |= 4;
    }
    (idx, exit_node)
}

#[inline(always)]
pub fn child_span(child_idx: usize, t0: &Vec3, tm: &Vec3, t1: &Vec3) -> (Vec3, Vec3) {
    let mut t0_child = *t0;
    let mut t1_child = *tm;

    if (child_idx & 1) != 0 {
        t0_child.x = tm.x;
        t1_child.x = t1.x;
    }
    if (child_idx & 2) != 0 {
        t0_child.y = tm.y;
        t1_child.y = t1.y;
    }
    if (child_idx & 4) != 0 {
        t0_child.z = tm.z;
        t1_child.z = t1.z;
    }

    (t0_child, t1_child)
}

// Perform ray cube intersection and return the intersection points
fn intersect_cube(ray: &Ray, center: &Vec3, extent: f32) -> (Vec3, Vec3) {
    // Calculate cube span
    let cube_min_corner = *center - extent;
    let cube_max_corner = *center + extent;

    // Calculate intersection points
    let t0 = (cube_min_corner.min(cube_max_corner) - ray.origin) / ray.direction;
    let t1 = (cube_max_corner.max(cube_min_corner) - ray.origin) / ray.direction;

    // Return intersections and swap axis values if necessary (negative ray directions)
    (t0.min(t1), t1.max(t0))
}

#[inline(always)]
fn initial_child_index(t_enter: f32, tm: &Vec3) -> usize {
    let mut idx = 0b000;

    if t_enter > tm.x {
        idx |= 1;
    }
    if t_enter > tm.y {
        idx |= 2;
    }
    if t_enter > tm.z {
        idx |= 4;
    }

    idx
}

pub fn trace_octree(ray: &Ray, octree: &[OctreeNode]) -> Vec3 {
    const S_MAX: usize = 23; // Maximum scale (number of float mantissa bits)

    // Calculate intersection points
    let (t0, t1) = intersect_cube(ray, &vec3(0.0, 0.0, 0.0), 1.0);

    // Calculate intersection distances
    let (t_enter, t_exit) = (t0.max_element(), t1.min_element());

    // Ray does not intersect
    if t_enter > t_exit {
        return sky_color(&ray);
    }

    let mut index_stack = [0; S_MAX];
    let mut t0_stack = [Vec3::default(); S_MAX];
    let mut t1_stack = [Vec3::default(); S_MAX];

    let mut stack_idx = 0;
    index_stack[stack_idx] = 0;
    t0_stack[stack_idx] = t0;
    t1_stack[stack_idx] = t1;

    // Calculate direction mask
    let mut dir_mask = 0b000;
    if ray.direction.x < 0.0 {
        dir_mask |= 1;
    }
    if ray.direction.y < 0.0 {
        dir_mask |= 2;
    }
    if ray.direction.z < 0.0 {
        dir_mask |= 4;
    }

    let mut child_idx = 0;
    'main: while stack_idx < S_MAX {
        let node = octree[index_stack[stack_idx]];
        let t0 = t0_stack[stack_idx];
        let t1 = t1_stack[stack_idx];

        // Calculate intersection distances
        let (t_enter, t_exit) = (t0.max_element(), t1.min_element());

        // Calculate middle of ray in cube
        let tm = (t0 + t1) / 2.0;

        // Get child index that ray will enter first
        child_idx = initial_child_index(t_enter, &tm);
        let mut exit_node = false;

        // Advance through all siblings
        while !exit_node {
            if node.valid(child_idx ^ dir_mask) {
                if node.leaf(child_idx ^ dir_mask) {
                    break 'main;
                } else {
                    // Push to stack
                    stack_idx += 1;
                    index_stack[stack_idx] = node.child_ptr() as usize + (child_idx ^ dir_mask);
                    (t0_stack[stack_idx], t1_stack[stack_idx]) =
                        child_span(child_idx, &t0, &tm, &t1);
                    continue 'main;
                }
            }

            let (t0_child, t1_child) = child_span(child_idx, &t0, &tm, &t1);
            let t_exit_child = t1_child.min_element();
            (child_idx, exit_node) = next_child_index(child_idx, &t1_child, t_exit_child);
        }

        // POP
        if stack_idx > 0 {
            stack_idx -= 1;
        } else {
            return sky_color(ray);
        }
    }

    // Color based on child index
    vec3(
        ((child_idx ^ dir_mask) & 1) as f32,
        (((child_idx ^ dir_mask) & 2) >> 1) as f32,
        (((child_idx ^ dir_mask) & 4) >> 2) as f32,
    )
}
