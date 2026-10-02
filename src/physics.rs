use crate::world::World;

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl AABB {
    pub fn for_player(x: f64, y: f64, z: f64) -> Self {
        Self {
            min_x: x - 0.3,
            max_x: x + 0.3,
            min_y: y,
            max_y: y + 1.8,
            min_z: z - 0.3,
            max_z: z + 0.3,
        }
    }

    pub fn offset(&self, dx: f64, dy: f64, dz: f64) -> Self {
        Self {
            min_x: self.min_x + dx,
            max_x: self.max_x + dx,
            min_y: self.min_y + dy,
            max_y: self.max_y + dy,
            min_z: self.min_z + dz,
            max_z: self.max_z + dz,
        }
    }

    pub fn clip_x_collide(&self, other: &AABB, mut dx: f64) -> f64 {
        if other.max_y <= self.min_y || other.min_y >= self.max_y {
            return dx;
        }
        if other.max_z <= self.min_z || other.min_z >= self.max_z {
            return dx;
        }
        if dx > 0.0 && self.max_x <= other.min_x {
            let max = other.min_x - self.max_x;
            if max < dx {
                dx = max;
            }
        }
        if dx < 0.0 && self.min_x >= other.max_x {
            let max = other.max_x - self.min_x;
            if max > dx {
                dx = max;
            }
        }
        dx
    }

    pub fn clip_y_collide(&self, other: &AABB, mut dy: f64) -> f64 {
        if other.max_x <= self.min_x || other.min_x >= self.max_x {
            return dy;
        }
        if other.max_z <= self.min_z || other.min_z >= self.max_z {
            return dy;
        }
        if dy > 0.0 && self.max_y <= other.min_y {
            let max = other.min_y - self.max_y;
            if max < dy {
                dy = max;
            }
        }
        if dy < 0.0 && self.min_y >= other.max_y {
            let max = other.max_y - self.min_y;
            if max > dy {
                dy = max;
            }
        }
        dy
    }

    pub fn clip_z_collide(&self, other: &AABB, mut dz: f64) -> f64 {
        if other.max_x <= self.min_x || other.min_x >= self.max_x {
            return dz;
        }
        if other.max_y <= self.min_y || other.min_y >= self.max_y {
            return dz;
        }
        if dz > 0.0 && self.max_z <= other.min_z {
            let max = other.min_z - self.max_z;
            if max < dz {
                dz = max;
            }
        }
        if dz < 0.0 && self.min_z >= other.max_z {
            let max = other.max_z - self.min_z;
            if max > dz {
                dz = max;
            }
        }
        dz
    }
}

pub fn has_collision(block_id: u16) -> bool {
    match block_id {
        0 | 6 | 8 | 9 | 10 | 11 | 27 | 28 | 30 | 31 | 32 | 37 | 38 | 39 | 40 | 50 | 51
        | 55 | 59 | 63 | 66 | 68 | 69 | 70 | 72 | 75 | 76 | 77 | 78 | 83 | 90 | 104
        | 105 | 106 | 115 | 119 | 131 | 132 | 141 | 142 | 143 | 175 => false,
        _ => true,
    }
}

