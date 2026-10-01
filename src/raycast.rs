use crate::world::{Block, World};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockFace {
    Bottom = 0,
    Top = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

#[allow(dead_code)]
impl BlockFace {
    pub fn name(&self) -> &'static str {
        match self {
            BlockFace::Bottom => "Bottom (-Y)",
            BlockFace::Top => "Top (+Y)",
            BlockFace::North => "North (-Z)",
            BlockFace::South => "South (+Z)",
            BlockFace::West => "West (-X)",
            BlockFace::East => "East (+X)",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub block_x: i32,
    pub block_y: i32,
    pub block_z: i32,
    pub face: BlockFace,
    pub distance: f64,
    pub block: Block,
}

#[allow(dead_code)]
pub fn get_look_vector(yaw: f32, pitch: f32) -> (f64, f64, f64) {
    let yaw_rad = (yaw as f64).to_radians();
    let pitch_rad = (pitch as f64).to_radians();

    let x = -yaw_rad.sin() * pitch_rad.cos();
    let y = -pitch_rad.sin();
    let z = yaw_rad.cos() * pitch_rad.cos();

    (x, y, z)
}

#[allow(dead_code)]
pub fn raycast_world(
    world: &World,
    origin: (f64, f64, f64),
    dir: (f64, f64, f64),
    max_dist: f64,
) -> Option<RaycastHit> {
    let (mut curr_x, mut curr_y, mut curr_z) = (
        origin.0.floor() as i32,
        origin.1.floor() as i32,
        origin.2.floor() as i32,
    );

    let (dx, dy, dz) = dir;

    let step_x = if dx > 0.0 { 1 } else { -1 };
    let step_y = if dy > 0.0 { 1 } else { -1 };
    let step_z = if dz > 0.0 { 1 } else { -1 };

    let delta_tx = if dx != 0.0 { (1.0 / dx).abs() } else { f64::INFINITY };
    let delta_ty = if dy != 0.0 { (1.0 / dy).abs() } else { f64::INFINITY };
    let delta_tz = if dz != 0.0 { (1.0 / dz).abs() } else { f64::INFINITY };

    let mut t_max_x = if dx > 0.0 {
        ((curr_x as f64 + 1.0) - origin.0) * delta_tx
    } else if dx < 0.0 {
        (origin.0 - curr_x as f64) * delta_tx
    } else {
        f64::INFINITY
    };

    let mut t_max_y = if dy > 0.0 {
        ((curr_y as f64 + 1.0) - origin.1) * delta_ty
    } else if dy < 0.0 {
        (origin.1 - curr_y as f64) * delta_ty
    } else {
        f64::INFINITY
    };

    let mut t_max_z = if dz > 0.0 {
        ((curr_z as f64 + 1.0) - origin.2) * delta_tz
    } else if dz < 0.0 {
        (origin.2 - curr_z as f64) * delta_tz
    } else {
        f64::INFINITY
    };

    let start_block = world.get_block(curr_x, curr_y, curr_z);
    if !start_block.is_air() && start_block.id != 8 && start_block.id != 9 && start_block.id != 10 && start_block.id != 11 {
        return Some(RaycastHit {
            block_x: curr_x,
            block_y: curr_y,
            block_z: curr_z,
            face: BlockFace::Top,
            distance: 0.0,
            block: start_block,
        });
    }

    let mut t = 0.0;

    while t <= max_dist {
        let hit_face;
        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                curr_x += step_x;
                t = t_max_x;
                t_max_x += delta_tx;
                hit_face = if step_x > 0 { BlockFace::West } else { BlockFace::East };
            } else {
                curr_z += step_z;
                t = t_max_z;
                t_max_z += delta_tz;
                hit_face = if step_z > 0 { BlockFace::North } else { BlockFace::South };
            }
        } else {
            if t_max_y < t_max_z {
                curr_y += step_y;
                t = t_max_y;
                t_max_y += delta_ty;
                hit_face = if step_y > 0 { BlockFace::Bottom } else { BlockFace::Top };
            } else {
                curr_z += step_z;
                t = t_max_z;
                t_max_z += delta_tz;
                hit_face = if step_z > 0 { BlockFace::North } else { BlockFace::South };
            }
        }

        if t > max_dist {
            break;
        }

        let block = world.get_block(curr_x, curr_y, curr_z);
        if !block.is_air() && block.id != 8 && block.id != 9 && block.id != 10 && block.id != 11 {
            return Some(RaycastHit {
                block_x: curr_x,
                block_y: curr_y,
                block_z: curr_z,
                face: hit_face,
                distance: t,
                block,
            });
        }
    }

    None
}
