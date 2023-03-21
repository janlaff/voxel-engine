const PPP: Vec3 = Vec3::new(1.0, 1.0, 1.0);
const PNP: Vec3 = Vec3::new(1.0, -1.0, 1.0);
const PNN: Vec3 = Vec3::new(1.0, -1.0, -1.0);
const NPN: Vec3 = Vec3::new(-1.0, 1.0, -1.0);
const NNN: Vec3 = Vec3::new(-1.0, -1.0, -1.0);
const NNP: Vec3 = Vec3::new(-1.0, -1.0, 1.0);
const NPP: Vec3 = Vec3::new(-1.0, 1.0, 1.0);
const PPN: Vec3 = Vec3::new(1.0, 1.0, -1.0);
const POS: [Vec3; 8] = [PNN, PNP, PPN, PPP, NNN, NNP, NPN, NPP];

pub fn trace_octree_top_down(ray: &Ray, octree: &[OctreeNode]) -> Vec3 {
    let center = vec3(0.0, 0.0, 0.0);
    let scale = 1.0;

    if !intersect_box(ray, center, scale).0 {
        return sky_color(ray);
    }

    let mut stack = Stack::new();
    stack.push(0, center, scale);

    let mut closest_hit: f32 = Float::infinity();
    let mut color = Vec3::splat(1.0); //sky_color(ray);

    while !stack.empty() {
        let (index, center, scale) = stack.pop();
        let node = octree[index];

        for child_index in 0..8 {
            if !node.valid(child_index) {
                continue;
            }

            let new_center = center + POS[child_index] * scale * 0.5;

            if node.leaf(child_index) {
                let (hit, distance) = intersect_box(ray, new_center, scale * 0.5);

                if hit && distance < closest_hit {
                    closest_hit = distance;
                    color = (POS[child_index] * 2.0 - 1.0) * scale; //(new_center + 1.0) / 2.0;
                }
                //if hit {
                //    color *= 0.7;
                //}
            } else {
                stack.push(node.child_ptr() as usize, new_center, scale * 0.5);
            }
        }
    }

    color
}

