use crate::raycast::BlockFace;
use crate::world::{Block, ChunkColumn, World};
use bytemuck::{Pod, Zeroable};
use std::collections::HashSet;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 36,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[derive(Default, Clone)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub transparent_vertices: Vec<Vertex>,
    pub transparent_indices: Vec<u32>,
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(2048),
            indices: Vec::with_capacity(3072),
            transparent_vertices: Vec::with_capacity(512),
            transparent_indices: Vec::with_capacity(768),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty() && self.transparent_indices.is_empty()
    }

    pub fn add_box_textured(
        &mut self,
        min: [f32; 3],
        max: [f32; 3],
        u: f32,
        v: f32,
        w: f32,
        h: f32,
        d: f32,
        pivot: [f32; 3],
        yaw_rad: f32,
        pitch_rad: f32,
    ) {
        let cos_p = pitch_rad.cos();
        let sin_p = pitch_rad.sin();
        let cos_y = yaw_rad.cos();
        let sin_y = yaw_rad.sin();

        let rotate_point = |p: [f32; 3]| -> [f32; 3] {
            let tilt_x = p[0];
            let tilt_y = p[1] * cos_p - p[2] * sin_p;
            let tilt_z = p[1] * sin_p + p[2] * cos_p;

            let rot_x = tilt_x * cos_y - tilt_z * sin_y;
            let rot_z = tilt_x * sin_y + tilt_z * cos_y;

            [rot_x + pivot[0], tilt_y + pivot[1], rot_z + pivot[2]]
        };

        let inv = 1.0 / 64.0;
        let uv = |px: f32, py: f32| -> [f32; 2] {
            [px * inv, py * inv]
        };

        let faces = [
            // Top (+Y)
            (
                [
                    [min[0], max[1], max[2]],
                    [max[0], max[1], max[2]],
                    [max[0], max[1], min[2]],
                    [min[0], max[1], min[2]],
                ],
                [
                    uv(u + d, v + d),
                    uv(u + d + w, v + d),
                    uv(u + d + w, v),
                    uv(u + d, v),
                ],
                1.0f32,
            ),
            // Bottom (-Y)
            (
                [
                    [min[0], min[1], min[2]],
                    [max[0], min[1], min[2]],
                    [max[0], min[1], max[2]],
                    [min[0], min[1], max[2]],
                ],
                [
                    uv(u + d + w, v),
                    uv(u + d + w + w, v),
                    uv(u + d + w + w, v + d),
                    uv(u + d + w, v + d),
                ],
                0.5f32,
            ),
            // Front (+Z)
            (
                [
                    [min[0], min[1], max[2]],
                    [max[0], min[1], max[2]],
                    [max[0], max[1], max[2]],
                    [min[0], max[1], max[2]],
                ],
                [
                    uv(u + d, v + d + h),
                    uv(u + d + w, v + d + h),
                    uv(u + d + w, v + d),
                    uv(u + d, v + d),
                ],
                0.8f32,
            ),
            // Back (-Z)
            (
                [
                    [max[0], min[1], min[2]],
                    [min[0], min[1], min[2]],
                    [min[0], max[1], min[2]],
                    [max[0], max[1], min[2]],
                ],
                [
                    uv(u + d + w + d, v + d + h),
                    uv(u + 2.0 * d + 2.0 * w, v + d + h),
                    uv(u + 2.0 * d + 2.0 * w, v + d),
                    uv(u + d + w + d, v + d),
                ],
                0.8f32,
            ),
            // Right (-X)
            (
                [
                    [min[0], min[1], min[2]],
                    [min[0], min[1], max[2]],
                    [min[0], max[1], max[2]],
                    [min[0], max[1], min[2]],
                ],
                [
                    uv(u, v + d + h),
                    uv(u + d, v + d + h),
                    uv(u + d, v + d),
                    uv(u, v + d),
                ],
                0.65f32,
            ),
            // Left (+X)
            (
                [
                    [max[0], min[1], max[2]],
                    [max[0], min[1], min[2]],
                    [max[0], max[1], min[2]],
                    [max[0], max[1], max[2]],
                ],
                [
                    uv(u + d + w, v + d + h),
                    uv(u + 2.0 * d + w, v + d + h),
                    uv(u + 2.0 * d + w, v + d),
                    uv(u + d + w, v + d),
                ],
                0.65f32,
            ),
        ];

        for (quad, uvs, shade) in faces {
            let base_idx = self.vertices.len() as u32;
            let col = [shade, shade, shade, 1.0];
            for (p, uv_coord) in quad.into_iter().zip(uvs.into_iter()) {
                self.vertices.push(Vertex {
                    position: rotate_point(p),
                    uv: uv_coord,
                    color: col,
                    normal: [1.0, 0.0, 1.0],
                });
            }
            self.indices.extend_from_slice(&[
                base_idx,
                base_idx + 1,
                base_idx + 2,
                base_idx,
                base_idx + 2,
                base_idx + 3,
            ]);
        }
    }

    #[allow(dead_code)]
    pub fn append(&mut self, other: &MeshData) {
        let base_idx = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        for idx in &other.indices {
            self.indices.push(base_idx + idx);
        }
        let t_base_idx = self.transparent_vertices.len() as u32;
        self.transparent_vertices.extend_from_slice(&other.transparent_vertices);
        for idx in &other.transparent_indices {
            self.transparent_indices.push(t_base_idx + idx);
        }
    }
}

pub fn build_player_model(
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    yaw_deg: f32,
    pitch_deg: f32,
) -> MeshData {
    let mut mesh = MeshData::new();
    let yaw_rad = yaw_deg.to_radians();
    let pitch_rad = pitch_deg.to_radians();
    let center = [pos_x, pos_y, pos_z];
    let head_pivot = [pos_x, pos_y + 1.50, pos_z];

    // 1. Head (8x8x8, UV origin: 0, 0)
    mesh.add_box_textured(
        [-0.25, 0.0, -0.25],
        [0.25, 0.50, 0.25],
        0.0, 0.0, 8.0, 8.0, 8.0,
        head_pivot,
        yaw_rad,
        pitch_rad,
    );

    // 2. Head Hat / Helmet layer (8x8x8 expanded by 0.025m, UV origin: 32, 0)
    mesh.add_box_textured(
        [-0.275, -0.025, -0.275],
        [0.275, 0.525, 0.275],
        32.0, 0.0, 8.0, 8.0, 8.0,
        head_pivot,
        yaw_rad,
        pitch_rad,
    );

    // 3. Torso (8x12x4, UV origin: 16, 16)
    mesh.add_box_textured(
        [-0.25, 0.75, -0.125],
        [0.25, 1.50, 0.125],
        16.0, 16.0, 8.0, 12.0, 4.0,
        center,
        yaw_rad,
        0.0,
    );

    // 4. Right Arm (4x12x4, UV origin: 40, 16)
    mesh.add_box_textured(
        [-0.50, 0.75, -0.125],
        [-0.25, 1.50, 0.125],
        40.0, 16.0, 4.0, 12.0, 4.0,
        center,
        yaw_rad,
        0.0,
    );

    // 5. Left Arm (4x12x4, UV origin: 32, 48)
    mesh.add_box_textured(
        [0.25, 0.75, -0.125],
        [0.50, 1.50, 0.125],
        32.0, 48.0, 4.0, 12.0, 4.0,
        center,
        yaw_rad,
        0.0,
    );

    // 6. Right Leg (4x12x4, UV origin: 0, 16)
    mesh.add_box_textured(
        [-0.25, 0.0, -0.125],
        [0.0, 0.75, 0.125],
        0.0, 16.0, 4.0, 12.0, 4.0,
        center,
        yaw_rad,
        0.0,
    );

    // 7. Left Leg (4x12x4, UV origin: 16, 48)
    mesh.add_box_textured(
        [0.0, 0.0, -0.125],
        [0.25, 0.75, 0.125],
        16.0, 48.0, 4.0, 12.0, 4.0,
        center,
        yaw_rad,
        0.0,
    );

    mesh
}

pub struct WorldMeshManager {
    dirty_chunks: HashSet<(i32, i32)>,
    meshed_chunks: HashSet<(i32, i32)>,
}

impl WorldMeshManager {
    pub fn new() -> Self {
        Self {
            dirty_chunks: HashSet::new(),
            meshed_chunks: HashSet::new(),
        }
    }

    pub fn mark_dirty(&mut self, chunk_x: i32, chunk_z: i32) {
        self.dirty_chunks.insert((chunk_x, chunk_z));
        self.dirty_chunks.insert((chunk_x + 1, chunk_z));
        self.dirty_chunks.insert((chunk_x - 1, chunk_z));
        self.dirty_chunks.insert((chunk_x, chunk_z + 1));
        self.dirty_chunks.insert((chunk_x, chunk_z - 1));
    }

    #[allow(dead_code)]
    pub fn mark_chunk_dirty(&mut self, chunk_x: i32, chunk_z: i32) {
        self.dirty_chunks.insert((chunk_x, chunk_z));
    }

    #[allow(dead_code)]
    pub fn remove_chunk(&mut self, chunk_x: i32, chunk_z: i32) {
        self.dirty_chunks.remove(&(chunk_x, chunk_z));
        self.meshed_chunks.remove(&(chunk_x, chunk_z));
    }

    pub fn update(
        &mut self,
        world: &mut World,
        center_cx: i32,
        center_cz: i32,
        render_radius: i32,
        max_chunks_per_frame: usize,
    ) -> Vec<((i32, i32), MeshData)> {
        for dirty in world.take_dirty_chunks() {
            self.dirty_chunks.insert(dirty);
        }

        for cx in (center_cx - render_radius)..=(center_cx + render_radius) {
            for cz in (center_cz - render_radius)..=(center_cz + render_radius) {
                if !self.meshed_chunks.contains(&(cx, cz)) && world.get_chunk(cx, cz).is_some() {
                    self.dirty_chunks.insert((cx, cz));
                }
            }
        }

        let mut dirty_in_range: Vec<(i32, i32)> = self
            .dirty_chunks
            .iter()
            .copied()
            .filter(|&(cx, cz)| {
                (cx - center_cx).abs() <= render_radius && (cz - center_cz).abs() <= render_radius
            })
            .collect();

        dirty_in_range.sort_by_key(|&(cx, cz)| {
            let dx = cx - center_cx;
            let dz = cz - center_cz;
            dx * dx + dz * dz
        });

        let mut remeshed = Vec::new();
        let count = dirty_in_range.len().min(max_chunks_per_frame);

        for &(cx, cz) in &dirty_in_range[..count] {
            self.dirty_chunks.remove(&(cx, cz));
            if let Some(chunk) = world.get_chunk(cx, cz) {
                let mesh = mesh_chunk_column(chunk, world);
                self.meshed_chunks.insert((cx, cz));
                remeshed.push(((cx, cz), mesh));
            } else {
                self.meshed_chunks.remove(&(cx, cz));
                remeshed.push(((cx, cz), MeshData::new()));
            }
        }

        remeshed
    }
}

pub fn is_opaque_cube(id: u16) -> bool {
    match id {
        0 | 6 | 8 | 9 | 10 | 11 | 18 | 20 | 26 | 27 | 28 | 29 | 30 | 31 | 32 | 33 | 34 | 36 | 37 | 38
        | 39 | 40 | 44 | 50 | 51 | 52 | 53 | 54 | 55 | 59 | 60 | 63 | 64 | 65 | 66
        | 67 | 68 | 69 | 70 | 71 | 72 | 75 | 76 | 77 | 78 | 79 | 81 | 83 | 85 | 90
        | 95 | 96 | 101 | 102 | 104 | 105 | 106 | 107 | 108 | 109 | 111 | 113 | 114
        | 115 | 116 | 117 | 118 | 120 | 126 | 127 | 128 | 130 | 131 | 132 | 134 | 135
        | 136 | 140 | 141 | 142 | 143 | 144 | 145 | 146 | 147 | 148 | 149 | 150 | 151 | 154 | 156
        | 160 | 161 | 163 | 164 | 165 | 171 | 175 => false,
        _ => true,
    }
}

pub fn is_cross_plant(id: u16) -> bool {
    matches!(id, 6 | 31 | 32 | 37 | 38 | 39 | 40 | 83 | 175)
}

pub fn is_torch(id: u16) -> bool {
    matches!(id, 50 | 75 | 76)
}

pub fn is_flat_floor(id: u16) -> bool {
    matches!(id, 27 | 28 | 66 | 111 | 157 | 171)
}

pub fn is_door(id: u16) -> bool {
    matches!(id, 64 | 71)
}

pub fn is_pressure_plate(id: u16) -> bool {
    matches!(id, 70 | 72 | 147 | 148)
}

pub fn is_button(id: u16) -> bool {
    matches!(id, 77 | 143)
}

pub fn is_lever(id: u16) -> bool {
    id == 69
}

pub fn is_stairs(id: u16) -> bool {
    matches!(
        id,
        53 | 67 | 108 | 109 | 114 | 128 | 134 | 135 | 136 | 156 | 163 | 164
    )
}

pub fn is_slab(id: u16) -> bool {
    matches!(id, 44 | 126)
}

pub fn is_fence(id: u16) -> bool {
    matches!(id, 85 | 107 | 113)
}

pub fn is_pane(id: u16) -> bool {
    matches!(id, 101 | 102 | 160)
}

pub fn is_piston(id: u16) -> bool {
    matches!(id, 29 | 33 | 34 | 36)
}

pub fn get_piston_side_rot(meta: u8, face: BlockFace) -> u8 {
    let orientation = meta & 7;
    match orientation {
        0 => match face {
            BlockFace::Bottom | BlockFace::Top => 0,
            _ => 2,
        },
        1 => 0,
        2 => match face {
            BlockFace::Bottom | BlockFace::West => 1,
            BlockFace::Top | BlockFace::East => 3,
            _ => 0,
        },
        3 => match face {
            BlockFace::Top | BlockFace::East => 1,
            BlockFace::Bottom | BlockFace::West => 3,
            _ => 0,
        },
        4 => match face {
            BlockFace::North => 3,
            BlockFace::South => 1,
            _ => 0,
        },
        5 => match face {
            BlockFace::Top | BlockFace::Bottom => 2,
            BlockFace::North => 1,
            BlockFace::South => 3,
            _ => 0,
        },
        _ => 0,
    }
}

pub fn should_render_face(block_id: u16, neighbor_id: u16) -> bool {
    if neighbor_id == 0 {
        return true;
    }
    if (block_id == 8 || block_id == 9) && (neighbor_id == 8 || neighbor_id == 9 || neighbor_id == 79) {
        return false;
    }
    if (block_id == 10 || block_id == 11) && (neighbor_id == 10 || neighbor_id == 11) {
        return false;
    }
    if block_id == 20 && (neighbor_id == 20 || neighbor_id == 95) {
        return false;
    }
    if block_id == 79 && neighbor_id == 79 {
        return false;
    }
    if block_id == 95 && (neighbor_id == 95 || neighbor_id == 20) {
        return false;
    }
    !is_opaque_cube(neighbor_id)
}