pub fn is_in_water(world: &World, bb: &AABB) -> bool {
    let check_min_y = bb.min_y + 0.4;
    let check_max_y = bb.max_y - 0.4;
    
    let min_x = (bb.min_x + 0.001).floor() as i32;
    let max_x = (bb.max_x - 0.001).floor() as i32;
    let min_y = check_min_y.floor() as i32;
    let max_y = check_max_y.floor() as i32;
    let min_z = (bb.min_z + 0.001).floor() as i32;
    let max_z = (bb.max_z - 0.001).floor() as i32;

    for bx in min_x..=max_x {
        for by in min_y..=max_y {
            for bz in min_z..=max_z {
                let block = world.get_block(bx, by, bz);
                if block.id == 8 || block.id == 9 {
                    let height = if block.meta >= 8 {
                        14.0 / 16.0
                    } else {
                        (14.0 - (block.meta & 7) as f64 * 1.5) / 16.0
                    };
                    let water_y = by as f64 + height;
                    if check_min_y < water_y {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn is_in_lava(world: &World, bb: &AABB) -> bool {
    let check_min_y = bb.min_y + 0.4;
    let check_max_y = bb.max_y - 0.4;
    
    let min_x = (bb.min_x + 0.001).floor() as i32;
    let max_x = (bb.max_x - 0.001).floor() as i32;
    let min_y = check_min_y.floor() as i32;
    let max_y = check_max_y.floor() as i32;
    let min_z = (bb.min_z + 0.001).floor() as i32;
    let max_z = (bb.max_z - 0.001).floor() as i32;

    for bx in min_x..=max_x {
        for by in min_y..=max_y {
            for bz in min_z..=max_z {
                let block = world.get_block(bx, by, bz);
                if block.id == 10 || block.id == 11 {
                    let height = if block.meta >= 8 {
                        14.0 / 16.0
                    } else {
                        (14.0 - (block.meta & 7) as f64 * 1.5) / 16.0
                    };
                    let lava_y = by as f64 + height;
                    if check_min_y < lava_y {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn get_water_flow(world: &World, bb: &AABB) -> (f64, f64, f64) {
    let check_min_y = bb.min_y + 0.4;
    let check_max_y = bb.max_y - 0.4;

    let min_x = (bb.min_x + 0.001).floor() as i32;
    let max_x = (bb.max_x - 0.001).floor() as i32;
    let min_y = check_min_y.floor() as i32;
    let max_y = check_max_y.floor() as i32;
    let min_z = (bb.min_z + 0.001).floor() as i32;
    let max_z = (bb.max_z - 0.001).floor() as i32;

    let mut flow_x = 0.0f64;
    let flow_y = 0.0f64;
    let mut flow_z = 0.0f64;

    let get_effective_decay = |x: i32, y: i32, z: i32| -> i32 {
        let b = world.get_block(x, y, z);
        if b.id == 8 || b.id == 9 {
            if b.meta >= 8 { 8 } else { (b.meta & 7) as i32 }
        } else {
            -1
        }
    };

    for bx in min_x..=max_x {
        for by in min_y..=max_y {
            for bz in min_z..=max_z {
                let block = world.get_block(bx, by, bz);
                if block.id == 8 || block.id == 9 {
                    let height = if block.meta >= 8 {
                        14.0 / 16.0
                    } else {
                        (14.0 - (block.meta & 7) as f64 * 1.5) / 16.0
                    };
                    let water_y = by as f64 + height;
                    if check_min_y >= water_y {
                        continue;
                    }

                    if block.meta >= 8 {
                        continue; // Falling water doesn't push horizontally
                    }

                    let decay = get_effective_decay(bx, by, bz);
                    if decay < 0 {
                        continue;
                    }

                    let neighbors = [
                        (-1, 0),
                        (1, 0),
                        (0, -1),
                        (0, 1),
                    ];

                    for &(dx, dz) in &neighbors {
                        let nx = bx + dx;
                        let nz = bz + dz;
                        let nd = get_effective_decay(nx, by, nz);

                        if nd < 0 {
                            let nb_block = world.get_block(nx, by, nz);
                            if !has_collision(nb_block.id) {
                                let below_decay = get_effective_decay(nx, by - 1, nz);
                                if below_decay >= 0 {
                                    let diff = below_decay - (decay - 8);
                                    flow_x += (dx as f64) * (diff as f64);
                                    flow_z += (dz as f64) * (diff as f64);
                                }
                            }
                        } else {
                            let diff = nd - decay;
                            flow_x += (dx as f64) * (diff as f64);
                            flow_z += (dz as f64) * (diff as f64);
                        }
                    }
                }
            }
        }
    }

    let len = (flow_x * flow_x + flow_z * flow_z).sqrt();
    let (push_x, push_z) = if len > 1e-4 {
        (flow_x / len * 0.035, flow_z / len * 0.035)
    } else {
        (0.0, 0.0)
    };

    (push_x, flow_y, push_z)
}

pub fn is_liquid_block_in_aabb(world: &World, bb: &AABB) -> bool {
    let min_x = (bb.min_x + 0.001).floor() as i32;
    let max_x = (bb.max_x - 0.001).floor() as i32;
    let min_y = (bb.min_y + 0.001).floor() as i32;
    let max_y = (bb.max_y - 0.001).floor() as i32;
    let min_z = (bb.min_z + 0.001).floor() as i32;
    let max_z = (bb.max_z - 0.001).floor() as i32;

    for bx in min_x..=max_x {
        for by in min_y..=max_y {
            for bz in min_z..=max_z {
                let id = world.get_block(bx, by, bz).id;
                if id == 8 || id == 9 || id == 10 || id == 11 {
                    return true;
                }
            }
        }
    }
    false
}

pub fn get_colliding_bounding_boxes(world: &World, box_to_check: &AABB) -> Vec<AABB> {
    let min_x = (box_to_check.min_x - 1.0).floor() as i32;
    let max_x = (box_to_check.max_x + 1.0).ceil() as i32;
    let min_y = (box_to_check.min_y - 1.0).floor() as i32;
    let max_y = (box_to_check.max_y + 1.0).ceil() as i32;
    let min_z = (box_to_check.min_z - 1.0).floor() as i32;
    let max_z = (box_to_check.max_z + 1.0).ceil() as i32;

    let mut boxes = Vec::new();
    for bx in min_x..=max_x {
        for by in min_y..=max_y {
            for bz in min_z..=max_z {
                let block = world.get_block(bx, by, bz);
                if has_collision(block.id) {
                    let (min_y_off, max_y_off) = match block.id {
                        120 => (0.0, 13.0 / 16.0),
                        44 | 126 => {
                            if (block.meta & 8) != 0 {
                                (0.5, 1.0)
                            } else {
                                (0.0, 0.5)
                            }
                        }
                        26 => (0.0, 9.0 / 16.0),
                        171 => (0.0, 1.0 / 16.0),
                        _ => (0.0, 1.0),
                    };
                    boxes.push(AABB {
                        min_x: bx as f64,
                        min_y: by as f64 + min_y_off,
                        min_z: bz as f64,
                        max_x: (bx + 1) as f64,
                        max_y: by as f64 + max_y_off,
                        max_z: (bz + 1) as f64,
                    });
                }
            }
        }
    }
    boxes
}

#[derive(Clone, Copy, Debug)]
pub struct MoveResult {
    pub new_x: f64,
    pub new_y: f64,
    pub new_z: f64,
    pub new_motion_x: f64,
    pub new_motion_y: f64,
    pub new_motion_z: f64,
    pub on_ground: bool,
    pub collided_horizontally: bool,
    pub did_jump: bool,
}

pub fn move_and_collide_full(
    world: &World,
    curr_x: f64,
    curr_y: f64,
    curr_z: f64,
    motion_x: f64,
    motion_y: f64,
    motion_z: f64,
    on_ground: bool,
    sneaking: bool,
) -> MoveResult {
    let start_box = AABB::for_player(curr_x, curr_y, curr_z);
    let mut dx = motion_x;
    let mut dy = motion_y;
    let mut dz = motion_z;

    // Safe walk edge checking when sneaking on ground
    if on_ground && sneaking {
        let sneak_check = 0.05;
        while dx != 0.0
            && get_colliding_bounding_boxes(world, &start_box.offset(dx, -1.0, 0.0)).is_empty()
        {
            if dx < sneak_check && dx >= -sneak_check {
                dx = 0.0;
            } else if dx > 0.0 {
                dx -= sneak_check;
            } else {
                dx += sneak_check;
            }
        }

        while dz != 0.0
            && get_colliding_bounding_boxes(world, &start_box.offset(0.0, -1.0, dz)).is_empty()
        {
            if dz < sneak_check && dz >= -sneak_check {
                dz = 0.0;
            } else if dz > 0.0 {
                dz -= sneak_check;
            } else {
                dz += sneak_check;
            }
        }

        while dx != 0.0
            && dz != 0.0
            && get_colliding_bounding_boxes(world, &start_box.offset(dx, -1.0, dz)).is_empty()
        {
            if dx < sneak_check && dx >= -sneak_check {
                dx = 0.0;
            } else if dx > 0.0 {
                dx -= sneak_check;
            } else {
                dx += sneak_check;
            }
            if dz < sneak_check && dz >= -sneak_check {
                dz = 0.0;
            } else if dz > 0.0 {
                dz -= sneak_check;
            } else {
                dz += sneak_check;
            }
        }
    }

    let expanded_box = AABB {
        min_x: start_box.min_x + dx.min(0.0) - 0.6,
        max_x: start_box.max_x + dx.max(0.0) + 0.6,
        min_y: start_box.min_y + dy.min(0.0) - 0.6,
        max_y: start_box.max_y + dy.max(0.0) + 0.6,
        min_z: start_box.min_z + dz.min(0.0) - 0.6,
        max_z: start_box.max_z + dz.max(0.0) + 0.6,
    };
    let block_boxes = get_colliding_bounding_boxes(world, &expanded_box);

    let orig_dx = dx;
    let orig_dy = dy;
    let orig_dz = dz;

    // Resolve Y axis
    let mut cur_box = start_box;
    for b in &block_boxes {
        dy = cur_box.clip_y_collide(b, dy);
    }
    cur_box = cur_box.offset(0.0, dy, 0.0);

    let flag_ground = on_ground || (orig_dy != dy && orig_dy < 0.0);

    // Resolve X axis
    for b in &block_boxes {
        dx = cur_box.clip_x_collide(b, dx);
    }
    cur_box = cur_box.offset(dx, 0.0, 0.0);

    // Resolve Z axis
    for b in &block_boxes {
        dz = cur_box.clip_z_collide(b, dz);
    }

    // Step assist (0.6 blocks)
    let step_height = 0.6;
    if step_height > 0.0 && flag_ground && (orig_dx != dx || orig_dz != dz) {
        let unstepped_dx = dx;
        let unstepped_dz = dz;

        let mut step_box = start_box;
        let mut step_dy = step_height;
        for b in &block_boxes {
            step_dy = step_box.clip_y_collide(b, step_dy);
        }
        step_box = step_box.offset(0.0, step_dy, 0.0);

        let mut step_dx = orig_dx;
        for b in &block_boxes {
            step_dx = step_box.clip_x_collide(b, step_dx);
        }
        step_box = step_box.offset(step_dx, 0.0, 0.0);

        let mut step_dz = orig_dz;
        for b in &block_boxes {
            step_dz = step_box.clip_z_collide(b, step_dz);
        }
        step_box = step_box.offset(0.0, 0.0, step_dz);

        let mut step_down_dy = -step_dy;
        for b in &block_boxes {
            step_down_dy = step_box.clip_y_collide(b, step_down_dy);
        }

        if unstepped_dx * unstepped_dx + unstepped_dz * unstepped_dz
            < step_dx * step_dx + step_dz * step_dz
        {
            dx = step_dx;
            dy = step_dy + step_down_dy;
            dz = step_dz;
        }
    }

    let landed = orig_dy < 0.0 && dy != orig_dy;
    let collided_horizontally = orig_dx != dx || orig_dz != dz;

    let mut new_motion_x = motion_x;
    let mut new_motion_y = motion_y;
    let mut new_motion_z = motion_z;

    if orig_dx != dx {
        new_motion_x = 0.0;
    }
    if orig_dy != dy {
        new_motion_y = 0.0;
    }
    if orig_dz != dz {
        new_motion_z = 0.0;
    }

    MoveResult {
        new_x: curr_x + dx,
        new_y: curr_y + dy,
        new_z: curr_z + dz,
        new_motion_x,
        new_motion_y,
        new_motion_z,
        on_ground: landed,
        collided_horizontally,
        did_jump: false,
    }
}

pub fn tick_movement(
    world: &World,
    x: f64,
    y: f64,
    z: f64,
    mut motion_x: f64,
    mut motion_y: f64,
    mut motion_z: f64,
    yaw: f32,
    _pitch: f32,
    on_ground: bool,
    input_forward: bool,
    input_backward: bool,
    input_left: bool,
    input_right: bool,
    input_jump: bool,
    sneaking: bool,
    sprinting: bool,
) -> MoveResult {
    let mut strafe = 0.0f32;
    let mut forward = 0.0f32;

    if input_forward {
        forward += 1.0;
    }
    if input_backward {
        forward -= 1.0;
    }
    if input_left {
        strafe += 1.0;
    }
    if input_right {
        strafe -= 1.0;
    }

    if sneaking {
        strafe *= 0.3;
        forward *= 0.3;
    }

    // Vanilla 1.7.10 multiplies raw inputs by 0.98 before applying movement!
    strafe *= 0.98;
    forward *= 0.98;

    let player_box = AABB::for_player(x, y, z);
    let in_water = is_in_water(world, &player_box);
    let in_lava = !in_water && is_in_lava(world, &player_box);

    if in_water || in_lava {
        let drag = if in_water { 0.8f64 } else { 0.5f64 };

        if input_jump {
            motion_y += 0.04;
        }

        let speed_factor = 0.02f64;
        let mut move_dist = (strafe * strafe + forward * forward).sqrt();
        if move_dist >= 1e-4 {
            if move_dist < 1.0 {
                move_dist = 1.0;
            }
            let scale = (speed_factor as f32) / move_dist;
            let s = (strafe * scale) as f64;
            let f = (forward * scale) as f64;

            let yaw_rad = (yaw as f64).to_radians();
            let sin = yaw_rad.sin();
            let cos = yaw_rad.cos();

            motion_x += s * cos - f * sin;
            motion_z += f * cos + s * sin;
        }

        if in_water {
            let (flow_x, flow_y, flow_z) = get_water_flow(world, &player_box);
            motion_x += flow_x;
            motion_y += flow_y;
            motion_z += flow_z;
        }

        let mut res = move_and_collide_full(
            world,
            x,
            y,
            z,
            motion_x,
            motion_y,
            motion_z,
            on_ground,
            sneaking,
        );

        res.new_motion_x *= drag;
        res.new_motion_y *= drag;
        res.new_motion_z *= drag;
        res.new_motion_y -= 0.02;

        if res.collided_horizontally {
            let actual_dy = res.new_y - y;
            let offset_bb = player_box.offset(motion_x, motion_y + 0.6 - actual_dy, motion_z);
            if is_liquid_block_in_aabb(world, &offset_bb) {
                res.new_motion_y = 0.3;
            }
        }

        if res.new_motion_x.abs() < 0.005 {
            res.new_motion_x = 0.0;
        }
        if res.new_motion_y.abs() < 0.005 {
            res.new_motion_y = 0.0;
        }
        if res.new_motion_z.abs() < 0.005 {
            res.new_motion_z = 0.0;
        }

        return res;
    }

    // Jump velocity and sprint boost on land
    let mut actually_jumped = false;
    if input_jump && on_ground {
        actually_jumped = true;
        motion_y = 0.42_f32 as f64;
        if sprinting {
            let yaw_rad = (yaw as f32).to_radians();
            motion_x -= (yaw_rad.sin() * 0.2_f32) as f64;
            motion_z += (yaw_rad.cos() * 0.2_f32) as f64;
        }
    }

    // Acceleration calculation
    
    let speed_factor = if on_ground {
        let base_speed = if sprinting { 0.13000001_f32 } else { 0.1_f32 };
        let f2 = 0.16277136_f32 / (0.546_f32 * 0.546_f32 * 0.546_f32);
        (base_speed * f2) as f64
    } else {
        if sprinting {
            0.026_f32 as f64
        } else {
            0.02_f32 as f64
        }
    };

    let mut move_dist = (strafe * strafe + forward * forward).sqrt();
    if move_dist >= 1e-4 {
        if move_dist < 1.0 {
            move_dist = 1.0;
        }
        let scale = (speed_factor as f32) / move_dist;
        let s = (strafe * scale) as f64;
        let f = (forward * scale) as f64;

        let yaw_rad = (yaw as f64).to_radians();
        let sin = yaw_rad.sin();
        let cos = yaw_rad.cos();

        motion_x += s * cos - f * sin;
        motion_z += f * cos + s * sin;
    }

    let mut res = move_and_collide_full(
        world,
        x,
        y,
        z,
        motion_x,
        motion_y,
        motion_z,
        on_ground,
        sneaking,
    );

    // Apply gravity and air drag
    res.did_jump = actually_jumped;
    // Apply gravity and air drag
    res.new_motion_y -= 0.08;
    res.new_motion_y *= 0.9800000190734863_f64;

    let friction = if on_ground {
        0.546_f32 as f64
    } else {
        0.91_f32 as f64
    };
    res.new_motion_x *= friction;
    res.new_motion_z *= friction;

    if res.new_motion_x.abs() < 0.005 {
        res.new_motion_x = 0.0;
    }
    if res.new_motion_y.abs() < 0.005 {
        res.new_motion_y = 0.0;
    }
    if res.new_motion_z.abs() < 0.005 {
        res.new_motion_z = 0.0;
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_collision() {
        assert!(!has_collision(0)); // Air
        assert!(!has_collision(8)); // Flowing water
        assert!(!has_collision(9)); // Still water
        assert!(!has_collision(31)); // Tall grass
        assert!(!has_collision(50)); // Torch
        assert!(has_collision(1)); // Stone
        assert!(has_collision(2)); // Grass
    }

    #[test]
    fn test_water_sinking_and_jump() {
        let mut world = World::new();
        world.insert_chunk(crate::world::ChunkColumn::new(0, 0));
        world.set_block(0, 60, 0, crate::world::Block { id: 9, meta: 0, block_light: 0, sky_light: 15 });

        let bb = AABB::for_player(0.5, 60.0, 0.5);
        assert!(is_in_water(&world, &bb));

        // Test sinking
        let res_sink = tick_movement(&world, 0.5, 60.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, false, false, false, false, false, false, false, false);
        assert!(res_sink.new_motion_y < 0.0);

        // Test swim up
        let res_swim = tick_movement(&world, 0.5, 60.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, false, false, false, false, false, true, false, false);
        assert!(res_swim.new_motion_y > 0.0);
    }

    #[test]
    fn test_water_flow_push() {
        let mut world = World::new();
        world.insert_chunk(crate::world::ChunkColumn::new(0, 0));
        // Water at (0, 60, 0) with decay 1, neighbor at (1, 60, 0) with decay 3
        world.set_block(0, 60, 0, crate::world::Block { id: 8, meta: 1, block_light: 0, sky_light: 15 });
        world.set_block(1, 60, 0, crate::world::Block { id: 8, meta: 3, block_light: 0, sky_light: 15 });

        let bb = AABB::for_player(0.5, 60.0, 0.5);
        let (push_x, _push_y, push_z) = get_water_flow(&world, &bb);
        assert!(push_x > 0.0, "Expected push_x > 0 towards downstream (+X), got {}", push_x);
        assert_eq!(push_z, 0.0);
    }

    #[test]
    fn test_portal_and_frame_physics() {
        assert!(!has_collision(90)); // Nether portal
        assert!(!has_collision(119)); // End portal
        assert!(has_collision(120)); // End portal frame

        let mut world = World::new();
        world.insert_chunk(crate::world::ChunkColumn::new(0, 0));
        world.set_block(0, 64, 0, crate::world::Block { id: 120, meta: 0, block_light: 0, sky_light: 15 });

        let bb = AABB::for_player(0.5, 64.0, 0.5);
        let boxes = get_colliding_bounding_boxes(&world, &bb);
        assert_eq!(boxes.len(), 1);
        assert!((boxes[0].max_y - (64.0 + 13.0 / 16.0)).abs() < 1e-4);
    }
}