pub fn trace_octree(mut ray: Ray, octree: &[OctreeNode]) -> Vec3 {
    const S_MAX: usize = 23; // Maximum scale (number of float mantissa bits)
    let epsilon = (-(S_MAX as f32)).exp2();

    let mut stack = [UVec2::default(); S_MAX + 1]; // Voxel stack

    // Get rid of small ray direction components
    if ray.direction.x.abs() < epsilon {
        ray.direction.x = ray.direction.x.signum() * epsilon;
    }
    if ray.direction.y.abs() < epsilon {
        ray.direction.y = ray.direction.y.signum() * epsilon;
    }
    if ray.direction.z.abs() < epsilon {
        ray.direction.z = ray.direction.z.signum() * epsilon;
    }

    // Precompute ray coefficients
    let tx_coef = 1.0 / -ray.direction.x.abs();
    let ty_coef = 1.0 / -ray.direction.y.abs();
    let tz_coef = 1.0 / -ray.direction.z.abs();

    let mut tx_bias = tx_coef * ray.origin.x;
    let mut ty_bias = ty_coef * ray.origin.y;
    let mut tz_bias = tz_coef * ray.origin.z;

    // Create octant mask
    let mut octant_mask = 7u32;
    if ray.direction.x > 0.0 {
        octant_mask ^= 1;
        tx_bias = 3.0 * tx_coef - tx_bias;
    }
    if ray.direction.y > 0.0 {
        octant_mask ^= 2;
        ty_bias = 3.0 * ty_coef - ty_bias;
    }
    if ray.direction.z > 0.0 {
        octant_mask ^= 4;
        tz_bias = 3.0 * tz_coef - tz_bias;
    }

    // active span t-values
    let mut t_min = (2.0 * tx_coef - tx_bias)
        .max(2.0 * ty_coef - ty_bias)
        .max(2.0 * tz_coef - tz_bias)
        .max(0.0);
    let mut t_max = (tx_coef - tx_bias)
        .min(ty_coef - ty_bias)
        .min(tz_coef - tz_bias)
        .min(1.0);

    // Initialize to first child of the root
    let mut parent_idx = 0u32;
    let mut child_idx = 0u32;
    let mut node = 0u32;
    let mut position = Vec3::splat(1.0);
    let mut scale = S_MAX - 1;
    let mut scale_exp2 = 0.5;

    if 1.5 * tx_coef - tx_bias > t_min {
        child_idx ^= 1;
        position.x = 1.5;
    }
    if 1.5 * ty_coef - ty_bias > t_min {
        child_idx ^= 2;
        position.y = 1.5;
    }
    if 1.5 * tz_coef - tz_bias > t_min {
        child_idx ^= 4;
        position.z = 1.5;
    }

    while scale < S_MAX {
        // Fetch child descriptor unless already valid
        if node == 0 {
            node = octree[parent_idx as usize].0;
        }

        // Determine maximum t-value
        let tx_corner = position.x * tx_coef - tx_bias;
        let ty_corner = position.y * ty_coef - ty_bias;
        let tz_corner = position.z * tz_coef - tz_bias;
        let tc_max = tx_corner.min(ty_corner).min(tz_corner);

        // Process voxel if bit in valid mask is set and active t-span not empty
        let child_shift = child_idx ^ octant_mask;
        let child_masks = node << child_shift;

        if (child_masks & 0x8000) != 0 && t_min <= t_max {
            // TODO: terminate if ray is small enough

            // INTERSECT
            // Intersect active t-span with the cube
            let tv_max = t_max.min(tc_max);
            let half = scale_exp2 * 0.5;
            let tx_center = half * tx_coef + tx_corner;
            let ty_center = half * ty_coef + ty_corner;
            let tz_center = half * tz_coef + tz_corner;

            // TODO: contour maybe

            // Descend to first child
            if t_min <= tv_max {
                // Terminate if bit in non-leaf mask is not set
                if (child_masks & 0x0080) == 0 {
                    break;
                }

                // PUSH
                // Write parent to stack
                stack[scale] = uvec2(parent_idx, t_max.to_bits());

                // Find child descriptor
                let child_offset = node >> 18;
                let sibling_count = (child_masks & 127).count_ones();

                if (node & 0x10000) != 0 {
                    // far
                    parent_idx += sibling_count;
                }

                // Select child voxel that ray enters first
                child_idx = 0;
                scale -= 1;
                scale_exp2 = half;

                if tx_center > t_min {
                    child_idx ^= 1;
                    position.x += scale_exp2;
                }
                if ty_center > t_min {
                    child_idx ^= 2;
                    position.y += scale_exp2;
                }
                if tz_center > t_min {
                    child_idx ^= 4;
                    position.z += scale_exp2;
                }

                // Update active t-span and invalidate child descriptor
                t_max = tv_max;
                node = 0;
                continue;
            }
        }

        // ADVANCE
        // Step along the ray

        let mut step_mask = 0u32;
        if tx_corner <= tc_max {
            step_mask ^= 1;
            position.x -= scale_exp2;
        }
        if ty_corner <= tc_max {
            step_mask ^= 2;
            position.y -= scale_exp2;
        }
        if tz_corner <= tc_max {
            step_mask ^= 4;
            position.z -= scale_exp2;
        }

        // Update active t-span
        t_min = tc_max;
        child_idx ^= step_mask;

        // Preceed with pop if the bit flips disagree with ray direction
        if (child_idx & step_mask) != 0 {
            // POP
            // Find highest differing bits

            let mut differing_bits = 0u32;
            if (step_mask & 1) != 0 {
                differing_bits |= position.x.to_bits() ^ (position.x + scale_exp2).to_bits();
            }
            if (step_mask & 2) != 0 {
                differing_bits |= position.y.to_bits() ^ (position.y + scale_exp2).to_bits();
            }
            if (step_mask & 4) != 0 {
                differing_bits |= position.z.to_bits() ^ (position.z + scale_exp2).to_bits();
            }
            scale = (((differing_bits as f32).to_bits() >> 23) - 127) as usize;
            scale_exp2 = f32::from_bits(((scale - S_MAX + 127) << 23) as u32);

            // Restore parent voxel from stack
            let entry = stack[scale];
            parent_idx = entry.x;
            t_max = f32::from_bits(entry.y);

            // Round cube position and extract child slot index
            let shx = position.x.to_bits() >> scale;
            let shy = position.y.to_bits() >> scale;
            let shz = position.z.to_bits() >> scale;
            position.x = f32::from_bits(shx << scale);
            position.y = f32::from_bits(shy << scale);
            position.z = f32::from_bits(shz << scale);
            child_idx = (shx & 1) | ((shy & 1) << 1) | ((shz & 1) << 2);

            // Prevent same parent from being stored again
            node = 0;
        }
    }

    // Indicate miss if outside the octree
    if scale >= S_MAX {
        t_min = 2.0;
        return sky_color(&ray);
    }

    // Undo mirroring of coordinate system
    if (octant_mask & 1) == 0 {
        position.x = 3.0 - scale_exp2 - position.x;
    }
    if (octant_mask & 2) == 0 {
        position.y = 3.0 - scale_exp2 - position.y;
    }
    if (octant_mask & 4) == 0 {
        position.z = 3.0 - scale_exp2 - position.z;
    }

    // Output results
    vec3(1.0, 0.0, 0.0)
}