fn face_shading_factor(face: BlockFace) -> f32 {
    match face {
        BlockFace::Top => 1.0,
        BlockFace::Bottom => 0.5,
        BlockFace::North | BlockFace::South => 0.8,
        BlockFace::West | BlockFace::East => 0.6,
    }
}

fn add_cross_quads(mesh: &mut MeshData, wx: i32, wy: i32, wz: i32, block: Block) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let (slot, tint) = crate::texture::get_block_slot(block.id, block.meta, BlockFace::North);
    let (u0, v0, u1, v1) = crate::texture::get_slot_uv(slot);

    let final_color = match tint {
        Some(t) => [t[0], t[1], t[2], 1.0],
        None => [1.0, 1.0, 1.0, 1.0],
    };

    let normal = [
        (block.sky_light as f32 / 15.0).clamp(0.0, 1.0),
        (block.block_light as f32 / 15.0).clamp(0.0, 1.0),
        1.0,
    ];

    let x0 = fx + 0.05;
    let x1 = fx + 0.95;
    let z0 = fz + 0.05;
    let z1 = fz + 0.95;
    let y0 = fy;
    let y1 = fy + 1.0;

    let planes = [
        // Diagonal 1: (x0, z0) to (x1, z1)
        [
            ([x0, y0, z0], [u0, v1]),
            ([x1, y0, z1], [u1, v1]),
            ([x1, y1, z1], [u1, v0]),
            ([x0, y1, z0], [u0, v0]),
        ],
        [
            ([x1, y0, z1], [u0, v1]),
            ([x0, y0, z0], [u1, v1]),
            ([x0, y1, z0], [u1, v0]),
            ([x1, y1, z1], [u0, v0]),
        ],
        // Diagonal 2: (x0, z1) to (x1, z0)
        [
            ([x0, y0, z1], [u0, v1]),
            ([x1, y0, z0], [u1, v1]),
            ([x1, y1, z0], [u1, v0]),
            ([x0, y1, z1], [u0, v0]),
        ],
        [
            ([x1, y0, z0], [u0, v1]),
            ([x0, y0, z1], [u1, v1]),
            ([x0, y1, z1], [u1, v0]),
            ([x1, y1, z0], [u0, v0]),
        ],
    ];

    for quad in planes {
        let start_idx = mesh.vertices.len() as u32;
        for (pos, uv) in quad {
            mesh.vertices.push(Vertex {
                position: pos,
                uv,
                color: final_color,
                normal,
            });
        }
        mesh.indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    }
}

fn add_torch_quads(mesh: &mut MeshData, wx: i32, wy: i32, wz: i32, block: Block) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let (slot, _) = crate::texture::get_block_slot(block.id, block.meta, BlockFace::North);
    let (u0, v0, u1, v1) = crate::texture::get_slot_uv(slot);
    let du = u1 - u0;
    let dv = v1 - v0;

    let is_redstone = block.id == 75 || block.id == 76;
    let u_cap_min = u0 + (7.0 / 16.0) * du;
    let u_cap_max = u0 + (9.0 / 16.0) * du;
    let v_top_min = if is_redstone {
        v0 + (5.0 / 16.0) * dv
    } else {
        v0 + (6.0 / 16.0) * dv
    };
    let v_top_max = if is_redstone {
        v0 + (7.0 / 16.0) * dv
    } else {
        v0 + (8.0 / 16.0) * dv
    };
    let v_bot_min = v0 + (13.0 / 16.0) * dv;
    let v_bot_max = v0 + (15.0 / 16.0) * dv;

    let light = [1.0f32, 1.0, 1.0];

    // Torch local to world transform
    // Meta in 1.7.10: 1: West wall (+X), 2: East wall (-X), 3: North wall (+Z), 4: South wall (-Z), 5/other: Floor
    let transform = |p: [f32; 3]| -> [f32; 3] {
        let (x, y, z) = (p[0], p[1], p[2]);
        let sin_22_5: f32 = 0.38268343;
        let cos_22_5: f32 = 0.9238795;
        match block.meta {
            1 => {
                let p0 = [fx, fy + 3.5 / 16.0, fz + 0.5];
                let rx = x * cos_22_5 + y * sin_22_5;
                let ry = -x * sin_22_5 + y * cos_22_5;
                [p0[0] + rx, p0[1] + ry, p0[2] + z]
            }
            2 => {
                let p0 = [fx + 1.0, fy + 3.5 / 16.0, fz + 0.5];
                let rx = x * cos_22_5 - y * sin_22_5;
                let ry = x * sin_22_5 + y * cos_22_5;
                [p0[0] + rx, p0[1] + ry, p0[2] + z]
            }
            3 => {
                let p0 = [fx + 0.5, fy + 3.5 / 16.0, fz];
                let rz = z * cos_22_5 + y * sin_22_5;
                let ry = -z * sin_22_5 + y * cos_22_5;
                [p0[0] + x, p0[1] + ry, p0[2] + rz]
            }
            4 => {
                let p0 = [fx + 0.5, fy + 3.5 / 16.0, fz + 1.0];
                let rz = z * cos_22_5 - y * sin_22_5;
                let ry = z * sin_22_5 + y * cos_22_5;
                [p0[0] + x, p0[1] + ry, p0[2] + rz]
            }
            _ => {
                [fx + 0.5 + x, fy + y, fz + 0.5 + z]
            }
        }
    };

    let w = 1.0 / 16.0;
    let faces: [([ [f32; 3]; 4], [[f32; 2]; 4], f32); 6] = [
        // 1. Top cap (+Y): 2x2 stick top at height 10/16
        (
            [
                [-w, 10.0 / 16.0, w],
                [w, 10.0 / 16.0, w],
                [w, 10.0 / 16.0, -w],
                [-w, 10.0 / 16.0, -w],
            ],
            [
                [u_cap_min, v_top_max],
                [u_cap_max, v_top_max],
                [u_cap_max, v_top_min],
                [u_cap_min, v_top_min],
            ],
            1.0f32,
        ),
        // 2. North face (-Z) of Slab 2: full 16x16 sprite along X and Y
        (
            [
                [0.5, 1.0, -w],
                [0.5, 0.0, -w],
                [-0.5, 0.0, -w],
                [-0.5, 1.0, -w],
            ],
            [
                [u0, v0],
                [u0, v1],
                [u1, v1],
                [u1, v0],
            ],
            0.8f32,
        ),
        // 3. South face (+Z) of Slab 2: full 16x16 sprite along X and Y
        (
            [
                [-0.5, 1.0, w],
                [-0.5, 0.0, w],
                [0.5, 0.0, w],
                [0.5, 1.0, w],
            ],
            [
                [u0, v0],
                [u0, v1],
                [u1, v1],
                [u1, v0],
            ],
            0.8f32,
        ),
        // 4. West face (-X) of Slab 1: full 16x16 sprite along Z and Y
        (
            [
                [-w, 1.0, -0.5],
                [-w, 0.0, -0.5],
                [-w, 0.0, 0.5],
                [-w, 1.0, 0.5],
            ],
            [
                [u0, v0],
                [u0, v1],
                [u1, v1],
                [u1, v0],
            ],
            0.8f32,
        ),
        // 5. East face (+X) of Slab 1: full 16x16 sprite along Z and Y
        (
            [
                [w, 1.0, 0.5],
                [w, 0.0, 0.5],
                [w, 0.0, -0.5],
                [w, 1.0, -0.5],
            ],
            [
                [u0, v0],
                [u0, v1],
                [u1, v1],
                [u1, v0],
            ],
            0.8f32,
        ),
        // 6. Bottom cap (-Y): 2x2 stick bottom at height 0
        (
            [
                [-w, 0.0, -w],
                [w, 0.0, -w],
                [w, 0.0, w],
                [-w, 0.0, w],
            ],
            [
                [u_cap_min, v_bot_min],
                [u_cap_max, v_bot_min],
                [u_cap_max, v_bot_max],
                [u_cap_min, v_bot_max],
            ],
            0.5f32,
        ),
    ];

    for (quad, uvs, shade) in faces {
        let col = [shade, shade, shade, 1.0];
        let start_idx = mesh.vertices.len() as u32;
        for (i, &pos) in quad.iter().enumerate() {
            mesh.vertices.push(Vertex {
                position: transform(pos),
                uv: uvs[i],
                color: col,
                normal: light,
            });
        }
        mesh.indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    }
}

fn is_redstone_connectable(id: u16) -> bool {
    matches!(
        id,
        55
        | 69
        | 70 | 72 | 147 | 148
        | 75 | 76
        | 77 | 143
        | 93 | 94
        | 146
        | 149 | 150
        | 151
        | 152
    )
}

fn redstone_connects_to(world: &World, wx: i32, wy: i32, wz: i32, dx: i32, dz: i32) -> bool {
    let same_id = world.get_block(wx + dx, wy, wz + dz).id;
    if is_redstone_connectable(same_id) {
        return true;
    }
    if !is_opaque_cube(same_id) {
        let below_id = world.get_block(wx + dx, wy - 1, wz + dz).id;
        if below_id == 55 {
            return true;
        }
    }
    let above_id = world.get_block(wx, wy + 1, wz).id;
    if !is_opaque_cube(above_id) {
        let up_id = world.get_block(wx + dx, wy + 1, wz + dz).id;
        if up_id == 55 {
            return true;
        }
    }
    false
}

fn add_door_quads(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let is_upper = (block.meta & 8) != 0;
    let (facing, is_open, hinge_right) = if is_upper {
        let lower = world.get_block(wx, wy - 1, wz);
        let (f, o) = if lower.id == block.id {
            (lower.meta & 3, (lower.meta & 4) != 0)
        } else {
            (0, false)
        };
        let hr = (block.meta & 1) == 1;
        (f, o, hr)
    } else {
        let f = block.meta & 3;
        let o = (block.meta & 4) != 0;
        let upper = world.get_block(wx, wy + 1, wz);
        let hr = if upper.id == block.id {
            (upper.meta & 1) == 1
        } else {
            false
        };
        (f, o, hr)
    };

    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;
    let t = 3.0 / 16.0;

    let (rx0, rx1, rz0, rz1) = match (facing, is_open, hinge_right) {
        (0, false, _) => (0.0, t, 0.0, 1.0),
        (0, true, true) => (0.0, 1.0, 1.0 - t, 1.0),
        (0, true, false) => (0.0, 1.0, 0.0, t),

        (1, false, _) => (0.0, 1.0, 0.0, t),
        (1, true, true) => (0.0, t, 0.0, 1.0),
        (1, true, false) => (1.0 - t, 1.0, 0.0, 1.0),

        (2, false, _) => (1.0 - t, 1.0, 0.0, 1.0),
        (2, true, true) => (0.0, 1.0, 0.0, t),
        (2, true, false) => (0.0, 1.0, 1.0 - t, 1.0),

        (3, false, _) => (0.0, 1.0, 1.0 - t, 1.0),
        (3, true, true) => (1.0 - t, 1.0, 0.0, 1.0),
        (3, true, false) => (0.0, t, 0.0, 1.0),

        _ => (0.0, t, 0.0, 1.0),
    };

    let p_min_x = fx + rx0;
    let p_max_x = fx + rx1;
    let p_min_y = fy;
    let p_max_y = fy + 1.0;
    let p_min_z = fz + rz0;
    let p_max_z = fz + rz1;

    let is_iron = block.id == 71;
    let main_slot = if is_iron {
        if is_upper { 118 } else { 119 }
    } else {
        if is_upper { 116 } else { 117 }
    };

    let (u0_m, v0_m, u1_m, v1_m) = crate::texture::get_slot_uv(main_slot);
    let du_m = u1_m - u0_m;
    let dv_m = v1_m - v0_m;

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    let is_thick_x = (p_max_x - p_min_x) < 0.5;

    let hinge_at_min = if is_thick_x {
        // Width is along Z
        match (facing, is_open) {
            (0, false) => !hinge_right,
            (2, false) => hinge_right,
            (1, true) => true,
            (3, true) => false,
            _ => !hinge_right,
        }
    } else {
        // Width is along X
        match (facing, is_open) {
            (1, false) => hinge_right,
            (3, false) => !hinge_right,
            (0, true) => true,
            (2, true) => false,
            _ => hinge_right,
        }
    };

    // In door_wood / door_iron textures, handle is at u0 (columns 0..2) and hinge is at u1 (columns 13..15).
    let u_hinge = u1_m;
    let u_handle = u0_m;

    let (u_at_min, u_at_max) = if hinge_at_min {
        (u_hinge, u_handle)
    } else {
        (u_handle, u_hinge)
    };

    let (u0_edge_min, u1_edge_min) = if hinge_at_min {
        (u1_m - (3.0 / 16.0) * du_m, u1_m)
    } else {
        (u0_m, u0_m + (3.0 / 16.0) * du_m)
    };

    let (u0_edge_max, u1_edge_max) = if hinge_at_min {
        (u0_m, u0_m + (3.0 / 16.0) * du_m)
    } else {
        (u1_m - (3.0 / 16.0) * du_m, u1_m)
    };

    let v_top_edge_bot = v0_m + (3.0 / 16.0) * dv_m;
    let v_bot_edge_top = v1_m - (3.0 / 16.0) * dv_m;

    let faces = [
        // Top edge (+Y)
        (
            [
                [p_min_x, p_max_y, p_max_z],
                [p_max_x, p_max_y, p_max_z],
                [p_max_x, p_max_y, p_min_z],
                [p_min_x, p_max_y, p_min_z],
            ],
            if is_thick_x {
                [
                    [u_at_max, v0_m],
                    [u_at_max, v_top_edge_bot],
                    [u_at_min, v_top_edge_bot],
                    [u_at_min, v0_m],
                ]
            } else {
                [
                    [u_at_min, v_top_edge_bot],
                    [u_at_max, v_top_edge_bot],
                    [u_at_max, v0_m],
                    [u_at_min, v0_m],
                ]
            },
            1.0f32,
        ),
        // Bottom edge (-Y)
        (
            [
                [p_min_x, p_min_y, p_min_z],
                [p_max_x, p_min_y, p_min_z],
                [p_max_x, p_min_y, p_max_z],
                [p_min_x, p_min_y, p_max_z],
            ],
            if is_thick_x {
                [
                    [u_at_min, v_bot_edge_top],
                    [u_at_min, v1_m],
                    [u_at_max, v1_m],
                    [u_at_max, v_bot_edge_top],
                ]
            } else {
                [
                    [u_at_min, v_bot_edge_top],
                    [u_at_max, v_bot_edge_top],
                    [u_at_max, v1_m],
                    [u_at_min, v1_m],
                ]
            },
            0.5f32,
        ),
        // North face (-Z)
        (
            [
                [p_max_x, p_max_y, p_min_z],
                [p_max_x, p_min_y, p_min_z],
                [p_min_x, p_min_y, p_min_z],
                [p_min_x, p_max_y, p_min_z],
            ],
            if is_thick_x {
                [[u0_edge_min, v0_m], [u0_edge_min, v1_m], [u1_edge_min, v1_m], [u1_edge_min, v0_m]]
            } else {
                [[u_at_max, v0_m], [u_at_max, v1_m], [u_at_min, v1_m], [u_at_min, v0_m]]
            },
            0.8f32,
        ),
        // South face (+Z)
        (
            [
                [p_min_x, p_max_y, p_max_z],
                [p_min_x, p_min_y, p_max_z],
                [p_max_x, p_min_y, p_max_z],
                [p_max_x, p_max_y, p_max_z],
            ],
            if is_thick_x {
                [[u0_edge_max, v0_m], [u0_edge_max, v1_m], [u1_edge_max, v1_m], [u1_edge_max, v0_m]]
            } else {
                [[u_at_min, v0_m], [u_at_min, v1_m], [u_at_max, v1_m], [u_at_max, v0_m]]
            },
            0.8f32,
        ),
        // West face (-X)
        (
            [
                [p_min_x, p_max_y, p_min_z],
                [p_min_x, p_min_y, p_min_z],
                [p_min_x, p_min_y, p_max_z],
                [p_min_x, p_max_y, p_max_z],
            ],
            if is_thick_x {
                [[u_at_min, v0_m], [u_at_min, v1_m], [u_at_max, v1_m], [u_at_max, v0_m]]
            } else {
                [[u0_edge_min, v0_m], [u0_edge_min, v1_m], [u1_edge_min, v1_m], [u1_edge_min, v0_m]]
            },
            0.6f32,
        ),
        // East face (+X)
        (
            [
                [p_max_x, p_max_y, p_max_z],
                [p_max_x, p_min_y, p_max_z],
                [p_max_x, p_min_y, p_min_z],
                [p_max_x, p_max_y, p_min_z],
            ],
            if is_thick_x {
                [[u_at_max, v0_m], [u_at_max, v1_m], [u_at_min, v1_m], [u_at_min, v0_m]]
            } else {
                [[u0_edge_max, v0_m], [u0_edge_max, v1_m], [u1_edge_max, v1_m], [u1_edge_max, v0_m]]
            },
            0.6f32,
        ),
    ];

    for (quad, uvs, shade) in faces {
        let col = [shade, shade, shade, 1.0];
        let start_idx = mesh.vertices.len() as u32;
        for (i, &pos) in quad.iter().enumerate() {
            mesh.vertices.push(Vertex {
                position: pos,
                uv: uvs[i],
                color: col,
                normal: light,
            });
        }
        mesh.indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    }
}

fn add_bed_quads(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let is_head = (block.meta & 8) != 0;
    let facing = block.meta & 3;

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    let x0 = fx;
    let x1 = fx + 1.0;
    let y0 = fy;
    let y1 = fy + 9.0 / 16.0;
    let z0 = fz;
    let z1 = fz + 1.0;

    let mut add_quad_helper = |quad: [[f32; 3]; 4], uvs: [[f32; 2]; 4], shade: f32| {
        let col = [shade, shade, shade, 1.0];
        let start_idx = mesh.vertices.len() as u32;
        for (i, &pos) in quad.iter().enumerate() {
            mesh.vertices.push(Vertex {
                position: pos,
                uv: uvs[i],
                color: col,
                normal: light,
            });
        }
        mesh.indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    };

    // 1. Top face (+Y)
    let top_slot = if is_head { 198 } else { 195 };
    let (u0_t, v0_t, u1_t, v1_t) = crate::texture::get_slot_uv(top_slot);
    let top_uvs = match facing {
        // South: Head is +Z (z1 has pillow/sheet = u1_t, z0 has blanket = u0_t)
        0 => [
            [u1_t, v0_t],
            [u1_t, v1_t],
            [u0_t, v1_t],
            [u0_t, v0_t],
        ],
        // West: Head is -X (x0 has pillow/sheet = u1_t, x1 has blanket = u0_t)
        1 => [
            [u1_t, v1_t],
            [u0_t, v1_t],
            [u0_t, v0_t],
            [u1_t, v0_t],
        ],
        // North: Head is -Z (z0 has pillow/sheet = u1_t, z1 has blanket = u0_t)
        2 => [
            [u0_t, v1_t],
            [u0_t, v0_t],
            [u1_t, v0_t],
            [u1_t, v1_t],
        ],
        // East: Head is +X (x1 has pillow/sheet = u1_t, x0 has blanket = u0_t)
        _ => [
            [u0_t, v0_t],
            [u1_t, v0_t],
            [u1_t, v1_t],
            [u0_t, v1_t],
        ],
    };
    add_quad_helper([[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]], top_uvs, 1.0);

    // 2. Bottom face (-Y)
    let below_id = world.get_block(wx, wy - 1, wz).id;
    if !is_opaque_cube(below_id) {
        let (u0_b, v0_b, u1_b, v1_b) = crate::texture::get_slot_uv(6);
        add_quad_helper(
            [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]],
            [[u0_b, v0_b], [u1_b, v0_b], [u1_b, v1_b], [u0_b, v1_b]],
            0.5,
        );
    }

    let end_slot = if is_head { 196 } else { 193 };
    let (u0_e, v0_e, u1_e, v1_e) = crate::texture::get_slot_uv(end_slot);
    let v_top_e = v0_e + (7.0 / 16.0) * (v1_e - v0_e);
    let v_bot_e = v1_e;
    let end_uvs = [[u0_e, v_top_e], [u0_e, v_bot_e], [u1_e, v_bot_e], [u1_e, v_top_e]];

    let side_slot = if is_head { 197 } else { 194 };
    let (u0_s, v0_s, u1_s, v1_s) = crate::texture::get_slot_uv(side_slot);
    let v_top_s = v0_s + (7.0 / 16.0) * (v1_s - v0_s);
    let v_bot_s = v1_s;
    let side_uvs_head_first = [[u1_s, v_top_s], [u1_s, v_bot_s], [u0_s, v_bot_s], [u0_s, v_top_s]];
    let side_uvs_foot_first = [[u0_s, v_top_s], [u0_s, v_bot_s], [u1_s, v_bot_s], [u1_s, v_top_s]];

    // 3. North face (-Z)
    let neighbor_n = world.get_block(wx, wy, wz - 1).id;
    match facing {
        2 => {
            if is_head {
                if !is_opaque_cube(neighbor_n) {
                    add_quad_helper([[x1, y1, z0], [x1, y0, z0], [x0, y0, z0], [x0, y1, z0]], end_uvs, 0.8);
                }
            } else if neighbor_n != 26 && !is_opaque_cube(neighbor_n) {
                add_quad_helper([[x1, y1, z0], [x1, y0, z0], [x0, y0, z0], [x0, y1, z0]], end_uvs, 0.8);
            }
        }
        0 => {
            if !is_head {
                if !is_opaque_cube(neighbor_n) {
                    add_quad_helper([[x1, y1, z0], [x1, y0, z0], [x0, y0, z0], [x0, y1, z0]], end_uvs, 0.8);
                }
            } else if neighbor_n != 26 && !is_opaque_cube(neighbor_n) {
                add_quad_helper([[x1, y1, z0], [x1, y0, z0], [x0, y0, z0], [x0, y1, z0]], end_uvs, 0.8);
            }
        }
        1 => {
            if !is_opaque_cube(neighbor_n) {
                add_quad_helper([[x1, y1, z0], [x1, y0, z0], [x0, y0, z0], [x0, y1, z0]], side_uvs_foot_first, 0.8);
            }
        }
        _ => {
            if !is_opaque_cube(neighbor_n) {
                add_quad_helper([[x1, y1, z0], [x1, y0, z0], [x0, y0, z0], [x0, y1, z0]], side_uvs_head_first, 0.8);
            }
        }
    }

    // 4. South face (+Z)
    let neighbor_s = world.get_block(wx, wy, wz + 1).id;
    match facing {
        0 => {
            if is_head {
                if !is_opaque_cube(neighbor_s) {
                    add_quad_helper([[x0, y1, z1], [x0, y0, z1], [x1, y0, z1], [x1, y1, z1]], end_uvs, 0.8);
                }
            } else if neighbor_s != 26 && !is_opaque_cube(neighbor_s) {
                add_quad_helper([[x0, y1, z1], [x0, y0, z1], [x1, y0, z1], [x1, y1, z1]], end_uvs, 0.8);
            }
        }
        2 => {
            if !is_head {
                if !is_opaque_cube(neighbor_s) {
                    add_quad_helper([[x0, y1, z1], [x0, y0, z1], [x1, y0, z1], [x1, y1, z1]], end_uvs, 0.8);
                }
            } else if neighbor_s != 26 && !is_opaque_cube(neighbor_s) {
                add_quad_helper([[x0, y1, z1], [x0, y0, z1], [x1, y0, z1], [x1, y1, z1]], end_uvs, 0.8);
            }
        }
        1 => {
            if !is_opaque_cube(neighbor_s) {
                add_quad_helper([[x0, y1, z1], [x0, y0, z1], [x1, y0, z1], [x1, y1, z1]], side_uvs_head_first, 0.8);
            }
        }
        _ => {
            if !is_opaque_cube(neighbor_s) {
                add_quad_helper([[x0, y1, z1], [x0, y0, z1], [x1, y0, z1], [x1, y1, z1]], side_uvs_foot_first, 0.8);
            }
        }
    }

    // 5. West face (-X)
    let neighbor_w = world.get_block(wx - 1, wy, wz).id;
    match facing {
        1 => {
            if is_head {
                if !is_opaque_cube(neighbor_w) {
                    add_quad_helper([[x0, y1, z0], [x0, y0, z0], [x0, y0, z1], [x0, y1, z1]], end_uvs, 0.6);
                }
            } else if neighbor_w != 26 && !is_opaque_cube(neighbor_w) {
                add_quad_helper([[x0, y1, z0], [x0, y0, z0], [x0, y0, z1], [x0, y1, z1]], end_uvs, 0.6);
            }
        }
        3 => {
            if !is_head {
                if !is_opaque_cube(neighbor_w) {
                    add_quad_helper([[x0, y1, z0], [x0, y0, z0], [x0, y0, z1], [x0, y1, z1]], end_uvs, 0.6);
                }
            } else if neighbor_w != 26 && !is_opaque_cube(neighbor_w) {
                add_quad_helper([[x0, y1, z0], [x0, y0, z0], [x0, y0, z1], [x0, y1, z1]], end_uvs, 0.6);
            }
        }
        0 => {
            if !is_opaque_cube(neighbor_w) {
                add_quad_helper([[x0, y1, z0], [x0, y0, z0], [x0, y0, z1], [x0, y1, z1]], side_uvs_foot_first, 0.6);
            }
        }
        _ => {
            if !is_opaque_cube(neighbor_w) {
                add_quad_helper([[x0, y1, z0], [x0, y0, z0], [x0, y0, z1], [x0, y1, z1]], side_uvs_head_first, 0.6);
            }
        }
    }

    // 6. East face (+X)
    let neighbor_e = world.get_block(wx + 1, wy, wz).id;
    match facing {
        3 => {
            if is_head {
                if !is_opaque_cube(neighbor_e) {
                    add_quad_helper([[x1, y1, z1], [x1, y0, z1], [x1, y0, z0], [x1, y1, z0]], end_uvs, 0.6);
                }
            } else if neighbor_e != 26 && !is_opaque_cube(neighbor_e) {
                add_quad_helper([[x1, y1, z1], [x1, y0, z1], [x1, y0, z0], [x1, y1, z0]], end_uvs, 0.6);
            }
        }
        1 => {
            if !is_head {
                if !is_opaque_cube(neighbor_e) {
                    add_quad_helper([[x1, y1, z1], [x1, y0, z1], [x1, y0, z0], [x1, y1, z0]], end_uvs, 0.6);
                }
            } else if neighbor_e != 26 && !is_opaque_cube(neighbor_e) {
                add_quad_helper([[x1, y1, z1], [x1, y0, z1], [x1, y0, z0], [x1, y1, z0]], end_uvs, 0.6);
            }
        }
        0 => {
            if !is_opaque_cube(neighbor_e) {
                add_quad_helper([[x1, y1, z1], [x1, y0, z1], [x1, y0, z0], [x1, y1, z0]], side_uvs_head_first, 0.6);
            }
        }
        _ => {
            if !is_opaque_cube(neighbor_e) {
                add_quad_helper([[x1, y1, z1], [x1, y0, z1], [x1, y0, z0], [x1, y1, z0]], side_uvs_foot_first, 0.6);
            }
        }
    }
}

fn add_redstone_wire_quads(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let p = (block.meta.min(15) as f32) / 15.0;
    let red_tint = [0.3 + 0.7 * p, 0.0, 0.0, 1.0];

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    let west = redstone_connects_to(world, wx, wy, wz, -1, 0);
    let east = redstone_connects_to(world, wx, wy, wz, 1, 0);
    let north = redstone_connects_to(world, wx, wy, wz, 0, -1);
    let south = redstone_connects_to(world, wx, wy, wz, 0, 1);

    let is_ew = (west || east) && !north && !south;
    let is_ns = (north || south) && !west && !east;

    let (_slot, uvs_top) = if is_ew {
        let (u0, v0, u1, v1) = crate::texture::get_slot_uv(175);
        (
            175,
            [
                [u0, v1],
                [u1, v1],
                [u1, v0],
                [u0, v0],
            ],
        )
    } else if is_ns {
        let (u0, v0, u1, v1) = crate::texture::get_slot_uv(175);
        (
            175,
            [
                [u1, v0],
                [u1, v1],
                [u0, v1],
                [u0, v0],
            ],
        )
    } else {
        let (u0, v0, u1, v1) = crate::texture::get_slot_uv(101);
        (
            101,
            [
                [u0, v1],
                [u1, v1],
                [u1, v0],
                [u0, v0],
            ],
        )
    };

    let h = 0.02f32;
    let floor_quad = [
        [fx, fy + h, fz + 1.0],
        [fx + 1.0, fy + h, fz + 1.0],
        [fx + 1.0, fy + h, fz],
        [fx, fy + h, fz],
    ];

    let start_idx = mesh.vertices.len() as u32;
    for (i, &pos) in floor_quad.iter().enumerate() {
        mesh.vertices.push(Vertex {
            position: pos,
            uv: uvs_top[i],
            color: red_tint,
            normal: light,
        });
    }
    mesh.indices.extend_from_slice(&[
        start_idx,
        start_idx + 1,
        start_idx + 2,
        start_idx,
        start_idx + 2,
        start_idx + 3,
    ]);

    let above_open = !is_opaque_cube(above.id);
    if above_open {
        let (u0_l, v0_l, u1_l, v1_l) = crate::texture::get_slot_uv(175);
        let dirs = [
            (1, 0, [
                [fx + 0.998, fy + 1.0, fz],
                [fx + 0.998, fy, fz],
                [fx + 0.998, fy, fz + 1.0],
                [fx + 0.998, fy + 1.0, fz + 1.0],
            ], 0.6f32),
            (-1, 0, [
                [fx + 0.002, fy + 1.0, fz + 1.0],
                [fx + 0.002, fy, fz + 1.0],
                [fx + 0.002, fy, fz],
                [fx + 0.002, fy + 1.0, fz],
            ], 0.6f32),
            (0, 1, [
                [fx + 1.0, fy + 1.0, fz + 0.998],
                [fx + 1.0, fy, fz + 0.998],
                [fx, fy, fz + 0.998],
                [fx, fy + 1.0, fz + 0.998],
            ], 0.8f32),
            (0, -1, [
                [fx, fy + 1.0, fz + 0.002],
                [fx, fy, fz + 0.002],
                [fx + 1.0, fy, fz + 0.002],
                [fx + 1.0, fy + 1.0, fz + 0.002],
            ], 0.8f32),
        ];

        let climb_uvs = [
            [u1_l, v0_l],
            [u0_l, v0_l],
            [u0_l, v1_l],
            [u1_l, v1_l],
        ];

        for (dx, dz, quad, shade) in dirs {
            let step_id = world.get_block(wx + dx, wy, wz + dz).id;
            let up_id = world.get_block(wx + dx, wy + 1, wz + dz).id;
            if is_opaque_cube(step_id) && up_id == 55 {
                let col = [red_tint[0] * shade, red_tint[1] * shade, red_tint[2] * shade, 1.0];
                let s_idx = mesh.vertices.len() as u32;
                for (i, &pos) in quad.iter().enumerate() {
                    mesh.vertices.push(Vertex {
                        position: pos,
                        uv: climb_uvs[i],
                        color: col,
                        normal: light,
                    });
                }
                mesh.indices.extend_from_slice(&[
                    s_idx,
                    s_idx + 1,
                    s_idx + 2,
                    s_idx,
                    s_idx + 2,
                    s_idx + 3,
                ]);
            }
        }
    }
}

fn add_sub_box_with_uv(
    mesh: &mut MeshData,
    min: [f32; 3],
    max: [f32; 3],
    slots: [u8; 6], // Top, Bottom, North, South, West, East
    rots: [u8; 6],
    light: [f32; 3],
    custom_uvs: Option<[[[f32; 2]; 4]; 6]>,
    visible_mask: Option<[bool; 6]>,
) {
    let [x0, y0, z0] = min;
    let [x1, y1, z1] = max;

    let faces = [
        (
            [
                [x0, y1, z1],
                [x1, y1, z1],
                [x1, y1, z0],
                [x0, y1, z0],
            ],
            BlockFace::Top,
            slots[0],
            rots[0],
            1.0f32,
        ),
        (
            [
                [x0, y0, z0],
                [x1, y0, z0],
                [x1, y0, z1],
                [x0, y0, z1],
            ],
            BlockFace::Bottom,
            slots[1],
            rots[1],
            0.5f32,
        ),
        (
            [
                [x1, y1, z0],
                [x1, y0, z0],
                [x0, y0, z0],
                [x0, y1, z0],
            ],
            BlockFace::North,
            slots[2],
            rots[2],
            0.8f32,
        ),
        (
            [
                [x0, y1, z1],
                [x0, y0, z1],
                [x1, y0, z1],
                [x1, y1, z1],
            ],
            BlockFace::South,
            slots[3],
            rots[3],
            0.8f32,
        ),
        (
            [
                [x0, y1, z0],
                [x0, y0, z0],
                [x0, y0, z1],
                [x0, y1, z1],
            ],
            BlockFace::West,
            slots[4],
            rots[4],
            0.6f32,
        ),
        (
            [
                [x1, y1, z1],
                [x1, y0, z1],
                [x1, y0, z0],
                [x1, y1, z0],
            ],
            BlockFace::East,
            slots[5],
            rots[5],
            0.6f32,
        ),
    ];

    for (face_idx, (quad, face, slot, rot, shade)) in faces.into_iter().enumerate() {
        if let Some(mask) = visible_mask {
            if !mask[face_idx] {
                continue;
            }
        }

        let uvs = if let Some(ref c) = custom_uvs {
            c[face_idx]
        } else {
            let (u0, v0, u1, v1) = crate::texture::get_slot_uv(slot);
            if rot == 0 {
                match face {
                    BlockFace::Top => [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
                    BlockFace::Bottom => [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
                    _ => [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
                }
            } else {
                get_rotated_quad_uvs(u0, v0, u1, v1, rot)
            }
        };

        let col = [shade, shade, shade, 1.0];
        let start_idx = mesh.vertices.len() as u32;
        for (i, &pos) in quad.iter().enumerate() {
            mesh.vertices.push(Vertex {
                position: pos,
                uv: uvs[i],
                color: col,
                normal: light,
            });
        }
        mesh.indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    }
}

#[allow(dead_code)]
fn add_sub_box(
    mesh: &mut MeshData,
    min: [f32; 3],
    max: [f32; 3],
    slots: [u8; 6],
    rots: [u8; 6],
    light: [f32; 3],
) {
    add_sub_box_with_uv(mesh, min, max, slots, rots, light, None, None);
}

fn add_sub_box_proportional(
    mesh: &mut MeshData,
    min: [f32; 3],
    max: [f32; 3],
    block_origin: [f32; 3],
    slots: [u8; 6],
    light: [f32; 3],
) {
    let fx = block_origin[0];
    let fy = block_origin[1];
    let fz = block_origin[2];

    let x0 = (min[0] - fx).clamp(0.0, 1.0);
    let x1 = (max[0] - fx).clamp(0.0, 1.0);
    let y0 = (min[1] - fy).clamp(0.0, 1.0);
    let y1 = (max[1] - fy).clamp(0.0, 1.0);
    let z0 = (min[2] - fz).clamp(0.0, 1.0);
    let z1 = (max[2] - fz).clamp(0.0, 1.0);

    let mut custom = [[[0.0f32; 2]; 4]; 6];
    for face_idx in 0..6 {
        let slot = slots[face_idx];
        let (u0, v0, u1, v1) = crate::texture::get_slot_uv(slot);
        let du = u1 - u0;
        let dv = v1 - v0;

        let uvs = match face_idx {
            // Top (+Y): quad is [x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]
            0 => [
                [u0 + du * x0, v0 + dv * z1],
                [u0 + du * x1, v0 + dv * z1],
                [u0 + du * x1, v0 + dv * z0],
                [u0 + du * x0, v0 + dv * z0],
            ],
            // Bottom (-Y): quad is [x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]
            1 => [
                [u0 + du * x0, v0 + dv * z0],
                [u0 + du * x1, v0 + dv * z0],
                [u0 + du * x1, v0 + dv * z1],
                [u0 + du * x0, v0 + dv * z1],
            ],
            // North (-Z): quad is [x1, y1, z0], [x1, y0, z0], [x0, y0, z0], [x0, y1, z0]
            2 => [
                [u0 + du * (1.0 - x1), v0 + dv * (1.0 - y1)],
                [u0 + du * (1.0 - x1), v0 + dv * (1.0 - y0)],
                [u0 + du * (1.0 - x0), v0 + dv * (1.0 - y0)],
                [u0 + du * (1.0 - x0), v0 + dv * (1.0 - y1)],
            ],
            // South (+Z): quad is [x0, y1, z1], [x0, y0, z1], [x1, y0, z1], [x1, y1, z1]
            3 => [
                [u0 + du * x0, v0 + dv * (1.0 - y1)],
                [u0 + du * x0, v0 + dv * (1.0 - y0)],
                [u0 + du * x1, v0 + dv * (1.0 - y0)],
                [u0 + du * x1, v0 + dv * (1.0 - y1)],
            ],
            // West (-X): quad is [x0, y1, z0], [x0, y0, z0], [x0, y0, z1], [x0, y1, z1]
            4 => [
                [u0 + du * z0, v0 + dv * (1.0 - y1)],
                [u0 + du * z0, v0 + dv * (1.0 - y0)],
                [u0 + du * z1, v0 + dv * (1.0 - y0)],
                [u0 + du * z1, v0 + dv * (1.0 - y1)],
            ],
            // East (+X): quad is [x1, y1, z1], [x1, y0, z1], [x1, y0, z0], [x1, y1, z0]
            _ => [
                [u0 + du * (1.0 - z1), v0 + dv * (1.0 - y1)],
                [u0 + du * (1.0 - z1), v0 + dv * (1.0 - y0)],
                [u0 + du * (1.0 - z0), v0 + dv * (1.0 - y0)],
                [u0 + du * (1.0 - z0), v0 + dv * (1.0 - y1)],
            ],
        };
        custom[face_idx] = uvs;
    }
    add_sub_box_with_uv(mesh, min, max, slots, [0; 6], light, Some(custom), None);
}

fn get_piston_dir(ori: u8) -> (i32, i32, i32) {
    match ori & 7 {
        0 => (0, -1, 0), // Down
        1 => (0, 1, 0),  // Up
        2 => (0, 0, -1), // North
        3 => (0, 0, 1),  // South
        4 => (-1, 0, 0), // West
        _ => (1, 0, 0),  // East
    }
}

fn add_extended_piston_base(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;
    let ori = block.meta & 7;

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    let (min, max) = match ori {
        0 => ([fx, fy + 0.25, fz], [fx + 1.0, fy + 1.0, fz + 1.0]),
        1 => ([fx, fy, fz], [fx + 1.0, fy + 0.75, fz + 1.0]),
        2 => ([fx, fy, fz + 0.25], [fx + 1.0, fy + 1.0, fz + 1.0]),
        3 => ([fx, fy, fz], [fx + 1.0, fy + 1.0, fz + 0.75]),
        4 => ([fx + 0.25, fy, fz], [fx + 1.0, fy + 1.0, fz + 1.0]),
        _ => ([fx, fy, fz], [fx + 0.75, fy + 1.0, fz + 1.0]),
    };

    let mut slots = [113u8; 6];
    let mut rots = [0u8; 6];
    for (i, face) in [
        BlockFace::Top,
        BlockFace::Bottom,
        BlockFace::North,
        BlockFace::South,
        BlockFace::West,
        BlockFace::East,
    ]
    .iter()
    .enumerate()
    {
        rots[i] = get_piston_side_rot(ori, *face);
    }
    let (front_idx, back_idx) = match ori {
        0 => (1, 0),
        1 => (0, 1),
        2 => (2, 3),
        3 => (3, 2),
        4 => (4, 5),
        _ => (5, 4),
    };
    slots[front_idx] = 115;
    slots[back_idx] = 114;

    let mut custom = [[[0.0f32; 2]; 4]; 6];
    for face_idx in 0..6 {
        let slot = slots[face_idx];
        let (u0, v0, u1, v1) = crate::texture::get_slot_uv(slot);
        let rot = rots[face_idx];
        if face_idx == front_idx || face_idx == back_idx {
            let uvs = if rot == 0 {
                match face_idx {
                    0 => [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
                    1 => [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
                    _ => [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
                }
            } else {
                get_rotated_quad_uvs(u0, v0, u1, v1, rot)
            };
            custom[face_idx] = uvs;
        } else {
            // Side faces of the base: bottom 12 pixels [v0 + 0.25*dv, v1] so it is not smushed
            let v_cut = v0 + 0.25 * (v1 - v0);
            let uvs = if rot == 0 {
                [[u0, v_cut], [u0, v1], [u1, v1], [u1, v_cut]]
            } else {
                get_rotated_quad_uvs(u0, v_cut, u1, v1, rot)
            };
            custom[face_idx] = uvs;
        }
    }

    let id_above = world.get_block(wx, wy + 1, wz).id;
    let id_below = world.get_block(wx, wy - 1, wz).id;
    let id_north = world.get_block(wx, wy, wz - 1).id;
    let id_south = world.get_block(wx, wy, wz + 1).id;
    let id_west = world.get_block(wx - 1, wy, wz).id;
    let id_east = world.get_block(wx + 1, wy, wz).id;

    let neighbor_ids = [id_above, id_below, id_north, id_south, id_west, id_east];
    let mut visible_mask = [true; 6];
    for face_idx in 0..6 {
        // Front face (slot 115, piston_inner) is inside the block space and always visible.
        // Outer boundary faces are culled if touching an opaque neighbor cube (e.g. grass below).
        if face_idx != front_idx && is_opaque_cube(neighbor_ids[face_idx]) {
            visible_mask[face_idx] = false;
        }
    }

    add_sub_box_with_uv(mesh, min, max, slots, rots, light, Some(custom), Some(visible_mask));
}

fn add_piston_extension(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;
    let ori = block.meta & 7;
    let is_sticky = (block.meta & 8) != 0;
    let head_slot = if is_sticky { 112 } else { 111 };

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    // 1. Head Plate:
    let (plate_min, plate_max) = match ori {
        0 => ([fx, fy, fz], [fx + 1.0, fy + 0.25, fz + 1.0]),
        1 => ([fx, fy + 0.75, fz], [fx + 1.0, fy + 1.0, fz + 1.0]),
        2 => ([fx, fy, fz], [fx + 1.0, fy + 1.0, fz + 0.25]),
        3 => ([fx, fy, fz + 0.75], [fx + 1.0, fy + 1.0, fz + 1.0]),
        4 => ([fx, fy, fz], [fx + 0.25, fy + 1.0, fz + 1.0]),
        _ => ([fx + 0.75, fy, fz], [fx + 1.0, fy + 1.0, fz + 1.0]),
    };

    let mut plate_slots = [113u8; 6];
    let mut plate_rots = [0u8; 6];
    for (i, face) in [
        BlockFace::Top,
        BlockFace::Bottom,
        BlockFace::North,
        BlockFace::South,
        BlockFace::West,
        BlockFace::East,
    ]
    .iter()
    .enumerate()
    {
        plate_rots[i] = get_piston_side_rot(ori, *face);
    }
    let (front_idx, back_idx) = match ori {
        0 => (1, 0),
        1 => (0, 1),
        2 => (2, 3),
        3 => (3, 2),
        4 => (4, 5),
        _ => (5, 4),
    };
    plate_slots[front_idx] = head_slot;
    plate_slots[back_idx] = 111; // In vanilla, back of head plate is wooden piston_top_normal (unsticky)

    let mut plate_custom = [[[0.0f32; 2]; 4]; 6];
    for face_idx in 0..6 {
        let slot = plate_slots[face_idx];
        let (u0, v0, u1, v1) = crate::texture::get_slot_uv(slot);
        let rot = plate_rots[face_idx];
        if face_idx == front_idx || face_idx == back_idx {
            let uvs = if rot == 0 {
                match face_idx {
                    0 => [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
                    1 => [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
                    _ => [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
                }
            } else {
                get_rotated_quad_uvs(u0, v0, u1, v1, rot)
            };
            plate_custom[face_idx] = uvs;
        } else {
            // Side faces of head plate: top 4 pixels [v0, v0 + 0.25*dv] so wooden rim is NOT smushed
            let v_cut = v0 + 0.25 * (v1 - v0);
            let uvs = if rot == 0 {
                [[u0, v0], [u0, v_cut], [u1, v_cut], [u1, v0]]
            } else {
                get_rotated_quad_uvs(u0, v0, u1, v_cut, rot)
            };
            plate_custom[face_idx] = uvs;
        }
    }

    add_sub_box_with_uv(mesh, plate_min, plate_max, plate_slots, plate_rots, light, Some(plate_custom), None);

    // 2. Continuous Connecting Rod (Shaft):
    // In vanilla Minecraft (models/block/piston_head.json), the rod is a 4x4 beam
    // spanning 16 pixels (1.0 block) between the head plate and the base casing.
    // Each of the 4 lengthwise faces uses the top 4 pixels of piston_side (slot 113)
    // with exact 1:1 pixel scale (16 texels along length, 4 texels across width).
    let (rod_min, rod_max) = match ori {
        0 => ([fx + 0.375, fy + 0.25, fz + 0.375], [fx + 0.625, fy + 1.25, fz + 0.625]),
        1 => ([fx + 0.375, fy - 0.25, fz + 0.375], [fx + 0.625, fy + 0.75, fz + 0.625]),
        2 => ([fx + 0.375, fy + 0.375, fz + 0.25], [fx + 0.625, fy + 0.625, fz + 1.25]),
        3 => ([fx + 0.375, fy + 0.375, fz - 0.25], [fx + 0.625, fy + 0.625, fz + 0.75]),
        4 => ([fx + 0.25, fy + 0.375, fz + 0.375], [fx + 1.25, fy + 0.625, fz + 0.625]),
        _ => ([fx - 0.25, fy + 0.375, fz + 0.375], [fx + 0.75, fy + 0.625, fz + 0.625]),
    };

    let (u0, v0, u1, v1) = crate::texture::get_slot_uv(113);
    let _du = u1 - u0;
    let dv = v1 - v0;
    let v_top = v0;
    let v_bot = v0 + 0.25 * dv;

    let [rx0, ry0, rz0] = rod_min;
    let [rx1, ry1, rz1] = rod_max;

    let rod_faces: Vec<([[f32; 3]; 4], [[f32; 2]; 4], f32)> = match ori {
        // Vertical rod (along Y)
        0 | 1 => vec![
            // North (-Z)
            (
                [[rx1, ry1, rz0], [rx1, ry0, rz0], [rx0, ry0, rz0], [rx0, ry1, rz0]],
                [[u1, v_bot], [u0, v_bot], [u0, v_top], [u1, v_top]],
                0.8,
            ),
            // South (+Z)
            (
                [[rx0, ry1, rz1], [rx0, ry0, rz1], [rx1, ry0, rz1], [rx1, ry1, rz1]],
                [[u1, v_top], [u0, v_top], [u0, v_bot], [u1, v_bot]],
                0.8,
            ),
            // West (-X)
            (
                [[rx0, ry1, rz0], [rx0, ry0, rz0], [rx0, ry0, rz1], [rx0, ry1, rz1]],
                [[u1, v_top], [u0, v_top], [u0, v_bot], [u1, v_bot]],
                0.6,
            ),
            // East (+X)
            (
                [[rx1, ry1, rz1], [rx1, ry0, rz1], [rx1, ry0, rz0], [rx1, ry1, rz0]],
                [[u1, v_bot], [u0, v_bot], [u0, v_top], [u1, v_top]],
                0.6,
            ),
        ],
        // Horizontal rod (along Z)
        2 | 3 => vec![
            // Top (+Y)
            (
                [[rx0, ry1, rz1], [rx1, ry1, rz1], [rx1, ry1, rz0], [rx0, ry1, rz0]],
                [[u1, v_bot], [u1, v_top], [u0, v_top], [u0, v_bot]],
                1.0,
            ),
            // Bottom (-Y)
            (
                [[rx0, ry0, rz0], [rx1, ry0, rz0], [rx1, ry0, rz1], [rx0, ry0, rz1]],
                [[u0, v_top], [u0, v_bot], [u1, v_bot], [u1, v_top]],
                0.5,
            ),
            // West (-X)
            (
                [[rx0, ry1, rz0], [rx0, ry0, rz0], [rx0, ry0, rz1], [rx0, ry1, rz1]],
                [[u0, v_top], [u0, v_bot], [u1, v_bot], [u1, v_top]],
                0.6,
            ),
            // East (+X)
            (
                [[rx1, ry1, rz1], [rx1, ry0, rz1], [rx1, ry0, rz0], [rx1, ry1, rz0]],
                [[u1, v_top], [u1, v_bot], [u0, v_bot], [u0, v_top]],
                0.6,
            ),
        ],
        // Horizontal rod (along X)
        _ => vec![
            // Top (+Y)
            (
                [[rx0, ry1, rz1], [rx1, ry1, rz1], [rx1, ry1, rz0], [rx0, ry1, rz0]],
                [[u0, v_bot], [u1, v_bot], [u1, v_top], [u0, v_top]],
                1.0,
            ),
            // Bottom (-Y)
            (
                [[rx0, ry0, rz0], [rx1, ry0, rz0], [rx1, ry0, rz1], [rx0, ry0, rz1]],
                [[u0, v_top], [u1, v_top], [u1, v_bot], [u0, v_bot]],
                0.5,
            ),
            // North (-Z)
            (
                [[rx1, ry1, rz0], [rx1, ry0, rz0], [rx0, ry0, rz0], [rx0, ry1, rz0]],
                [[u1, v_top], [u1, v_bot], [u0, v_bot], [u0, v_top]],
                0.8,
            ),
            // South (+Z)
            (
                [[rx0, ry1, rz1], [rx0, ry0, rz1], [rx1, ry0, rz1], [rx1, ry1, rz1]],
                [[u0, v_top], [u0, v_bot], [u1, v_bot], [u1, v_top]],
                0.8,
            ),
        ],
    };

    for (quad, uvs, shade) in rod_faces {
        let col = [shade, shade, shade, 1.0];
        let start_idx = mesh.vertices.len() as u32;
        for (i, &pos) in quad.iter().enumerate() {
            mesh.vertices.push(Vertex {
                position: pos,
                uv: uvs[i],
                color: col,
                normal: light,
            });
        }
        mesh.indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    }
}

fn add_pressure_plate_quads(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let slot = match block.id {
        70 => 1,
        72 => 6,
        147 => 60,
        _ => 59,
    };

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    let h = if block.meta == 1 { 0.5 / 16.0 } else { 1.0 / 16.0 };
    let min = [fx + 0.0625, fy, fz + 0.0625];
    let max = [fx + 0.9375, fy + h, fz + 0.9375];

    add_sub_box_proportional(mesh, min, max, [fx, fy, fz], [slot; 6], light);
}

fn add_button_quads(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let slot = if block.id == 77 { 1 } else { 6 };
    let is_pressed = (block.meta & 8) != 0;
    let depth = if is_pressed { 1.0 / 16.0 } else { 2.0 / 16.0 };

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    let (min, max) = match block.meta & 7 {
        1 => ([fx, fy + 0.375, fz + 0.3125], [fx + depth, fy + 0.625, fz + 0.6875]),
        2 => ([fx + 1.0 - depth, fy + 0.375, fz + 0.3125], [fx + 1.0, fy + 0.625, fz + 0.6875]),
        3 => ([fx + 0.3125, fy + 0.375, fz], [fx + 0.6875, fy + 0.625, fz + depth]),
        4 => ([fx + 0.3125, fy + 0.375, fz + 1.0 - depth], [fx + 0.6875, fy + 0.625, fz + 1.0]),
        _ => ([fx, fy + 0.375, fz + 0.3125], [fx + depth, fy + 0.625, fz + 0.6875]),
    };

    add_sub_box_proportional(mesh, min, max, [fx, fy, fz], [slot; 6], light);
}

fn add_lever_quads(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let ori = block.meta & 7;
    let is_active = (block.meta & 8) != 0;

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    // 1. Cobblestone Base: proportional UV mapping gives clean, un-squashed cobblestone
    let (base_min, base_max) = match ori {
        5 => ([fx + 0.3125, fy, fz + 0.25], [fx + 0.6875, fy + 0.1875, fz + 0.75]),
        6 => ([fx + 0.25, fy, fz + 0.3125], [fx + 0.75, fy + 0.1875, fz + 0.6875]),
        1 => ([fx, fy + 0.25, fz + 0.3125], [fx + 0.1875, fy + 0.75, fz + 0.6875]),
        2 => ([fx + 1.0 - 0.1875, fy + 0.25, fz + 0.3125], [fx + 1.0, fy + 0.75, fz + 0.6875]),
        3 => ([fx + 0.3125, fy + 0.25, fz], [fx + 0.6875, fy + 0.75, fz + 0.1875]),
        4 => ([fx + 0.3125, fy + 0.25, fz + 1.0 - 0.1875], [fx + 0.6875, fy + 0.75, fz + 1.0]),
        0 => ([fx + 0.25, fy + 1.0 - 0.1875, fz + 0.3125], [fx + 0.75, fy + 1.0, fz + 0.6875]),
        7 => ([fx + 0.3125, fy + 1.0 - 0.1875, fz + 0.25], [fx + 0.6875, fy + 1.0, fz + 0.75]),
        _ => ([fx + 0.3125, fy, fz + 0.25], [fx + 0.6875, fy + 0.1875, fz + 0.75]),
    };
    add_sub_box_proportional(mesh, base_min, base_max, [fx, fy, fz], [5; 6], light);

    // 2. Lever Stick:
    // A 2x2 pixel stick, 10 pixels long, pivoted and tilted at 45 degrees.
    // Built using an orthonormal basis (l_hat along stick, w_hat along hinge axis, u_hat = w_hat x l_hat)
    // so the stick head cap is a crisp square perpendicular to the stick axis.
    let s: f32 = 0.7071068; // sqrt(2)/2
    let (base_c, l_hat, w_hat) = match ori {
        // Floor lever (pointing along Z / North-South): Off = North (-Z), On = South (+Z)
        5 => {
            let b = [fx + 0.5, fy + 0.0625, fz + 0.5];
            let w = [1.0, 0.0, 0.0];
            let l = if is_active { [0.0, s, s] } else { [0.0, s, -s] };
            (b, l, w)
        }
        // Floor lever (pointing along X / East-West): Off = West (-X), On = East (+X)
        6 => {
            let b = [fx + 0.5, fy + 0.0625, fz + 0.5];
            let w = [0.0, 0.0, 1.0];
            let l = if is_active { [s, s, 0.0] } else { [-s, s, 0.0] };
            (b, l, w)
        }
        // Wall lever: West wall (attached at X=0, points +X into room): Off = Up (+Y), On = Down (-Y)
        1 => {
            let b = [fx + 0.0625, fy + 0.5, fz + 0.5];
            let w = [0.0, 0.0, 1.0];
            let l = if is_active { [s, -s, 0.0] } else { [s, s, 0.0] };
            (b, l, w)
        }
        // Wall lever: East wall (attached at X=1, points -X into room): Off = Up (+Y), On = Down (-Y)
        2 => {
            let b = [fx + 1.0 - 0.0625, fy + 0.5, fz + 0.5];
            let w = [0.0, 0.0, 1.0];
            let l = if is_active { [-s, -s, 0.0] } else { [-s, s, 0.0] };
            (b, l, w)
        }
        // Wall lever: North wall (attached at Z=0, points +Z into room): Off = Up (+Y), On = Down (-Y)
        3 => {
            let b = [fx + 0.5, fy + 0.5, fz + 0.0625];
            let w = [1.0, 0.0, 0.0];
            let l = if is_active { [0.0, -s, s] } else { [0.0, s, s] };
            (b, l, w)
        }
        // Wall lever: South wall (attached at Z=1, points -Z into room): Off = Up (+Y), On = Down (-Y)
        4 => {
            let b = [fx + 0.5, fy + 0.5, fz + 1.0 - 0.0625];
            let w = [1.0, 0.0, 0.0];
            let l = if is_active { [0.0, -s, -s] } else { [0.0, s, -s] };
            (b, l, w)
        }
        // Ceiling lever
        0 => {
            let b = [fx + 0.5, fy + 1.0 - 0.0625, fz + 0.5];
            let w = [0.0, 0.0, 1.0];
            let l = if is_active { [s, -s, 0.0] } else { [-s, -s, 0.0] };
            (b, l, w)
        }
        7 => {
            let b = [fx + 0.5, fy + 1.0 - 0.0625, fz + 0.5];
            let w = [1.0, 0.0, 0.0];
            let l = if is_active { [0.0, -s, s] } else { [0.0, -s, -s] };
            (b, l, w)
        }
        _ => {
            let b = [fx + 0.5, fy + 0.0625, fz + 0.5];
            (b, [0.0, s, -s], [1.0, 0.0, 0.0])
        }
    };

    let u_hat = [
        w_hat[1] * l_hat[2] - w_hat[2] * l_hat[1],
        w_hat[2] * l_hat[0] - w_hat[0] * l_hat[2],
        w_hat[0] * l_hat[1] - w_hat[1] * l_hat[0],
    ];

    let length: f32 = 10.0 / 16.0;
    let hw: f32 = 1.0 / 16.0;
    let top_c = [
        base_c[0] + length * l_hat[0],
        base_c[1] + length * l_hat[1],
        base_c[2] + length * l_hat[2],
    ];

    let b_0 = [
        base_c[0] - hw * u_hat[0] - hw * w_hat[0],
        base_c[1] - hw * u_hat[1] - hw * w_hat[1],
        base_c[2] - hw * u_hat[2] - hw * w_hat[2],
    ];
    let b_1 = [
        base_c[0] + hw * u_hat[0] - hw * w_hat[0],
        base_c[1] + hw * u_hat[1] - hw * w_hat[1],
        base_c[2] + hw * u_hat[2] - hw * w_hat[2],
    ];
    let b_2 = [
        base_c[0] + hw * u_hat[0] + hw * w_hat[0],
        base_c[1] + hw * u_hat[1] + hw * w_hat[1],
        base_c[2] + hw * u_hat[2] + hw * w_hat[2],
    ];
    let b_3 = [
        base_c[0] - hw * u_hat[0] + hw * w_hat[0],
        base_c[1] - hw * u_hat[1] + hw * w_hat[1],
        base_c[2] - hw * u_hat[2] + hw * w_hat[2],
    ];

    let t_0 = [
        top_c[0] - hw * u_hat[0] - hw * w_hat[0],
        top_c[1] - hw * u_hat[1] - hw * w_hat[1],
        top_c[2] - hw * u_hat[2] - hw * w_hat[2],
    ];
    let t_1 = [
        top_c[0] + hw * u_hat[0] - hw * w_hat[0],
        top_c[1] + hw * u_hat[1] - hw * w_hat[1],
        top_c[2] + hw * u_hat[2] - hw * w_hat[2],
    ];
    let t_2 = [
        top_c[0] + hw * u_hat[0] + hw * w_hat[0],
        top_c[1] + hw * u_hat[1] + hw * w_hat[1],
        top_c[2] + hw * u_hat[2] + hw * w_hat[2],
    ];
    let t_3 = [
        top_c[0] - hw * u_hat[0] + hw * w_hat[0],
        top_c[1] - hw * u_hat[1] + hw * w_hat[1],
        top_c[2] - hw * u_hat[2] + hw * w_hat[2],
    ];

    let (u0, v0, u1, v1) = crate::texture::get_slot_uv(176);
    let du = u1 - u0;
    let dv = v1 - v0;

    let u_min = u0 + (7.0 / 16.0) * du;
    let u_max = u0 + (9.0 / 16.0) * du;
    let v_min = v0 + (6.0 / 16.0) * dv;
    let v_tip = v0 + (8.0 / 16.0) * dv;
    let v_max = v0 + (16.0 / 16.0) * dv;

    let faces = [
        // Top cap (perpendicular to stick)
        (
            [t_0, t_1, t_2, t_3],
            [[u_min, v_min], [u_max, v_min], [u_max, v_tip], [u_min, v_tip]],
            1.0f32,
        ),
        // +U side face
        (
            [b_1, b_2, t_2, t_1],
            [[u_min, v_max], [u_max, v_max], [u_max, v_min], [u_min, v_min]],
            0.8f32,
        ),
        // -U side face
        (
            [b_3, b_0, t_0, t_3],
            [[u_min, v_max], [u_max, v_max], [u_max, v_min], [u_min, v_min]],
            0.8f32,
        ),
        // +W side face
        (
            [b_2, b_3, t_3, t_2],
            [[u_min, v_max], [u_max, v_max], [u_max, v_min], [u_min, v_min]],
            0.6f32,
        ),
        // -W side face
        (
            [b_0, b_1, t_1, t_0],
            [[u_min, v_max], [u_max, v_max], [u_max, v_min], [u_min, v_min]],
            0.6f32,
        ),
        // Bottom cap (inside base)
        (
            [b_0, b_3, b_2, b_1],
            [[u_min, v_max], [u_max, v_max], [u_max, v_tip], [u_min, v_tip]],
            0.5f32,
        ),
    ];

    for (quad, uvs, shade) in faces {
        let col = [shade, shade, shade, 1.0];
        let start_idx = mesh.vertices.len() as u32;
        for (i, &pos) in quad.iter().enumerate() {
            mesh.vertices.push(Vertex {
                position: pos,
                uv: uvs[i],
                color: col,
                normal: light,
            });
        }
        mesh.indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    }
}

fn add_stairs_quads(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    let (slot, _) = crate::texture::get_block_slot(block.id, block.meta, BlockFace::North);
    let slots = [slot; 6];

    let is_upside_down = (block.meta & 4) != 0;
    let facing = block.meta & 3;

    // 1. Base half-slab:
    let (base_min, base_max) = if is_upside_down {
        ([fx, fy + 0.5, fz], [fx + 1.0, fy + 1.0, fz + 1.0])
    } else {
        ([fx, fy, fz], [fx + 1.0, fy + 0.5, fz + 1.0])
    };
    add_sub_box_proportional(mesh, base_min, base_max, [fx, fy, fz], slots, light);

    // 2. Step quarter-box:
    let (step_y0, step_y1) = if is_upside_down {
        (fy, fy + 0.5)
    } else {
        (fy + 0.5, fy + 1.0)
    };

    let (step_min, step_max) = match facing {
        0 => ([fx + 0.5, step_y0, fz], [fx + 1.0, step_y1, fz + 1.0]), // East
        1 => ([fx, step_y0, fz], [fx + 0.5, step_y1, fz + 1.0]),       // West
        2 => ([fx, step_y0, fz + 0.5], [fx + 1.0, step_y1, fz + 1.0]), // South
        _ => ([fx, step_y0, fz], [fx + 1.0, step_y1, fz + 0.5]),       // North
    };
    add_sub_box_proportional(mesh, step_min, step_max, [fx, fy, fz], slots, light);
}

fn add_slab_quads(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    let (slot, _) = crate::texture::get_block_slot(block.id, block.meta, BlockFace::North);
    let slots = [slot; 6];

    let is_upside_down = (block.meta & 8) != 0;
    let (min, max) = if is_upside_down {
        ([fx, fy + 0.5, fz], [fx + 1.0, fy + 1.0, fz + 1.0])
    } else {
        ([fx, fy, fz], [fx + 1.0, fy + 0.5, fz + 1.0])
    };
    add_sub_box_proportional(mesh, min, max, [fx, fy, fz], slots, light);
}

fn add_fence_quads(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    let (slot, _) = crate::texture::get_block_slot(block.id, block.meta, BlockFace::North);
    let slots = [slot; 6];

    // Central post: 4x4 pixels, full height
    let post_min = [fx + 0.375, fy, fz + 0.375];
    let post_max = [fx + 0.625, fy + 1.0, fz + 0.625];
    add_sub_box_proportional(mesh, post_min, post_max, [fx, fy, fz], slots, light);

    let id_north = world.get_block(wx, wy, wz - 1).id;
    let id_south = world.get_block(wx, wy, wz + 1).id;
    let id_west = world.get_block(wx - 1, wy, wz).id;
    let id_east = world.get_block(wx + 1, wy, wz).id;

    let connect_n = id_north == block.id || is_opaque_cube(id_north) || id_north == 107;
    let connect_s = id_south == block.id || is_opaque_cube(id_south) || id_south == 107;
    let connect_w = id_west == block.id || is_opaque_cube(id_west) || id_west == 107;
    let connect_e = id_east == block.id || is_opaque_cube(id_east) || id_east == 107;

    if connect_n {
        let r1 = ([fx + 0.4375, fy + 0.75, fz], [fx + 0.5625, fy + 0.9375, fz + 0.375]);
        let r2 = ([fx + 0.4375, fy + 0.375, fz], [fx + 0.5625, fy + 0.5625, fz + 0.375]);
        add_sub_box_proportional(mesh, r1.0, r1.1, [fx, fy, fz], slots, light);
        add_sub_box_proportional(mesh, r2.0, r2.1, [fx, fy, fz], slots, light);
    }
    if connect_s {
        let r1 = ([fx + 0.4375, fy + 0.75, fz + 0.625], [fx + 0.5625, fy + 0.9375, fz + 1.0]);
        let r2 = ([fx + 0.4375, fy + 0.375, fz + 0.625], [fx + 0.5625, fy + 0.5625, fz + 1.0]);
        add_sub_box_proportional(mesh, r1.0, r1.1, [fx, fy, fz], slots, light);
        add_sub_box_proportional(mesh, r2.0, r2.1, [fx, fy, fz], slots, light);
    }
    if connect_w {
        let r1 = ([fx, fy + 0.75, fz + 0.4375], [fx + 0.375, fy + 0.9375, fz + 0.5625]);
        let r2 = ([fx, fy + 0.375, fz + 0.4375], [fx + 0.375, fy + 0.5625, fz + 0.5625]);
        add_sub_box_proportional(mesh, r1.0, r1.1, [fx, fy, fz], slots, light);
        add_sub_box_proportional(mesh, r2.0, r2.1, [fx, fy, fz], slots, light);
    }
    if connect_e {
        let r1 = ([fx + 0.625, fy + 0.75, fz + 0.4375], [fx + 1.0, fy + 0.9375, fz + 0.5625]);
        let r2 = ([fx + 0.625, fy + 0.375, fz + 0.4375], [fx + 1.0, fy + 0.5625, fz + 0.5625]);
        add_sub_box_proportional(mesh, r1.0, r1.1, [fx, fy, fz], slots, light);
        add_sub_box_proportional(mesh, r2.0, r2.1, [fx, fy, fz], slots, light);
    }
}

fn add_pane_quads(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let above = world.get_block(wx, wy + 1, wz);
    let sky_l = (block.sky_light.max(above.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (block.block_light.max(above.block_light) as f32 / 15.0).clamp(0.0, 1.0);
    let light = [sky_l, block_l, 1.0];

    let (slot, _) = crate::texture::get_block_slot(block.id, block.meta, BlockFace::North);
    let slots = [slot; 6];

    let id_north = world.get_block(wx, wy, wz - 1).id;
    let id_south = world.get_block(wx, wy, wz + 1).id;
    let id_west = world.get_block(wx - 1, wy, wz).id;
    let id_east = world.get_block(wx + 1, wy, wz).id;

    let connect_n = id_north == block.id || is_opaque_cube(id_north) || is_pane(id_north);
    let connect_s = id_south == block.id || is_opaque_cube(id_south) || is_pane(id_south);
    let connect_w = id_west == block.id || is_opaque_cube(id_west) || is_pane(id_west);
    let connect_e = id_east == block.id || is_opaque_cube(id_east) || is_pane(id_east);

    let post_min = [fx + 0.4375, fy, fz + 0.4375];
    let post_max = [fx + 0.5625, fy + 1.0, fz + 0.5625];
    add_sub_box_proportional(mesh, post_min, post_max, [fx, fy, fz], slots, light);

    if connect_n {
        let b = ([fx + 0.4375, fy, fz], [fx + 0.5625, fy + 1.0, fz + 0.4375]);
        add_sub_box_proportional(mesh, b.0, b.1, [fx, fy, fz], slots, light);
    }
    if connect_s {
        let b = ([fx + 0.4375, fy, fz + 0.5625], [fx + 0.5625, fy + 1.0, fz + 1.0]);
        add_sub_box_proportional(mesh, b.0, b.1, [fx, fy, fz], slots, light);
    }
    if connect_w {
        let b = ([fx, fy, fz + 0.4375], [fx + 0.4375, fy + 1.0, fz + 0.5625]);
        add_sub_box_proportional(mesh, b.0, b.1, [fx, fy, fz], slots, light);
    }
    if connect_e {
        let b = ([fx + 0.5625, fy, fz + 0.4375], [fx + 1.0, fy + 1.0, fz + 0.5625]);
        add_sub_box_proportional(mesh, b.0, b.1, [fx, fy, fz], slots, light);
    }
    if !connect_n && !connect_s && !connect_w && !connect_e {
        let b1 = ([fx + 0.4375, fy, fz], [fx + 0.5625, fy + 1.0, fz + 1.0]);
        let b2 = ([fx, fy, fz + 0.4375], [fx + 1.0, fy + 1.0, fz + 0.5625]);
        add_sub_box_proportional(mesh, b1.0, b1.1, [fx, fy, fz], slots, light);
        add_sub_box_proportional(mesh, b2.0, b2.1, [fx, fy, fz], slots, light);
    }
}

pub fn mesh_chunk_column(chunk: &ChunkColumn, world: &World) -> MeshData {
    let mut mesh = MeshData::new();
    let cx = chunk.x;
    let cz = chunk.z;
    let base_x = cx * 16;
    let base_z = cz * 16;

    let neighbor_px = world.get_chunk(cx + 1, cz);
    let neighbor_nx = world.get_chunk(cx - 1, cz);
    let neighbor_pz = world.get_chunk(cx, cz + 1);
    let neighbor_nz = world.get_chunk(cx, cz - 1);

    for s_idx in 0..16 {
        let section = match &chunk.sections[s_idx] {
            Some(s) => s,
            None => continue,
        };

        let base_y = (s_idx * 16) as i32;
        let sec_above = if s_idx < 15 {
            chunk.sections[s_idx + 1].as_deref()
        } else {
            None
        };
        let sec_below = if s_idx > 0 {
            chunk.sections[s_idx - 1].as_deref()
        } else {
            None
        };
        let sec_nx = neighbor_nx.and_then(|c| c.sections[s_idx].as_deref());
        let sec_px = neighbor_px.and_then(|c| c.sections[s_idx].as_deref());
        let sec_nz = neighbor_nz.and_then(|c| c.sections[s_idx].as_deref());
        let sec_pz = neighbor_pz.and_then(|c| c.sections[s_idx].as_deref());

        for ly in 0..16 {
            for lz in 0..16 {
                for lx in 0..16 {
                    let idx = (ly << 8) | (lz << 4) | lx;
                    let id = section.block_ids[idx];
                    if id == 0 {
                        continue;
                    }

                    let block = section.get_block(lx, ly, lz);
                    let block_id = block.id;
                    let wx = base_x + lx as i32;
                    let wy = base_y + ly as i32;
                    let wz = base_z + lz as i32;

                    if is_cross_plant(block_id) {
                        add_cross_quads(&mut mesh, wx, wy, wz, block);
                        continue;
                    }

                    if is_torch(block_id) {
                        add_torch_quads(&mut mesh, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 55 {
                        add_redstone_wire_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if is_door(block_id) {
                        add_door_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if is_pressure_plate(block_id) {
                        add_pressure_plate_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if is_button(block_id) {
                        add_button_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if is_lever(block_id) {
                        add_lever_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if is_stairs(block_id) {
                        add_stairs_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if is_slab(block_id) {
                        add_slab_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if is_fence(block_id) {
                        add_fence_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if is_pane(block_id) {
                        add_pane_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }


                    if block_id == 26 {
                        add_bed_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 34 {
                        add_piston_extension(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 29 || block_id == 33 {
                        let (dx, dy, dz) = get_piston_dir(block.meta & 7);
                        let is_ext = (block.meta & 8) != 0 || world.get_block(wx + dx, wy + dy, wz + dz).id == 34;
                        if is_ext {
                            add_extended_piston_base(&mut mesh, world, wx, wy, wz, block);
                            continue;
                        }
                    }

                    if is_flat_floor(block_id) {
                        let h = if block_id == 171 { 1.0 / 16.0 } else { 0.02 };
                        add_quad(&mut mesh, world, wx, wy, wz, BlockFace::Top, block, h);
                        continue;
                    }

                    let id_above = if ly < 15 {
                        section.block_ids[((ly + 1) << 8) | (lz << 4) | lx] as u16
                    } else if let Some(above) = sec_above {
                        above.block_ids[(lz << 4) | lx] as u16
                    } else {
                        0
                    };

                    let id_below = if ly > 0 {
                        section.block_ids[((ly - 1) << 8) | (lz << 4) | lx] as u16
                    } else if let Some(below) = sec_below {
                        below.block_ids[(15 << 8) | (lz << 4) | lx] as u16
                    } else {
                        if wy > 0 { 0 } else { 1 }
                    };

                    let id_west = if lx > 0 {
                        section.block_ids[(ly << 8) | (lz << 4) | (lx - 1)] as u16
                    } else if let Some(nx) = sec_nx {
                        nx.block_ids[(ly << 8) | (lz << 4) | 15] as u16
                    } else {
                        0
                    };

                    let id_east = if lx < 15 {
                        section.block_ids[(ly << 8) | (lz << 4) | (lx + 1)] as u16
                    } else if let Some(px) = sec_px {
                        px.block_ids[(ly << 8) | (lz << 4) | 0] as u16
                    } else {
                        0
                    };

                    let id_north = if lz > 0 {
                        section.block_ids[(ly << 8) | ((lz - 1) << 4) | lx] as u16
                    } else if let Some(nz) = sec_nz {
                        nz.block_ids[(ly << 8) | (15 << 4) | lx] as u16
                    } else {
                        0
                    };

                    let id_south = if lz < 15 {
                        section.block_ids[(ly << 8) | ((lz + 1) << 4) | lx] as u16
                    } else if let Some(pz) = sec_pz {
                        pz.block_ids[(ly << 8) | (0 << 4) | lx] as u16
                    } else {
                        0
                    };

                    let top_visible = should_render_face(block_id, id_above);
                    let bottom_visible = should_render_face(block_id, id_below);
                    let west_visible = should_render_face(block_id, id_west);
                    let east_visible = should_render_face(block_id, id_east);
                    let north_visible = should_render_face(block_id, id_north);
                    let south_visible = should_render_face(block_id, id_south);

                    if !(top_visible || bottom_visible || west_visible || east_visible || north_visible || south_visible) {
                        continue;
                    }

                    let mut height = 1.0;
                    if block_id == 8 || block_id == 9 {
                        if id_above == 8 || id_above == 9 || id_above == 79 || id_above == 174 || is_opaque_cube(id_above) {
                            height = 1.0;
                        } else {
                            if block.meta >= 8 {
                                height = 14.0 / 16.0; // Falling water
                            } else {
                                // Source block or flowing
                                height = (14.0 - (block.meta & 7) as f32 * 1.5) / 16.0;
                                if height < 0.1 { height = 0.1; }
                            }
                        }
                    } else if block_id == 78 {
                        height = (((block.meta & 7) + 1) as f32 * 2.0) / 16.0;
                    } else if block_id == 93 || block_id == 94 || block_id == 149 || block_id == 150 {
                        height = 2.0 / 16.0;
                    } else if block_id == 96 || block_id == 167 {
                        height = 3.0 / 16.0;
                    }

                    let is_p = is_piston(block_id);
                    if top_visible {
                        let rot = if is_p { get_piston_side_rot(block.meta, BlockFace::Top) } else { 0 };
                        add_quad_with_rot(&mut mesh, world, wx, wy, wz, BlockFace::Top, block, height, rot);
                    }
                    if bottom_visible {
                        let rot = if is_p { get_piston_side_rot(block.meta, BlockFace::Bottom) } else { 0 };
                        add_quad_with_rot(&mut mesh, world, wx, wy, wz, BlockFace::Bottom, block, height, rot);
                    }
                    if west_visible {
                        let rot = if is_p { get_piston_side_rot(block.meta, BlockFace::West) } else { 0 };
                        add_quad_with_rot(&mut mesh, world, wx, wy, wz, BlockFace::West, block, height, rot);
                    }
                    if east_visible {
                        let rot = if is_p { get_piston_side_rot(block.meta, BlockFace::East) } else { 0 };
                        add_quad_with_rot(&mut mesh, world, wx, wy, wz, BlockFace::East, block, height, rot);
                    }
                    if north_visible {
                        let rot = if is_p { get_piston_side_rot(block.meta, BlockFace::North) } else { 0 };
                        add_quad_with_rot(&mut mesh, world, wx, wy, wz, BlockFace::North, block, height, rot);
                    }
                    if south_visible {
                        let rot = if is_p { get_piston_side_rot(block.meta, BlockFace::South) } else { 0 };
                        add_quad_with_rot(&mut mesh, world, wx, wy, wz, BlockFace::South, block, height, rot);
                    }
                }
            }
        }
    }

    mesh
}

fn get_vertex_ao(side1: bool, side2: bool, corner: bool) -> f32 {
    if side1 && side2 {
        0.45
    } else {
        let count = (side1 as u8) + (side2 as u8) + (corner as u8);
        match count {
            0 => 1.0,
            1 => 0.8,
            2 => 0.65,
            _ => 0.45,
        }
    }
}

fn compute_face_ao(world: &World, wx: i32, wy: i32, wz: i32, face: BlockFace) -> [f32; 4] {
    let is_solid = |dx: i32, dy: i32, dz: i32| -> bool {
        is_opaque_cube(world.get_block(wx + dx, wy + dy, wz + dz).id)
    };

    match face {
        BlockFace::Top => {
            let ao0 = get_vertex_ao(is_solid(-1, 1, 0), is_solid(0, 1, 1), is_solid(-1, 1, 1));
            let ao1 = get_vertex_ao(is_solid(1, 1, 0), is_solid(0, 1, 1), is_solid(1, 1, 1));
            let ao2 = get_vertex_ao(is_solid(1, 1, 0), is_solid(0, 1, -1), is_solid(1, 1, -1));
            let ao3 = get_vertex_ao(is_solid(-1, 1, 0), is_solid(0, 1, -1), is_solid(-1, 1, -1));
            [ao0, ao1, ao2, ao3]
        }
        BlockFace::Bottom => {
            let ao0 = get_vertex_ao(is_solid(-1, -1, 0), is_solid(0, -1, -1), is_solid(-1, -1, -1));
            let ao1 = get_vertex_ao(is_solid(1, -1, 0), is_solid(0, -1, -1), is_solid(1, -1, -1));
            let ao2 = get_vertex_ao(is_solid(1, -1, 0), is_solid(0, -1, 1), is_solid(1, -1, 1));
            let ao3 = get_vertex_ao(is_solid(-1, -1, 0), is_solid(0, -1, 1), is_solid(-1, -1, 1));
            [ao0, ao1, ao2, ao3]
        }
        BlockFace::North => {
            let ao0 = get_vertex_ao(is_solid(1, 0, -1), is_solid(0, 1, -1), is_solid(1, 1, -1));
            let ao1 = get_vertex_ao(is_solid(1, 0, -1), is_solid(0, -1, -1), is_solid(1, -1, -1));
            let ao2 = get_vertex_ao(is_solid(-1, 0, -1), is_solid(0, -1, -1), is_solid(-1, -1, -1));
            let ao3 = get_vertex_ao(is_solid(-1, 0, -1), is_solid(0, 1, -1), is_solid(-1, 1, -1));
            [ao0, ao1, ao2, ao3]
        }
        BlockFace::South => {
            let ao0 = get_vertex_ao(is_solid(-1, 0, 1), is_solid(0, 1, 1), is_solid(-1, 1, 1));
            let ao1 = get_vertex_ao(is_solid(-1, 0, 1), is_solid(0, -1, 1), is_solid(-1, -1, 1));
            let ao2 = get_vertex_ao(is_solid(1, 0, 1), is_solid(0, -1, 1), is_solid(1, -1, 1));
            let ao3 = get_vertex_ao(is_solid(1, 0, 1), is_solid(0, 1, 1), is_solid(1, 1, 1));
            [ao0, ao1, ao2, ao3]
        }
        BlockFace::West => {
            let ao0 = get_vertex_ao(is_solid(-1, 0, -1), is_solid(-1, 1, 0), is_solid(-1, 1, -1));
            let ao1 = get_vertex_ao(is_solid(-1, 0, -1), is_solid(-1, -1, 0), is_solid(-1, -1, -1));
            let ao2 = get_vertex_ao(is_solid(-1, 0, 1), is_solid(-1, -1, 0), is_solid(-1, -1, 1));
            let ao3 = get_vertex_ao(is_solid(-1, 0, 1), is_solid(-1, 1, 0), is_solid(-1, 1, 1));
            [ao0, ao1, ao2, ao3]
        }
        BlockFace::East => {
            let ao0 = get_vertex_ao(is_solid(1, 0, 1), is_solid(1, 1, 0), is_solid(1, 1, 1));
            let ao1 = get_vertex_ao(is_solid(1, 0, 1), is_solid(1, -1, 0), is_solid(1, -1, 1));
            let ao2 = get_vertex_ao(is_solid(1, 0, -1), is_solid(1, -1, 0), is_solid(1, -1, -1));
            let ao3 = get_vertex_ao(is_solid(1, 0, -1), is_solid(1, 1, 0), is_solid(1, 1, -1));
            [ao0, ao1, ao2, ao3]
        }
    }
}

fn add_quad(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    face: BlockFace,
    block: Block,
    height: f32,
) {
    add_quad_with_rot(mesh, world, wx, wy, wz, face, block, height, 0);
}

fn get_rotated_quad_uvs(u0: f32, v0: f32, u1: f32, v1: f32, rot: u8) -> [[f32; 2]; 4] {
    match rot {
        1 => [[u1, v0], [u0, v0], [u0, v1], [u1, v1]],
        2 => [[u1, v1], [u1, v0], [u0, v0], [u0, v1]],
        3 => [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
        _ => [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
    }
}

fn add_quad_with_rot(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    face: BlockFace,
    block: Block,
    height: f32,
    rot: u8,
) {
    let is_transparent = block.id == 8 || block.id == 9 || block.id == 79 || block.id == 95;

    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let (slot, tint) = crate::texture::get_block_slot(block.id, block.meta, face);
    let (u0, v0, u1, v1) = crate::texture::get_slot_uv(slot);

    let shade = face_shading_factor(face);
    let is_water = block.id == 8 || block.id == 9;
    let alpha = if is_water { 0.7 } else { 1.0 };
    let final_color = match tint {
        Some(t) => [
            t[0] * shade,
            t[1] * shade,
            t[2] * shade,
            alpha,
        ],
        None => [shade, shade, shade, alpha],
    };

    let (adj_x, adj_y, adj_z) = match face {
        BlockFace::Top => (wx, wy + 1, wz),
        BlockFace::Bottom => (wx, wy - 1, wz),
        BlockFace::North => (wx, wy, wz - 1),
        BlockFace::South => (wx, wy, wz + 1),
        BlockFace::West => (wx - 1, wy, wz),
        BlockFace::East => (wx + 1, wy, wz),
    };
    let adj_b = world.get_block(adj_x, adj_y, adj_z);
    let sky_l = (adj_b.sky_light.max(block.sky_light) as f32 / 15.0).clamp(0.0, 1.0);
    let block_l = (adj_b.block_light.max(block.block_light) as f32 / 15.0).clamp(0.0, 1.0);

    let ao = if is_water {
        [1.0, 1.0, 1.0, 1.0]
    } else {
        compute_face_ao(world, wx, wy, wz, face)
    };

    let quad = match face {
        BlockFace::Bottom => {
            let uvs = if rot == 0 {
                [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]
            } else {
                get_rotated_quad_uvs(u0, v0, u1, v1, rot)
            };
            [
                ([fx, fy, fz], uvs[0]),
                ([fx + 1.0, fy, fz], uvs[1]),
                ([fx + 1.0, fy, fz + 1.0], uvs[2]),
                ([fx, fy, fz + 1.0], uvs[3]),
            ]
        }
        BlockFace::Top => {
            let uvs = if rot == 0 {
                [[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
            } else {
                get_rotated_quad_uvs(u0, v0, u1, v1, rot)
            };
            [
                ([fx, fy + height, fz + 1.0], uvs[0]),
                ([fx + 1.0, fy + height, fz + 1.0], uvs[1]),
                ([fx + 1.0, fy + height, fz], uvs[2]),
                ([fx, fy + height, fz], uvs[3]),
            ]
        }
        BlockFace::North => {
            let uvs = get_rotated_quad_uvs(u0, v0, u1, v1, rot);
            [
                ([fx + 1.0, fy + height, fz], uvs[0]),
                ([fx + 1.0, fy, fz], uvs[1]),
                ([fx, fy, fz], uvs[2]),
                ([fx, fy + height, fz], uvs[3]),
            ]
        }
        BlockFace::South => {
            let uvs = get_rotated_quad_uvs(u0, v0, u1, v1, rot);
            [
                ([fx, fy + height, fz + 1.0], uvs[0]),
                ([fx, fy, fz + 1.0], uvs[1]),
                ([fx + 1.0, fy, fz + 1.0], uvs[2]),
                ([fx + 1.0, fy + height, fz + 1.0], uvs[3]),
            ]
        }
        BlockFace::West => {
            let uvs = get_rotated_quad_uvs(u0, v0, u1, v1, rot);
            [
                ([fx, fy + height, fz], uvs[0]),
                ([fx, fy, fz], uvs[1]),
                ([fx, fy, fz + 1.0], uvs[2]),
                ([fx, fy + height, fz + 1.0], uvs[3]),
            ]
        }
        BlockFace::East => {
            let uvs = get_rotated_quad_uvs(u0, v0, u1, v1, rot);
            [
                ([fx + 1.0, fy + height, fz + 1.0], uvs[0]),
                ([fx + 1.0, fy, fz + 1.0], uvs[1]),
                ([fx + 1.0, fy, fz], uvs[2]),
                ([fx + 1.0, fy + height, fz], uvs[3]),
            ]
        }
    };

    if is_transparent {
        let start_idx = mesh.transparent_vertices.len() as u32;
        for (i, &(pos, uv)) in quad.iter().enumerate() {
            mesh.transparent_vertices.push(Vertex {
                position: pos,
                uv,
                color: final_color,
                normal: [sky_l, block_l, ao[i]],
            });
        }
        mesh.transparent_indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    } else {
        let start_idx = mesh.vertices.len() as u32;
        for (i, &(pos, uv)) in quad.iter().enumerate() {
            mesh.vertices.push(Vertex {
                position: pos,
                uv,
                color: final_color,
                normal: [sky_l, block_l, ao[i]],
            });
        }
        mesh.indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torch_mesh() {
        let mut mesh = MeshData::new();
        let block = Block {
            id: 50,
            meta: 0,
            sky_light: 15,
            block_light: 14,
        };
        add_torch_quads(&mut mesh, 10, 64, 10, block);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);

        // Check standing bounds: within block bounds [10.0, 11.0] x [64.0, 65.0] x [10.0, 11.0]
        for v in &mesh.vertices {
            assert!(v.position[0] >= 10.0 && v.position[0] <= 11.0);
            assert!(v.position[1] >= 64.0 && v.position[1] <= 65.0);
            assert!(v.position[2] >= 10.0 && v.position[2] <= 11.0);
        }

        // Verify that all 6 faces have outward normals (v1 - v0) x (v2 - v0)
        let expected_normals = [
            [0.0, 1.0, 0.0],  // Top (+Y)
            [0.0, 0.0, -1.0], // North (-Z)
            [0.0, 0.0, 1.0],  // South (+Z)
            [-1.0, 0.0, 0.0], // West (-X)
            [1.0, 0.0, 0.0],  // East (+X)
            [0.0, -1.0, 0.0], // Bottom (-Y)
        ];

        for face_idx in 0..6 {
            let i0 = mesh.indices[face_idx * 6] as usize;
            let i1 = mesh.indices[face_idx * 6 + 1] as usize;
            let i2 = mesh.indices[face_idx * 6 + 2] as usize;

            let p0 = mesh.vertices[i0].position;
            let p1 = mesh.vertices[i1].position;
            let p2 = mesh.vertices[i2].position;

            let v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let v2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

            let cross = [
                v1[1] * v2[2] - v1[2] * v2[1],
                v1[2] * v2[0] - v1[0] * v2[2],
                v1[0] * v2[1] - v1[1] * v2[0],
            ];

            let exp = expected_normals[face_idx];
            let dot = cross[0] * exp[0] + cross[1] * exp[1] + cross[2] * exp[2];
            assert!(
                dot > 0.0,
                "Face {} winding normal should point outward (dot = {})",
                face_idx,
                dot
            );
        }
    }

    #[test]
    fn test_leaves_faces_rendered() {
        // In Fancy graphics, leaves render faces between adjacent leaf blocks
        assert!(should_render_face(18, 18));
        assert!(should_render_face(161, 161));
        assert!(!should_render_face(18, 1));
        assert!(should_render_face(18, 0));
    }

    #[test]
    fn test_piston_rotation_table() {
        // Ori 0 (Down): all sides rotated 180 degrees (rot 2)
        assert_eq!(get_piston_side_rot(0, BlockFace::North), 2);
        assert_eq!(get_piston_side_rot(0, BlockFace::South), 2);
        assert_eq!(get_piston_side_rot(0, BlockFace::West), 2);
        assert_eq!(get_piston_side_rot(0, BlockFace::East), 2);

        // Ori 1 (Up): all sides upright (rot 0)
        assert_eq!(get_piston_side_rot(1, BlockFace::North), 0);
        assert_eq!(get_piston_side_rot(1, BlockFace::South), 0);
        assert_eq!(get_piston_side_rot(1, BlockFace::West), 0);
        assert_eq!(get_piston_side_rot(1, BlockFace::East), 0);

        // Ori 2 (North): Top rot 3, Bottom rot 1, West rot 1, East rot 3
        assert_eq!(get_piston_side_rot(2, BlockFace::Top), 3);
        assert_eq!(get_piston_side_rot(2, BlockFace::Bottom), 1);
        assert_eq!(get_piston_side_rot(2, BlockFace::West), 1);
        assert_eq!(get_piston_side_rot(2, BlockFace::East), 3);
    }

    #[test]
    fn test_door_mesh() {
        let mut mesh = MeshData::new();
        let world = World::new();
        let block = Block {
            id: 64, // oak door
            meta: 0, // lower, closed, facing east
            sky_light: 15,
            block_light: 0,
        };
        add_door_quads(&mut mesh, &world, 10, 64, 10, block);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);

        // Thickness should be 3/16 = 0.1875 along X
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        for v in &mesh.vertices {
            min_x = min_x.min(v.position[0]);
            max_x = max_x.max(v.position[0]);
        }
        assert!((max_x - min_x - 0.1875).abs() < 1e-4);

        // Hinge is at min_z (10.0) -> must map to u1_m (hinge side of texture)
        // Handle is at max_z (11.0) -> must map to u0_m (handle side of texture)
        let (u0_m, _, u1_m, _) = crate::texture::get_slot_uv(117);
        for v in &mesh.vertices {
            if (v.position[0] - min_x).abs() < 1e-4 {
                if (v.position[2] - 10.0).abs() < 1e-4 {
                    assert!((v.uv[0] - u1_m).abs() < 1e-4, "Hinge at min_z should use u1");
                } else if (v.position[2] - 11.0).abs() < 1e-4 {
                    assert!((v.uv[0] - u0_m).abs() < 1e-4, "Handle at max_z should use u0");
                }
            }
        }
    }

    #[test]
    fn test_redstone_wire_mesh() {
        let mut mesh = MeshData::new();
        let world = World::new();
        let block = Block {
            id: 55, // redstone wire
            meta: 15, // full power
            sky_light: 15,
            block_light: 0,
        };
        add_redstone_wire_quads(&mut mesh, &world, 10, 64, 10, block);
        // Floor quad: 4 vertices, 6 indices
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices.len(), 6);
        assert_eq!(mesh.vertices[0].color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_extended_piston_base_mesh() {
        let mut mesh = MeshData::new();
        let world = World::new();
        let block = Block {
            id: 33, // normal piston
            meta: 1 | 8, // pointing up, extended
            sky_light: 15,
            block_light: 0,
        };
        add_extended_piston_base(&mut mesh, &world, 10, 64, 10, block);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);

        // Height should be 0.75 for pointing up
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        for v in &mesh.vertices {
            min_y = min_y.min(v.position[1]);
            max_y = max_y.max(v.position[1]);
        }
        assert!((min_y - 64.0).abs() < 1e-4);
        assert!((max_y - 64.75).abs() < 1e-4);
    }

    #[test]
    fn test_piston_extension_mesh() {
        let mut mesh = MeshData::new();
        let world = World::new();
        let block = Block {
            id: 34, // piston extension
            meta: 1, // pointing up, normal head
            sky_light: 15,
            block_light: 0,
        };
        add_piston_extension(&mut mesh, &world, 10, 65, 10, block);
        // Plate (24 verts) + Rod (16 verts, 4 lengthwise faces) = 40 vertices
        assert_eq!(mesh.vertices.len(), 40);
        assert_eq!(mesh.indices.len(), 60);

        // Check head plate bounds: y in [65.75, 66.0]
        let plate_verts = &mesh.vertices[0..24];
        let mut p_min_y = f32::MAX;
        let mut p_max_y = f32::MIN;
        for v in plate_verts {
            p_min_y = p_min_y.min(v.position[1]);
            p_max_y = p_max_y.max(v.position[1]);
        }
        assert!((p_min_y - 65.75).abs() < 1e-4);
        assert!((p_max_y - 66.0).abs() < 1e-4);

        // Check rod bounds: y in [64.75, 65.75], x in [10.375, 10.625], z in [10.375, 10.625]
        let rod_verts = &mesh.vertices[24..40];
        let mut r_min_y = f32::MAX;
        let mut r_max_y = f32::MIN;
        for v in rod_verts {
            r_min_y = r_min_y.min(v.position[1]);
            r_max_y = r_max_y.max(v.position[1]);
            assert!(v.position[0] >= 10.37 && v.position[0] <= 10.63);
            assert!(v.position[2] >= 10.37 && v.position[2] <= 10.63);
        }
        assert!((r_min_y - 64.75).abs() < 1e-4);
        assert!((r_max_y - 65.75).abs() < 1e-4);
    }

    #[test]
    fn test_piston_neighbor_occlusion() {
        // Verify pistons are NOT treated as opaque cubes
        assert!(!is_opaque_cube(33)); // normal piston
        assert!(!is_opaque_cube(29)); // sticky piston
        assert!(!is_opaque_cube(34)); // piston extension
        assert!(!is_opaque_cube(36)); // piston moving

        // Grass (id 2) underneath a piston (id 33) MUST render its top face
        // so extending the piston does not expose a gaping hole into the void
        assert!(should_render_face(2, 33));
        assert!(should_render_face(2, 29));
        assert!(should_render_face(2, 34));
    }

    #[test]
    fn test_pressure_plate_mesh() {
        let mut mesh = MeshData::new();
        let world = World::new();
        let block_unpressed = Block {
            id: 70, // stone pressure plate
            meta: 0,
            sky_light: 15,
            block_light: 0,
        };
        add_pressure_plate_quads(&mut mesh, &world, 10, 64, 10, block_unpressed);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);

        let mut max_y = f32::MIN;
        for v in &mesh.vertices {
            max_y = max_y.max(v.position[1]);
            assert!(v.position[0] >= 10.06 && v.position[0] <= 10.94);
            assert!(v.position[2] >= 10.06 && v.position[2] <= 10.94);
        }
        assert!((max_y - (64.0 + 1.0 / 16.0)).abs() < 1e-4);

        let mut pressed_mesh = MeshData::new();
        let block_pressed = Block {
            id: 72, // oak pressure plate
            meta: 1, // pressed
            sky_light: 15,
            block_light: 0,
        };
        add_pressure_plate_quads(&mut pressed_mesh, &world, 10, 64, 10, block_pressed);
        let mut pressed_max_y = f32::MIN;
        for v in &pressed_mesh.vertices {
            pressed_max_y = pressed_max_y.max(v.position[1]);
        }
        assert!((pressed_max_y - (64.0 + 0.5 / 16.0)).abs() < 1e-4);
    }

    #[test]
    fn test_button_mesh() {
        let mut mesh = MeshData::new();
        let world = World::new();
        let block = Block {
            id: 77, // stone button
            meta: 1, // facing west (mounted on east of block)
            sky_light: 15,
            block_light: 0,
        };
        add_button_quads(&mut mesh, &world, 10, 64, 10, block);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        for v in &mesh.vertices {
            min_x = min_x.min(v.position[0]);
            max_x = max_x.max(v.position[0]);
        }
        assert!((max_x - min_x - 2.0 / 16.0).abs() < 1e-4);
    }

    #[test]
    fn test_lever_mesh() {
        let world = World::new();
        // Floor lever (meta 5): off points North (Z < 10.5), on points South (Z > 10.5)
        let mut mesh_off = MeshData::new();
        add_lever_quads(&mut mesh_off, &world, 10, 64, 10, Block { id: 69, meta: 5, sky_light: 15, block_light: 0 });
        assert_eq!(mesh_off.vertices.len(), 48);
        assert_eq!(mesh_off.indices.len(), 72);
        let min_z_off = mesh_off.vertices.iter().map(|v| v.position[2]).fold(f32::INFINITY, f32::min);
        assert!(min_z_off < 10.5, "Floor lever OFF must point North (min Z < 10.5)");

        let mut mesh_on = MeshData::new();
        add_lever_quads(&mut mesh_on, &world, 10, 64, 10, Block { id: 69, meta: 13, sky_light: 15, block_light: 0 });
        let max_z_on = mesh_on.vertices.iter().map(|v| v.position[2]).fold(f32::NEG_INFINITY, f32::max);
        assert!(max_z_on > 10.5, "Floor lever ON must point South (max Z > 10.5)");

        // Wall lever on West wall (meta 1): off points UP (Y > 64.5), on points DOWN (Y < 64.5)
        let mut mesh_wall_off = MeshData::new();
        add_lever_quads(&mut mesh_wall_off, &world, 10, 64, 10, Block { id: 69, meta: 1, sky_light: 15, block_light: 0 });
        let max_y_off = mesh_wall_off.vertices.iter().map(|v| v.position[1]).fold(f32::NEG_INFINITY, f32::max);
        assert!(max_y_off > 64.5, "Wall lever OFF must point UP (max Y > 64.5)");

        let mut mesh_wall_on = MeshData::new();
        add_lever_quads(&mut mesh_wall_on, &world, 10, 64, 10, Block { id: 69, meta: 9, sky_light: 15, block_light: 0 });
        let min_y_on = mesh_wall_on.vertices.iter().map(|v| v.position[1]).fold(f32::INFINITY, f32::min);
        assert!(min_y_on < 64.5, "Wall lever ON must point DOWN (min Y < 64.5)");
    }

    #[test]
    fn test_stairs_mesh() {
        let mut mesh = MeshData::new();
        let world = World::new();
        let block = Block {
            id: 53, // oak stairs
            meta: 0, // east facing, upright
            sky_light: 15,
            block_light: 0,
        };
        add_stairs_quads(&mut mesh, &world, 10, 64, 10, block);
        // Base half-slab (24) + step quarter-box (24) = 48 vertices
        assert_eq!(mesh.vertices.len(), 48);
        assert_eq!(mesh.indices.len(), 72);
    }

    #[test]
    fn test_slab_mesh() {
        let mut mesh = MeshData::new();
        let world = World::new();
        let bottom_slab = Block {
            id: 44, // stone slab
            meta: 0, // bottom half
            sky_light: 15,
            block_light: 0,
        };
        add_slab_quads(&mut mesh, &world, 10, 64, 10, bottom_slab);
        assert_eq!(mesh.vertices.len(), 24);
        let mut max_y = f32::MIN;
        for v in &mesh.vertices {
            max_y = max_y.max(v.position[1]);
        }
        assert!((max_y - 64.5).abs() < 1e-4);

        let mut top_mesh = MeshData::new();
        let top_slab = Block {
            id: 44,
            meta: 8, // top half
            sky_light: 15,
            block_light: 0,
        };
        add_slab_quads(&mut top_mesh, &world, 10, 64, 10, top_slab);
        let mut min_y = f32::MAX;
        for v in &top_mesh.vertices {
            min_y = min_y.min(v.position[1]);
        }
        assert!((min_y - 64.5).abs() < 1e-4);
    }

    #[test]
    fn test_fence_mesh() {
        let mut mesh = MeshData::new();
        let world = World::new();
        let block = Block {
            id: 85, // oak fence
            meta: 0,
            sky_light: 15,
            block_light: 0,
        };
        add_fence_quads(&mut mesh, &world, 10, 64, 10, block);
        // Post only (no neighbors) = 24 vertices
        assert_eq!(mesh.vertices.len(), 24);
    }

    #[test]
    fn test_pane_mesh() {
        let mut mesh = MeshData::new();
        let world = World::new();
        let block = Block {
            id: 102, // glass pane
            meta: 0,
            sky_light: 15,
            block_light: 0,
        };
        add_pane_quads(&mut mesh, &world, 10, 64, 10, block);
        // Isolated pane draws central post + cross arms
        assert!(mesh.vertices.len() >= 24);
    }

    #[test]
    fn test_bed_mesh() {
        let mut mesh = MeshData::new();
        let mut world = World::new();
        // Bed head at (10, 64, 11), facing 0 (South), head meta = 8
        // Bed foot at (10, 64, 10), facing 0 (South), foot meta = 0
        let head_block = Block { id: 26, meta: 8, sky_light: 15, block_light: 0 };
        let foot_block = Block { id: 26, meta: 0, sky_light: 15, block_light: 0 };
        world.set_block(10, 64, 11, head_block);
        world.set_block(10, 64, 10, foot_block);

        add_bed_quads(&mut mesh, &world, 10, 64, 11, head_block);
        // Head block has 4 visible faces: Top, Head End (South), West, East.
        // Connecting face to foot (North) is culled because foot block is adjacent.
        // Bottom is culled if opaque below or 4 verts if not.
        // 4 faces * 4 verts = 16 or 20 verts.
        assert!(mesh.vertices.len() >= 16);

        // Verify bed height is 9/16 = 0.5625
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        for v in &mesh.vertices {
            min_y = min_y.min(v.position[1]);
            max_y = max_y.max(v.position[1]);
        }
        assert!((min_y - 64.0).abs() < 1e-4);
        assert!((max_y - (64.0 + 9.0 / 16.0)).abs() < 1e-4);
    }

    #[test]
    fn test_piston_moving_rendered_as_piston() {
        let mut chunk = ChunkColumn::new(0, 0);
        chunk.sections[4] = Some(Box::new(crate::world::ChunkSection::new()));
        // Place moving piston (ID 36) in chunk
        chunk.sections[4].as_mut().unwrap().set_block(5, 0, 5, Block { id: 36, meta: 0, sky_light: 15, block_light: 0 });
        let world = World::new();
        let mesh = mesh_chunk_column(&chunk, &world);
        // Block 36 must render as piston (6 faces, 24 vertices, 36 indices) without disappearing or showing cobblestone
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
    }

    #[test]
    fn test_stained_glass_culling() {
        // Adjacent stained glass blocks cull faces between each other
        assert!(!should_render_face(95, 95));
        assert!(!should_render_face(95, 20));
        assert!(!should_render_face(20, 95));
        // Stained glass adjacent to air renders its face
        assert!(should_render_face(95, 0));
        // Stained glass against solid stone is culled (stone covers it)
        assert!(!should_render_face(95, 1));
        // Stone against stained glass renders its face (visible through glass)
        assert!(should_render_face(1, 95));
    }
}

