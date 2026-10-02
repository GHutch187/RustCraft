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
    pub end_portal_vertices: Vec<Vertex>,
    pub end_portal_indices: Vec<u32>,
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(2048),
            indices: Vec::with_capacity(3072),
            transparent_vertices: Vec::with_capacity(512),
            transparent_indices: Vec::with_capacity(768),
            end_portal_vertices: Vec::new(),
            end_portal_indices: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty() && self.transparent_indices.is_empty() && self.end_portal_indices.is_empty()
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
        | 92 | 93 | 94 | 95 | 96 | 101 | 102 | 104 | 105 | 106 | 107 | 108 | 109 | 111 | 113 | 114
        | 115 | 116 | 117 | 118 | 119 | 120 | 122 | 126 | 127 | 128 | 130 | 131 | 132 | 134 | 135
        | 136 | 138 | 139 | 140 | 141 | 142 | 143 | 144 | 145 | 146 | 147 | 148 | 149 | 150 | 151
        | 154 | 156 | 157 | 160 | 161 | 163 | 164 | 165 | 167 | 171 | 175 | 178 => false,
        _ => true,
    }
}

pub fn is_cross_plant(id: u16) -> bool {
    matches!(id, 6 | 30 | 31 | 32 | 37 | 38 | 39 | 40 | 59 | 83 | 104 | 105 | 115 | 141 | 142)
}

pub fn is_torch(id: u16) -> bool {
    matches!(id, 50 | 75 | 76)
}

pub fn is_flat_floor(id: u16) -> bool {
    matches!(id, 111 | 171)
}

pub fn is_rail(id: u16) -> bool {
    matches!(id, 27 | 28 | 66 | 157)
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

pub fn can_pane_connect(neighbor_id: u16) -> bool {
    is_opaque_cube(neighbor_id) || is_pane(neighbor_id) || neighbor_id == 20 || neighbor_id == 95
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
    if block_id == 20 && neighbor_id == 20 {
        return false;
    }
    if block_id == 79 && neighbor_id == 79 {
        return false;
    }
    if block_id == 95 && neighbor_id == 95 {
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
    slots: [u16; 6], // Top, Bottom, North, South, West, East
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
    slots: [u16; 6],
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
    slots: [u16; 6],
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

    let mut slots = [113u16; 6];
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

    let mut plate_slots = [113u16; 6];
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

    let (side_slot, tint) = crate::texture::get_block_slot(block.id, block.meta, BlockFace::North);
    let top_slot = crate::texture::get_pane_top_slot(block.id, block.meta);

    let (su0, sv0, su1, sv1) = crate::texture::get_slot_uv(side_slot);
    let sdu = su1 - su0;

    let (tu0, tv0, tu1, tv1) = crate::texture::get_slot_uv(top_slot);
    let tdu = tu1 - tu0;
    let tdv = tv1 - tv0;

    let id_north = world.get_block(wx, wy, wz - 1).id;
    let id_south = world.get_block(wx, wy, wz + 1).id;
    let id_west = world.get_block(wx - 1, wy, wz).id;
    let id_east = world.get_block(wx + 1, wy, wz).id;

    let mut connect_n = can_pane_connect(id_north);
    let mut connect_s = can_pane_connect(id_south);
    let mut connect_w = can_pane_connect(id_west);
    let mut connect_e = can_pane_connect(id_east);

    if !connect_n && !connect_s && !connect_w && !connect_e {
        connect_n = true;
        connect_s = true;
        connect_w = true;
        connect_e = true;
    }

    let is_transparent = block.id == 160;

    let mut push_quad = |verts: [[f32; 3]; 4], uvs: [[f32; 2]; 4], shade: f32| {
        let col = match tint {
            Some(t) => [t[0] * shade, t[1] * shade, t[2] * shade, 1.0],
            None => [shade, shade, shade, 1.0],
        };
        if is_transparent {
            let start = mesh.transparent_vertices.len() as u32;
            for i in 0..4 {
                mesh.transparent_vertices.push(Vertex {
                    position: verts[i],
                    uv: uvs[i],
                    color: col,
                    normal: light,
                });
            }
            mesh.transparent_indices.extend_from_slice(&[
                start, start + 1, start + 2,
                start, start + 2, start + 3,
            ]);
        } else {
            let start = mesh.vertices.len() as u32;
            for i in 0..4 {
                mesh.vertices.push(Vertex {
                    position: verts[i],
                    uv: uvs[i],
                    color: col,
                    normal: light,
                });
            }
            mesh.indices.extend_from_slice(&[
                start, start + 1, start + 2,
                start, start + 2, start + 3,
            ]);
        }
    };

    let x_min = fx + 7.0 / 16.0;
    let x_max = fx + 9.0 / 16.0;
    let z_min = fz + 7.0 / 16.0;
    let z_max = fz + 9.0 / 16.0;
    let x_mid = fx + 0.5;
    let z_mid = fz + 0.5;
    let y0 = fy;
    let y1 = fy + 1.0;

    let tu_min = tu0 + (7.0 / 16.0) * tdu;
    let tu_max = tu0 + (9.0 / 16.0) * tdu;

    let id_above = above.id;
    let id_below = world.get_block(wx, wy - 1, wz).id;
    let render_top = !is_pane(id_above) && !is_opaque_cube(id_above) && id_above != 20 && id_above != 95;
    let render_bottom = !is_pane(id_below) && !is_opaque_cube(id_below) && id_below != 20 && id_below != 95;

    // North arm
    if connect_n {
        // West face (-X)
        push_quad(
            [
                [x_min, y1, fz],
                [x_min, y0, fz],
                [x_min, y0, z_mid],
                [x_min, y1, z_mid],
            ],
            [
                [su0, sv0],
                [su0, sv1],
                [su0 + 0.5 * sdu, sv1],
                [su0 + 0.5 * sdu, sv0],
            ],
            0.6,
        );
        // East face (+X)
        push_quad(
            [
                [x_max, y1, z_mid],
                [x_max, y0, z_mid],
                [x_max, y0, fz],
                [x_max, y1, fz],
            ],
            [
                [su0 + 0.5 * sdu, sv0],
                [su0 + 0.5 * sdu, sv1],
                [su0, sv1],
                [su0, sv0],
            ],
            0.6,
        );
        // Top (+Y)
        if render_top {
            push_quad(
                [
                    [x_min, y1, z_mid],
                    [x_max, y1, z_mid],
                    [x_max, y1, fz],
                    [x_min, y1, fz],
                ],
                [
                    [tu_min, tv0 + 0.5 * tdv],
                    [tu_max, tv0 + 0.5 * tdv],
                    [tu_max, tv0],
                    [tu_min, tv0],
                ],
                1.0,
            );
        }
        // Bottom (-Y)
        if render_bottom {
            push_quad(
                [
                    [x_min, y0, fz],
                    [x_max, y0, fz],
                    [x_max, y0, z_mid],
                    [x_min, y0, z_mid],
                ],
                [
                    [tu_min, tv0],
                    [tu_max, tv0],
                    [tu_max, tv0 + 0.5 * tdv],
                    [tu_min, tv0 + 0.5 * tdv],
                ],
                0.5,
            );
        }
        // North end cap (-Z)
        if !is_opaque_cube(id_north) && id_north != 20 && id_north != 95 && !is_pane(id_north) {
            push_quad(
                [
                    [x_max, y1, fz],
                    [x_max, y0, fz],
                    [x_min, y0, fz],
                    [x_min, y1, fz],
                ],
                [
                    [tu_max, tv0],
                    [tu_max, tv1],
                    [tu_min, tv1],
                    [tu_min, tv0],
                ],
                0.8,
            );
        }
        // South face of North arm at z_mid (if disconnected on other 3 sides)
        if !connect_s && !connect_w && !connect_e {
            push_quad(
                [
                    [x_min, y1, z_mid],
                    [x_min, y0, z_mid],
                    [x_max, y0, z_mid],
                    [x_max, y1, z_mid],
                ],
                [
                    [tu_min, tv0],
                    [tu_min, tv1],
                    [tu_max, tv1],
                    [tu_max, tv0],
                ],
                0.8,
            );
        }
    }

    // South arm
    if connect_s {
        // West face (-X)
        push_quad(
            [
                [x_min, y1, z_mid],
                [x_min, y0, z_mid],
                [x_min, y0, fz + 1.0],
                [x_min, y1, fz + 1.0],
            ],
            [
                [su0 + 0.5 * sdu, sv0],
                [su0 + 0.5 * sdu, sv1],
                [su1, sv1],
                [su1, sv0],
            ],
            0.6,
        );
        // East face (+X)
        push_quad(
            [
                [x_max, y1, fz + 1.0],
                [x_max, y0, fz + 1.0],
                [x_max, y0, z_mid],
                [x_max, y1, z_mid],
            ],
            [
                [su1, sv0],
                [su1, sv1],
                [su0 + 0.5 * sdu, sv1],
                [su0 + 0.5 * sdu, sv0],
            ],
            0.6,
        );
        // Top (+Y)
        if render_top {
            push_quad(
                [
                    [x_min, y1, fz + 1.0],
                    [x_max, y1, fz + 1.0],
                    [x_max, y1, z_mid],
                    [x_min, y1, z_mid],
                ],
                [
                    [tu_min, tv1],
                    [tu_max, tv1],
                    [tu_max, tv0 + 0.5 * tdv],
                    [tu_min, tv0 + 0.5 * tdv],
                ],
                1.0,
            );
        }
        // Bottom (-Y)
        if render_bottom {
            push_quad(
                [
                    [x_min, y0, z_mid],
                    [x_max, y0, z_mid],
                    [x_max, y0, fz + 1.0],
                    [x_min, y0, fz + 1.0],
                ],
                [
                    [tu_min, tv0 + 0.5 * tdv],
                    [tu_max, tv0 + 0.5 * tdv],
                    [tu_max, tv1],
                    [tu_min, tv1],
                ],
                0.5,
            );
        }
        // South end cap (+Z)
        if !is_opaque_cube(id_south) && id_south != 20 && id_south != 95 && !is_pane(id_south) {
            push_quad(
                [
                    [x_min, y1, fz + 1.0],
                    [x_min, y0, fz + 1.0],
                    [x_max, y0, fz + 1.0],
                    [x_max, y1, fz + 1.0],
                ],
                [
                    [tu_min, tv0],
                    [tu_min, tv1],
                    [tu_max, tv1],
                    [tu_max, tv0],
                ],
                0.8,
            );
        }
        // North face of South arm at z_mid (if disconnected on other 3 sides)
        if !connect_n && !connect_w && !connect_e {
            push_quad(
                [
                    [x_max, y1, z_mid],
                    [x_max, y0, z_mid],
                    [x_min, y0, z_mid],
                    [x_min, y1, z_mid],
                ],
                [
                    [tu_max, tv0],
                    [tu_max, tv1],
                    [tu_min, tv1],
                    [tu_min, tv0],
                ],
                0.8,
            );
        }
    }

    // West arm
    if connect_w {
        // North face (-Z)
        push_quad(
            [
                [x_mid, y1, z_min],
                [x_mid, y0, z_min],
                [fx, y0, z_min],
                [fx, y1, z_min],
            ],
            [
                [su0 + 0.5 * sdu, sv0],
                [su0 + 0.5 * sdu, sv1],
                [su0, sv1],
                [su0, sv0],
            ],
            0.8,
        );
        // South face (+Z)
        push_quad(
            [
                [fx, y1, z_max],
                [fx, y0, z_max],
                [x_mid, y0, z_max],
                [x_mid, y1, z_max],
            ],
            [
                [su0, sv0],
                [su0, sv1],
                [su0 + 0.5 * sdu, sv1],
                [su0 + 0.5 * sdu, sv0],
            ],
            0.8,
        );
        // Top (+Y)
        if render_top {
            let wx_end = if connect_n || connect_s { x_min } else { x_mid };
            push_quad(
                [
                    [fx, y1, z_max],
                    [wx_end, y1, z_max],
                    [wx_end, y1, z_min],
                    [fx, y1, z_min],
                ],
                [
                    [tu_min, tv0],
                    [tu_max, tv0],
                    [tu_max, tv0 + 0.5 * tdv],
                    [tu_min, tv0 + 0.5 * tdv],
                ],
                1.0,
            );
        }
        // Bottom (-Y)
        if render_bottom {
            let wx_end = if connect_n || connect_s { x_min } else { x_mid };
            push_quad(
                [
                    [fx, y0, z_min],
                    [wx_end, y0, z_min],
                    [wx_end, y0, z_max],
                    [fx, y0, z_max],
                ],
                [
                    [tu_min, tv0 + 0.5 * tdv],
                    [tu_max, tv0 + 0.5 * tdv],
                    [tu_max, tv0],
                    [tu_min, tv0],
                ],
                0.5,
            );
        }
        // West end cap (-X)
        if !is_opaque_cube(id_west) && id_west != 20 && id_west != 95 && !is_pane(id_west) {
            push_quad(
                [
                    [fx, y1, z_min],
                    [fx, y0, z_min],
                    [fx, y0, z_max],
                    [fx, y1, z_max],
                ],
                [
                    [tu_min, tv0],
                    [tu_min, tv1],
                    [tu_max, tv1],
                    [tu_max, tv0],
                ],
                0.6,
            );
        }
        // East face of West arm at x_mid (if disconnected on other 3 sides)
        if !connect_e && !connect_n && !connect_s {
            push_quad(
                [
                    [x_mid, y1, z_max],
                    [x_mid, y0, z_max],
                    [x_mid, y0, z_min],
                    [x_mid, y1, z_min],
                ],
                [
                    [tu_min, tv0],
                    [tu_min, tv1],
                    [tu_max, tv1],
                    [tu_max, tv0],
                ],
                0.6,
            );
        }
    }

    // East arm
    if connect_e {
        // North face (-Z)
        push_quad(
            [
                [fx + 1.0, y1, z_min],
                [fx + 1.0, y0, z_min],
                [x_mid, y0, z_min],
                [x_mid, y1, z_min],
            ],
            [
                [su1, sv0],
                [su1, sv1],
                [su0 + 0.5 * sdu, sv1],
                [su0 + 0.5 * sdu, sv0],
            ],
            0.8,
        );
        // South face (+Z)
        push_quad(
            [
                [x_mid, y1, z_max],
                [x_mid, y0, z_max],
                [fx + 1.0, y0, z_max],
                [fx + 1.0, y1, z_max],
            ],
            [
                [su0 + 0.5 * sdu, sv0],
                [su0 + 0.5 * sdu, sv1],
                [su1, sv1],
                [su1, sv0],
            ],
            0.8,
        );
        // Top (+Y)
        if render_top {
            let ex_start = if connect_n || connect_s { x_max } else { x_mid };
            push_quad(
                [
                    [ex_start, y1, z_max],
                    [fx + 1.0, y1, z_max],
                    [fx + 1.0, y1, z_min],
                    [ex_start, y1, z_min],
                ],
                [
                    [tu_min, tv0 + 0.5 * tdv],
                    [tu_max, tv0 + 0.5 * tdv],
                    [tu_max, tv1],
                    [tu_min, tv1],
                ],
                1.0,
            );
        }
        // Bottom (-Y)
        if render_bottom {
            let ex_start = if connect_n || connect_s { x_max } else { x_mid };
            push_quad(
                [
                    [ex_start, y0, z_min],
                    [fx + 1.0, y0, z_min],
                    [fx + 1.0, y0, z_max],
                    [ex_start, y0, z_max],
                ],
                [
                    [tu_min, tv1],
                    [tu_max, tv1],
                    [tu_max, tv0 + 0.5 * tdv],
                    [tu_min, tv0 + 0.5 * tdv],
                ],
                0.5,
            );
        }
        // East end cap (+X)
        if !is_opaque_cube(id_east) && id_east != 20 && id_east != 95 && !is_pane(id_east) {
            push_quad(
                [
                    [fx + 1.0, y1, z_max],
                    [fx + 1.0, y0, z_max],
                    [fx + 1.0, y0, z_min],
                    [fx + 1.0, y1, z_min],
                ],
                [
                    [tu_min, tv0],
                    [tu_min, tv1],
                    [tu_max, tv1],
                    [tu_max, tv0],
                ],
                0.6,
            );
        }
        // West face of East arm at x_mid (if disconnected on other 3 sides)
        if !connect_w && !connect_n && !connect_s {
            push_quad(
                [
                    [x_mid, y1, z_min],
                    [x_mid, y0, z_min],
                    [x_mid, y0, z_max],
                    [x_mid, y1, z_max],
                ],
                [
                    [tu_max, tv0],
                    [tu_max, tv1],
                    [tu_min, tv1],
                    [tu_min, tv0],
                ],
                0.6,
            );
        }
    }
}

fn add_fire_quads(mesh: &mut MeshData, wx: i32, wy: i32, wz: i32, block: Block) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let (slot, _) = crate::texture::get_block_slot(51, block.meta, BlockFace::North);
    let (u0, v0, u1, v1) = crate::texture::get_slot_uv(slot);

    let color = [1.0, 1.0, 1.0, 1.0];
    let normal = [1.0, 1.0, 1.0];

    let x0 = fx;
    let x1 = fx + 1.0;
    let z0 = fz;
    let z1 = fz + 1.0;
    let y0 = fy;
    let y1 = fy + 1.25;

    let planes = [
        // Diagonal 1
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
        // Diagonal 2
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
        // Lateral X
        [
            ([fx + 0.5, y0, z0], [u0, v1]),
            ([fx + 0.5, y0, z1], [u1, v1]),
            ([fx + 0.5, y1, z1], [u1, v0]),
            ([fx + 0.5, y1, z0], [u0, v0]),
        ],
        [
            ([fx + 0.5, y0, z1], [u0, v1]),
            ([fx + 0.5, y0, z0], [u1, v1]),
            ([fx + 0.5, y1, z0], [u1, v0]),
            ([fx + 0.5, y1, z1], [u0, v0]),
        ],
        // Lateral Z
        [
            ([x0, y0, fz + 0.5], [u0, v1]),
            ([x1, y0, fz + 0.5], [u1, v1]),
            ([x1, y1, fz + 0.5], [u1, v0]),
            ([x0, y1, fz + 0.5], [u0, v0]),
        ],
        [
            ([x1, y0, fz + 0.5], [u0, v1]),
            ([x0, y0, fz + 0.5], [u1, v1]),
            ([x0, y1, fz + 0.5], [u1, v0]),
            ([x1, y1, fz + 0.5], [u0, v0]),
        ],
    ];

    for quad in planes {
        let start_idx = mesh.vertices.len() as u32;
        for (pos, uv) in quad {
            mesh.vertices.push(Vertex {
                position: pos,
                uv,
                color,
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

fn add_nether_portal_quads(
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

    let (slot, _) = crate::texture::get_block_slot(90, block.meta, BlockFace::North);
    let (u0, v0, u1, v1) = crate::texture::get_slot_uv(slot);

    let color = [1.0, 1.0, 1.0, 1.0];
    let normal = [1.0, 1.0, 1.0];

    let connect_x = world.get_block(wx - 1, wy, wz).id == 90 || world.get_block(wx + 1, wy, wz).id == 90;
    let connect_z = world.get_block(wx, wy, wz - 1).id == 90 || world.get_block(wx, wy, wz + 1).id == 90;

    let along_x = if connect_x && !connect_z {
        true
    } else if connect_z && !connect_x {
        false
    } else {
        block.meta != 2
    };

    let quads = if along_x {
        let z = fz + 0.5;
        [
            // Front face (+Z)
            [
                ([fx, fy, z], [u0, v1]),
                ([fx + 1.0, fy, z], [u1, v1]),
                ([fx + 1.0, fy + 1.0, z], [u1, v0]),
                ([fx, fy + 1.0, z], [u0, v0]),
            ],
            // Back face (-Z)
            [
                ([fx + 1.0, fy, z], [u0, v1]),
                ([fx, fy, z], [u1, v1]),
                ([fx, fy + 1.0, z], [u1, v0]),
                ([fx + 1.0, fy + 1.0, z], [u0, v0]),
            ],
        ]
    } else {
        let x = fx + 0.5;
        [
            // Front face (+X)
            [
                ([x, fy, fz + 1.0], [u0, v1]),
                ([x, fy, fz], [u1, v1]),
                ([x, fy + 1.0, fz], [u1, v0]),
                ([x, fy + 1.0, fz + 1.0], [u0, v0]),
            ],
            // Back face (-X)
            [
                ([x, fy, fz], [u0, v1]),
                ([x, fy, fz + 1.0], [u1, v1]),
                ([x, fy + 1.0, fz + 1.0], [u1, v0]),
                ([x, fy + 1.0, fz], [u0, v0]),
            ],
        ]
    };

    for quad in quads {
        let start_idx = mesh.transparent_vertices.len() as u32;
        for (pos, uv) in quad {
            mesh.transparent_vertices.push(Vertex {
                position: pos,
                uv,
                color,
                normal,
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
    }
}

fn add_ladder_quads(mesh: &mut MeshData, wx: i32, wy: i32, wz: i32, block: Block) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let (slot, _) = crate::texture::get_block_slot(65, block.meta, BlockFace::North);
    let (u0, v0, u1, v1) = crate::texture::get_slot_uv(slot);

    let color = [1.0, 1.0, 1.0, 1.0];
    let normal = [
        (block.sky_light as f32 / 15.0).clamp(0.0, 1.0),
        (block.block_light as f32 / 15.0).clamp(0.0, 1.0),
        1.0,
    ];

    let d = 0.05f32; // Inset from wall matching Minecraft 1.7.10

    let quad = match block.meta {
        2 => [
            [fx + 1.0, fy, fz + 1.0 - d],
            [fx, fy, fz + 1.0 - d],
            [fx, fy + 1.0, fz + 1.0 - d],
            [fx + 1.0, fy + 1.0, fz + 1.0 - d],
        ],
        3 => [
            [fx, fy, fz + d],
            [fx + 1.0, fy, fz + d],
            [fx + 1.0, fy + 1.0, fz + d],
            [fx, fy + 1.0, fz + d],
        ],
        4 => [
            [fx + 1.0 - d, fy, fz],
            [fx + 1.0 - d, fy, fz + 1.0],
            [fx + 1.0 - d, fy + 1.0, fz + 1.0],
            [fx + 1.0 - d, fy + 1.0, fz],
        ],
        5 => [
            [fx + d, fy, fz + 1.0],
            [fx + d, fy, fz],
            [fx + d, fy + 1.0, fz],
            [fx + d, fy + 1.0, fz + 1.0],
        ],
        _ => [
            [fx + 1.0, fy, fz + 1.0 - d],
            [fx, fy, fz + 1.0 - d],
            [fx, fy + 1.0, fz + 1.0 - d],
            [fx + 1.0, fy + 1.0, fz + 1.0 - d],
        ],
    };

    let uvs = [
        [u0, v1],
        [u1, v1],
        [u1, v0],
        [u0, v0],
    ];

    let start_idx = mesh.vertices.len() as u32;
    for i in 0..4 {
        mesh.vertices.push(Vertex {
            position: quad[i],
            uv: uvs[i],
            color,
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
    mesh.indices.extend_from_slice(&[
        start_idx,
        start_idx + 2,
        start_idx + 1,
        start_idx,
        start_idx + 3,
        start_idx + 2,
    ]);
}

fn add_cactus_quads(
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

    let (top_slot, _) = crate::texture::get_block_slot(81, block.meta, BlockFace::Top);
    let (bottom_slot, _) = crate::texture::get_block_slot(81, block.meta, BlockFace::Bottom);
    let (side_slot, _) = crate::texture::get_block_slot(81, block.meta, BlockFace::North);

    let (tu0, tv0, tu1, tv1) = crate::texture::get_slot_uv(top_slot);
    let (bu0, bv0, bu1, bv1) = crate::texture::get_slot_uv(bottom_slot);
    let (su0, sv0, su1, sv1) = crate::texture::get_slot_uv(side_slot);

    let color = [1.0, 1.0, 1.0, 1.0];
    let normal = [
        (block.sky_light as f32 / 15.0).clamp(0.0, 1.0),
        (block.block_light as f32 / 15.0).clamp(0.0, 1.0),
        1.0,
    ];

    let mut faces: Vec<([[f32; 3]; 4], [[f32; 2]; 4], f32)> = Vec::with_capacity(6);

    // North face: inset on Z to fz + 1/16, spans full x in [fx, fx + 1.0] so thorns stick out!
    let z_north = fz + 1.0 / 16.0;
    faces.push((
        [[fx + 1.0, fy + 1.0, z_north], [fx + 1.0, fy, z_north], [fx, fy, z_north], [fx, fy + 1.0, z_north]],
        [[su1, sv0], [su1, sv1], [su0, sv1], [su0, sv0]],
        0.8,
    ));

    // South face: inset on Z to fz + 15/16, spans full x in [fx, fx + 1.0]
    let z_south = fz + 15.0 / 16.0;
    faces.push((
        [[fx, fy + 1.0, z_south], [fx, fy, z_south], [fx + 1.0, fy, z_south], [fx + 1.0, fy + 1.0, z_south]],
        [[su0, sv0], [su0, sv1], [su1, sv1], [su1, sv0]],
        0.8,
    ));

    // West face: inset on X to fx + 1/16, spans full z in [fz, fz + 1.0] so thorns stick out!
    let x_west = fx + 1.0 / 16.0;
    faces.push((
        [[x_west, fy + 1.0, fz], [x_west, fy, fz], [x_west, fy, fz + 1.0], [x_west, fy + 1.0, fz + 1.0]],
        [[su0, sv0], [su0, sv1], [su1, sv1], [su1, sv0]],
        0.6,
    ));

    // East face: inset on X to fx + 15/16, spans full z in [fz, fz + 1.0]
    let x_east = fx + 15.0 / 16.0;
    faces.push((
        [[x_east, fy + 1.0, fz + 1.0], [x_east, fy, fz + 1.0], [x_east, fy, fz], [x_east, fy + 1.0, fz]],
        [[su0, sv0], [su0, sv1], [su1, sv1], [su1, sv0]],
        0.6,
    ));

    // Top face
    if world.get_block(wx, wy + 1, wz).id != 81 {
        faces.push((
            [[fx, fy + 1.0, fz + 1.0], [fx + 1.0, fy + 1.0, fz + 1.0], [fx + 1.0, fy + 1.0, fz], [fx, fy + 1.0, fz]],
            [[tu0, tv1], [tu1, tv1], [tu1, tv0], [tu0, tv0]],
            1.0,
        ));
    }

    // Bottom face
    if world.get_block(wx, wy - 1, wz).id != 81 {
        faces.push((
            [[fx, fy, fz], [fx + 1.0, fy, fz], [fx + 1.0, fy, fz + 1.0], [fx, fy, fz + 1.0]],
            [[bu0, bv0], [bu1, bv0], [bu1, bv1], [bu0, bv1]],
            0.5,
        ));
    }

    for (pos, uv, shade) in faces {
        let face_col = [color[0] * shade, color[1] * shade, color[2] * shade, 1.0];
        let start_idx = mesh.vertices.len() as u32;
        for i in 0..4 {
            mesh.vertices.push(Vertex {
                position: pos[i],
                uv: uv[i],
                color: face_col,
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

fn add_anvil_quads(
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

    let base_slots = [246u16; 6];
    let damage = ((block.meta >> 2) & 3).min(2) as u16;
    let top_head_slot = 247 + damage;

    let is_x_axis = (block.meta & 1) != 0;

    // 1. Base plate: [2, 0, 2] to [14, 4, 14]
    let base_min = [fx + 2.0 / 16.0, fy, fz + 2.0 / 16.0];
    let base_max = [fx + 14.0 / 16.0, fy + 4.0 / 16.0, fz + 14.0 / 16.0];
    add_sub_box_proportional(mesh, base_min, base_max, [fx, fy, fz], base_slots, light);

    if is_x_axis {
        // 2. Lower narrow step: [3, 4, 4] to [13, 5, 12]
        let step_min = [fx + 3.0 / 16.0, fy + 4.0 / 16.0, fz + 4.0 / 16.0];
        let step_max = [fx + 13.0 / 16.0, fy + 5.0 / 16.0, fz + 12.0 / 16.0];
        add_sub_box_proportional(mesh, step_min, step_max, [fx, fy, fz], base_slots, light);

        // 3. Waist beneath top: [4, 5, 6] to [12, 10, 10]
        let mid_min = [fx + 4.0 / 16.0, fy + 5.0 / 16.0, fz + 6.0 / 16.0];
        let mid_max = [fx + 12.0 / 16.0, fy + 10.0 / 16.0, fz + 10.0 / 16.0];
        add_sub_box_proportional(mesh, mid_min, mid_max, [fx, fy, fz], base_slots, light);

        // 4. Anvil top: [0, 10, 3] to [16, 16, 13]
        let head_min = [fx, fy + 10.0 / 16.0, fz + 3.0 / 16.0];
        let head_max = [fx + 1.0, fy + 1.0, fz + 13.0 / 16.0];
        // Custom top UV: map only opaque columns 3..13 across Z, and full length across X
        let (tu0, tv0, tu1, tv1) = crate::texture::get_slot_uv(top_head_slot);
        let tdu = tu1 - tu0;
        let u_min = tu0 + tdu * (3.0 / 16.0);
        let u_max = tu0 + tdu * (13.0 / 16.0);
        let top_uvs = [
            [u_max, tv0],
            [u_max, tv1],
            [u_min, tv1],
            [u_min, tv0],
        ];
        let (bu0, bv0, bu1, bv1) = crate::texture::get_slot_uv(246);
        let dbu = bu1 - bu0;
        let dbv = bv1 - bv0;
        let v_top = bv0;
        let v_bot = bv0 + dbv * (6.0 / 16.0);
        let uv16_north = [[bu1, v_top], [bu1, v_bot], [bu0, v_bot], [bu0, v_top]];
        let uv16_south = [[bu0, v_top], [bu0, v_bot], [bu1, v_bot], [bu1, v_top]];
        let uv10_end = [
            [bu0 + dbu * (3.0 / 16.0), v_top],
            [bu0 + dbu * (3.0 / 16.0), v_bot],
            [bu0 + dbu * (13.0 / 16.0), v_bot],
            [bu0 + dbu * (13.0 / 16.0), v_top],
        ];

        let head_slots = [top_head_slot, 246, 246, 246, 246, 246];
        let mut custom = [[[0.0f32; 2]; 4]; 6];
        custom[0] = top_uvs;
        custom[1] = [[bu0, bv0], [bu1, bv0], [bu1, bv1], [bu0, bv1]];
        custom[2] = uv16_north; // North (16 wide)
        custom[3] = uv16_south; // South (16 wide)
        custom[4] = uv10_end;   // West (10 wide)
        custom[5] = uv10_end;   // East (10 wide)
        add_sub_box_with_uv(mesh, head_min, head_max, head_slots, [0; 6], light, Some(custom), None);
    } else {
        // 2. Lower narrow step: [4, 4, 3] to [12, 5, 13]
        let step_min = [fx + 4.0 / 16.0, fy + 4.0 / 16.0, fz + 3.0 / 16.0];
        let step_max = [fx + 12.0 / 16.0, fy + 5.0 / 16.0, fz + 13.0 / 16.0];
        add_sub_box_proportional(mesh, step_min, step_max, [fx, fy, fz], base_slots, light);

        // 3. Waist beneath top: [6, 5, 4] to [10, 10, 12]
        let mid_min = [fx + 6.0 / 16.0, fy + 5.0 / 16.0, fz + 4.0 / 16.0];
        let mid_max = [fx + 10.0 / 16.0, fy + 10.0 / 16.0, fz + 12.0 / 16.0];
        add_sub_box_proportional(mesh, mid_min, mid_max, [fx, fy, fz], base_slots, light);

        // 4. Anvil top: [3, 10, 0] to [13, 16, 16]
        let head_min = [fx + 3.0 / 16.0, fy + 10.0 / 16.0, fz];
        let head_max = [fx + 13.0 / 16.0, fy + 1.0, fz + 1.0];
        // Custom top UV: map only opaque columns 3..13 of anvil_top texture
        let (tu0, tv0, tu1, tv1) = crate::texture::get_slot_uv(top_head_slot);
        let tdu = tu1 - tu0;
        let tdv = tv1 - tv0;
        let top_uvs = [
            [tu0 + tdu * (3.0 / 16.0), tv0 + tdv * 1.0],
            [tu0 + tdu * (13.0 / 16.0), tv0 + tdv * 1.0],
            [tu0 + tdu * (13.0 / 16.0), tv0],
            [tu0 + tdu * (3.0 / 16.0), tv0],
        ];
        let (bu0, bv0, bu1, bv1) = crate::texture::get_slot_uv(246);
        let dbu = bu1 - bu0;
        let dbv = bv1 - bv0;
        let v_top = bv0;
        let v_bot = bv0 + dbv * (6.0 / 16.0);
        let uv10_north = [
            [bu0 + dbu * (13.0 / 16.0), v_top],
            [bu0 + dbu * (13.0 / 16.0), v_bot],
            [bu0 + dbu * (3.0 / 16.0), v_bot],
            [bu0 + dbu * (3.0 / 16.0), v_top],
        ];
        let uv10_south = [
            [bu0 + dbu * (3.0 / 16.0), v_top],
            [bu0 + dbu * (3.0 / 16.0), v_bot],
            [bu0 + dbu * (13.0 / 16.0), v_bot],
            [bu0 + dbu * (13.0 / 16.0), v_top],
        ];
        let uv16_side = [[bu0, v_top], [bu0, v_bot], [bu1, v_bot], [bu1, v_top]];

        let head_slots = [top_head_slot, 246, 246, 246, 246, 246];
        let mut custom = [[[0.0f32; 2]; 4]; 6];
        custom[0] = top_uvs;
        custom[1] = [[bu0, bv0], [bu1, bv0], [bu1, bv1], [bu0, bv1]];
        custom[2] = uv10_north; // North (10 wide)
        custom[3] = uv10_south; // South (10 wide)
        custom[4] = uv16_side;  // West (16 wide)
        custom[5] = uv16_side;  // East (16 wide)
        add_sub_box_with_uv(mesh, head_min, head_max, head_slots, [0; 6], light, Some(custom), None);
    }
}

fn add_chest_quads(
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

    let facing = match block.meta & 7 {
        2 => BlockFace::North,
        3 => BlockFace::South,
        4 => BlockFace::West,
        5 => BlockFace::East,
        _ => BlockFace::North,
    };

    let is_trapped = block.id == 146;
    let is_ender = block.id == 130;

    let (is_double, is_left_half) = if !is_ender {
        match facing {
            BlockFace::North => {
                let west_b = world.get_block(wx - 1, wy, wz);
                let east_b = world.get_block(wx + 1, wy, wz);
                if west_b.id == block.id && (west_b.meta & 7) == (block.meta & 7) {
                    (true, true)
                } else if east_b.id == block.id && (east_b.meta & 7) == (block.meta & 7) {
                    (true, false)
                } else {
                    (false, false)
                }
            }
            BlockFace::South => {
                let west_b = world.get_block(wx - 1, wy, wz);
                let east_b = world.get_block(wx + 1, wy, wz);
                if east_b.id == block.id && (east_b.meta & 7) == (block.meta & 7) {
                    (true, true)
                } else if west_b.id == block.id && (west_b.meta & 7) == (block.meta & 7) {
                    (true, false)
                } else {
                    (false, false)
                }
            }
            BlockFace::West => {
                let north_b = world.get_block(wx, wy, wz - 1);
                let south_b = world.get_block(wx, wy, wz + 1);
                if south_b.id == block.id && (south_b.meta & 7) == (block.meta & 7) {
                    (true, true)
                } else if north_b.id == block.id && (north_b.meta & 7) == (block.meta & 7) {
                    (true, false)
                } else {
                    (false, false)
                }
            }
            BlockFace::East => {
                let north_b = world.get_block(wx, wy, wz - 1);
                let south_b = world.get_block(wx, wy, wz + 1);
                if north_b.id == block.id && (north_b.meta & 7) == (block.meta & 7) {
                    (true, true)
                } else if south_b.id == block.id && (south_b.meta & 7) == (block.meta & 7) {
                    (true, false)
                } else {
                    (false, false)
                }
            }
            _ => (false, false),
        }
    } else {
        (false, false)
    };

    if is_double {
        let base_slot = if is_trapped {
            if is_left_half { 289u16 } else { 293u16 }
        } else {
            if is_left_half { 281u16 } else { 285u16 }
        };
        let top_slot = base_slot;
        let outer_side = base_slot + 1;
        let front_slot = base_slot + 2;
        let back_slot = base_slot + 3;

        let (u0_top, v0_top, u1_top, v1_top) = crate::texture::get_slot_uv(top_slot);
        let du_t = u1_top - u0_top;
        let dv_t = v1_top - v0_top;
        let (u_in, u_out) = if is_left_half {
            (u1_top, u0_top + du_t * (1.0 / 16.0))
        } else {
            (u0_top, u0_top + du_t * (15.0 / 16.0))
        };
        let vt_near = v0_top + dv_t * (1.0 / 16.0);
        let vt_far = v0_top + dv_t * (15.0 / 16.0);

        let (u0_f, v0_f, u1_f, v1_f) = crate::texture::get_slot_uv(front_slot);
        let du_f = u1_f - u0_f;
        let (uf_in, uf_out) = if is_left_half {
            (u1_f, u0_f + du_f * (1.0 / 16.0))
        } else {
            (u0_f, u0_f + du_f * (15.0 / 16.0))
        };
        let vf_top = v0_f + (v1_f - v0_f) * (2.0 / 16.0);
        let vf_bot = v1_f;

        let (u0_b, v0_b, u1_b, v1_b) = crate::texture::get_slot_uv(back_slot);
        let du_b = u1_b - u0_b;
        let (ub_in, ub_out) = if !is_left_half {
            (u1_b, u0_b + du_b * (1.0 / 16.0))
        } else {
            (u0_b, u0_b + du_b * (15.0 / 16.0))
        };
        let vb_top = v0_b + (v1_b - v0_b) * (2.0 / 16.0);
        let vb_bot = v1_b;

        let (u0_s, v0_s, u1_s, v1_s) = crate::texture::get_slot_uv(outer_side);
        let du_s = u1_s - u0_s;
        let dv_s = v1_s - v0_s;
        let u_s_min = u0_s + du_s * (1.0 / 16.0);
        let u_s_max = u0_s + du_s * (15.0 / 16.0);
        let v_s_min = v0_s + dv_s * (2.0 / 16.0);
        let v_s_max = v1_s;

        let mut custom = [[[0.0f32; 2]; 4]; 6];
        let (min, max, slots) = match (facing, is_left_half) {
            (BlockFace::North, true) => {
                custom[0] = [[u_in, vt_far], [u_out, vt_far], [u_out, vt_near], [u_in, vt_near]];
                custom[1] = [[u_in, vt_near], [u_out, vt_near], [u_out, vt_far], [u_in, vt_far]];
                custom[2] = [[uf_out, vf_top], [uf_out, vf_bot], [uf_in, vf_bot], [uf_in, vf_top]];
                custom[3] = [[ub_in, vb_top], [ub_in, vb_bot], [ub_out, vb_bot], [ub_out, vb_top]];
                custom[5] = [[u_s_max, v_s_min], [u_s_max, v_s_max], [u_s_min, v_s_max], [u_s_min, v_s_min]];
                (
                    [fx, fy, fz + 1.0 / 16.0],
                    [fx + 15.0 / 16.0, fy + 14.0 / 16.0, fz + 15.0 / 16.0],
                    [top_slot, top_slot, front_slot, back_slot, 0, outer_side],
                )
            }
            (BlockFace::North, false) => {
                custom[0] = [[u_out, vt_far], [u_in, vt_far], [u_in, vt_near], [u_out, vt_near]];
                custom[1] = [[u_out, vt_near], [u_in, vt_near], [u_in, vt_far], [u_out, vt_far]];
                custom[2] = [[uf_in, vf_top], [uf_in, vf_bot], [uf_out, vf_bot], [uf_out, vf_top]];
                custom[3] = [[ub_out, vb_top], [ub_out, vb_bot], [ub_in, vb_bot], [ub_in, vb_top]];
                custom[4] = [[u_s_min, v_s_min], [u_s_min, v_s_max], [u_s_max, v_s_max], [u_s_max, v_s_min]];
                (
                    [fx + 1.0 / 16.0, fy, fz + 1.0 / 16.0],
                    [fx + 1.0, fy + 14.0 / 16.0, fz + 15.0 / 16.0],
                    [top_slot, top_slot, front_slot, back_slot, outer_side, 0],
                )
            }
            (BlockFace::South, true) => {
                custom[0] = [[u_out, vt_near], [u_in, vt_near], [u_in, vt_far], [u_out, vt_far]];
                custom[1] = [[u_out, vt_far], [u_in, vt_far], [u_in, vt_near], [u_out, vt_near]];
                custom[3] = [[uf_out, vf_top], [uf_out, vf_bot], [uf_in, vf_bot], [uf_in, vf_top]];
                custom[2] = [[ub_in, vb_top], [ub_in, vb_bot], [ub_out, vb_bot], [ub_out, vb_top]];
                custom[4] = [[u_s_max, v_s_min], [u_s_max, v_s_max], [u_s_min, v_s_max], [u_s_min, v_s_min]];
                (
                    [fx + 1.0 / 16.0, fy, fz + 1.0 / 16.0],
                    [fx + 1.0, fy + 14.0 / 16.0, fz + 15.0 / 16.0],
                    [top_slot, top_slot, back_slot, front_slot, outer_side, 0],
                )
            }
            (BlockFace::South, false) => {
                custom[0] = [[u_in, vt_near], [u_out, vt_near], [u_out, vt_far], [u_in, vt_far]];
                custom[1] = [[u_in, vt_far], [u_out, vt_far], [u_out, vt_near], [u_in, vt_near]];
                custom[3] = [[uf_in, vf_top], [uf_in, vf_bot], [uf_out, vf_bot], [uf_out, vf_top]];
                custom[2] = [[ub_out, vb_top], [ub_out, vb_bot], [ub_in, vb_bot], [ub_in, vb_top]];
                custom[5] = [[u_s_min, v_s_min], [u_s_min, v_s_max], [u_s_max, v_s_max], [u_s_max, v_s_min]];
                (
                    [fx, fy, fz + 1.0 / 16.0],
                    [fx + 15.0 / 16.0, fy + 14.0 / 16.0, fz + 15.0 / 16.0],
                    [top_slot, top_slot, back_slot, front_slot, 0, outer_side],
                )
            }
            (BlockFace::West, true) => {
                custom[0] = [[u_in, vt_near], [u_in, vt_far], [u_out, vt_far], [u_out, vt_near]];
                custom[1] = [[u_out, vt_near], [u_out, vt_far], [u_in, vt_far], [u_in, vt_near]];
                custom[4] = [[uf_out, vf_top], [uf_out, vf_bot], [uf_in, vf_bot], [uf_in, vf_top]];
                custom[5] = [[ub_in, vb_top], [ub_in, vb_bot], [ub_out, vb_bot], [ub_out, vb_top]];
                custom[2] = [[u_s_max, v_s_min], [u_s_max, v_s_max], [u_s_min, v_s_max], [u_s_min, v_s_min]];
                (
                    [fx + 1.0 / 16.0, fy, fz + 1.0 / 16.0],
                    [fx + 15.0 / 16.0, fy + 14.0 / 16.0, fz + 1.0],
                    [top_slot, top_slot, outer_side, 0, front_slot, back_slot],
                )
            }
            (BlockFace::West, false) => {
                custom[0] = [[u_out, vt_near], [u_out, vt_far], [u_in, vt_far], [u_in, vt_near]];
                custom[1] = [[u_in, vt_near], [u_in, vt_far], [u_out, vt_far], [u_out, vt_near]];
                custom[4] = [[uf_in, vf_top], [uf_in, vf_bot], [uf_out, vf_bot], [uf_out, vf_top]];
                custom[5] = [[ub_out, vb_top], [ub_out, vb_bot], [ub_in, vb_bot], [ub_in, vb_top]];
                custom[3] = [[u_s_min, v_s_min], [u_s_min, v_s_max], [u_s_max, v_s_max], [u_s_max, v_s_min]];
                (
                    [fx + 1.0 / 16.0, fy, fz],
                    [fx + 15.0 / 16.0, fy + 14.0 / 16.0, fz + 15.0 / 16.0],
                    [top_slot, top_slot, 0, outer_side, front_slot, back_slot],
                )
            }
            (BlockFace::East, true) => {
                custom[0] = [[u_out, vt_far], [u_out, vt_near], [u_in, vt_near], [u_in, vt_far]];
                custom[1] = [[u_in, vt_far], [u_in, vt_near], [u_out, vt_near], [u_out, vt_far]];
                custom[5] = [[uf_out, vf_top], [uf_out, vf_bot], [uf_in, vf_bot], [uf_in, vf_top]];
                custom[4] = [[ub_in, vb_top], [ub_in, vb_bot], [ub_out, vb_bot], [ub_out, vb_top]];
                custom[3] = [[u_s_max, v_s_min], [u_s_max, v_s_max], [u_s_min, v_s_max], [u_s_min, v_s_min]];
                (
                    [fx + 1.0 / 16.0, fy, fz],
                    [fx + 15.0 / 16.0, fy + 14.0 / 16.0, fz + 15.0 / 16.0],
                    [top_slot, top_slot, 0, outer_side, back_slot, front_slot],
                )
            }
            (BlockFace::East, false) => {
                custom[0] = [[u_in, vt_far], [u_in, vt_near], [u_out, vt_near], [u_out, vt_far]];
                custom[1] = [[u_out, vt_far], [u_out, vt_near], [u_in, vt_near], [u_in, vt_far]];
                custom[5] = [[uf_in, vf_top], [uf_in, vf_bot], [uf_out, vf_bot], [uf_out, vf_top]];
                custom[4] = [[ub_out, vb_top], [ub_out, vb_bot], [ub_in, vb_bot], [ub_in, vb_top]];
                custom[2] = [[u_s_min, v_s_min], [u_s_min, v_s_max], [u_s_max, v_s_max], [u_s_max, v_s_min]];
                (
                    [fx + 1.0 / 16.0, fy, fz + 1.0 / 16.0],
                    [fx + 15.0 / 16.0, fy + 14.0 / 16.0, fz + 1.0],
                    [top_slot, top_slot, outer_side, 0, back_slot, front_slot],
                )
            }
            _ => (
                [fx + 1.0 / 16.0, fy, fz + 1.0 / 16.0],
                [fx + 15.0 / 16.0, fy + 14.0 / 16.0, fz + 15.0 / 16.0],
                [top_slot, top_slot, front_slot, back_slot, outer_side, outer_side],
            ),
        };
        add_sub_box_with_uv(mesh, min, max, slots, [0; 6], light, Some(custom), None);
    } else {
        let (top_slot, side_slot, front_slot) = match block.id {
            146 => (259u16, 260u16, 261u16),
            130 => (262u16, 263u16, 264u16),
            _ => (256u16, 257u16, 258u16),
        };

        let slots = [
            top_slot,
            top_slot,
            if facing == BlockFace::North { front_slot } else { side_slot },
            if facing == BlockFace::South { front_slot } else { side_slot },
            if facing == BlockFace::West { front_slot } else { side_slot },
            if facing == BlockFace::East { front_slot } else { side_slot },
        ];

        let min = [fx + 1.0 / 16.0, fy, fz + 1.0 / 16.0];
        let max = [fx + 15.0 / 16.0, fy + 14.0 / 16.0, fz + 15.0 / 16.0];
        add_sub_box_proportional(mesh, min, max, [fx, fy, fz], slots, light);
    }
}

fn add_daylight_sensor_quads(
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

    let slots = [251u16, 6, 252, 252, 252, 252];
    let min = [fx, fy, fz];
    let max = [fx + 1.0, fy + 6.0 / 16.0, fz + 1.0];
    add_sub_box_proportional(mesh, min, max, [fx, fy, fz], slots, light);
}

fn add_enchanting_table_quads(
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

    // 1. Table base (height 12/16)
    let slots = [253u16, 255, 254, 254, 254, 254];
    let min = [fx, fy, fz];
    let max = [fx + 1.0, fy + 12.0 / 16.0, fz + 1.0];
    add_sub_box_proportional(mesh, min, max, [fx, fy, fz], slots, light);
    // Book is rendered dynamically via build_enchanting_book_model in the entities mesh
}

/// Build a 3D book model above an enchanting table.
/// When `open_factor == 0.0` (player far away), the book lies closed flat on the table.
/// When `open_factor > 0.0`, the book floats up, tilts back, rotates to face the player via `yaw`,
/// and smoothly opens with 3D page slabs and covers textured via `get_book_texture_uv`.
pub fn build_enchanting_book_model(
    mesh: &mut MeshData,
    table_x: f32,
    table_y: f32,
    table_z: f32,
    mut yaw: f32,
    open_factor: f32,
    time_ticks: f32,
    light: [f32; 3],
) {
    let color = [1.0, 1.0, 1.0, 1.0];
    let normal = [light[0], light[1], light[2]];

    let cx = table_x + 0.5;
    let cz = table_z + 0.5;

    let open = open_factor.clamp(0.0, 1.0);
    
    // Idle spinning and bobbing when closed
    let idle_bob = if open < 0.01 {
        (time_ticks * 0.1).sin() * (0.5 / 16.0)
    } else {
        0.0
    };
    
    if open < 0.01 {
        yaw = (time_ticks * 0.05) % (std::f32::consts::PI * 2.0);
    }

    let cy = table_y + 12.5 / 16.0 + idle_bob + open * (1.5 / 16.0);
    let pitch = open * 0.45; // ~26 deg backward tilt towards player
    let phi = -1.4 + open * 1.2; // -1.4 is closed (pointing up), -0.2 is open (slightly angled up)

    let cos_y = yaw.cos();
    let sin_y = yaw.sin();
    let cos_p = pitch.cos();
    let sin_p = pitch.sin();

    // Transform local book coordinates (spine at origin, length along Z) to world
    let xform = |lx: f32, ly: f32, lz: f32| -> [f32; 3] {
        let py = ly * cos_p - lz * sin_p;
        let pz = ly * sin_p + lz * cos_p;
        [
            cx + lx * cos_y + pz * sin_y,
            cy + py,
            cz - lx * sin_y + pz * cos_y,
        ]
    };

    let hw_c = 6.0 / 16.0; // Cover width
    let hd = 5.0 / 16.0;   // Spine / cover half-depth (Z)
    let th = 0.8 / 16.0;   // 3D page slab thickness

    // Unit directions for right wing:
    let r_ux = phi.cos();
    let r_uy = phi.sin();
    let r_nx = -phi.sin();
    let r_ny = phi.cos();

    // Unit directions for left wing:
    let l_ux = -phi.cos();
    let l_uy = phi.sin();
    let l_nx = phi.sin();
    let l_ny = phi.cos();

    // 1. Right Cover Bottom (outside leather)
    let r_cov_b0 = xform(0.0, 0.0, -hd);
    let r_cov_b1 = xform(0.0, 0.0, hd);
    let r_cov_b2 = xform(hw_c * r_ux, hw_c * r_uy, hd);
    let r_cov_b3 = xform(hw_c * r_ux, hw_c * r_uy, -hd);
    let r_cov_uv = [
        crate::texture::get_book_texture_uv(0.0, 0.0),
        crate::texture::get_book_texture_uv(0.0, 10.0),
        crate::texture::get_book_texture_uv(6.0, 10.0),
        crate::texture::get_book_texture_uv(6.0, 0.0),
    ];

    // 2. Left Cover Bottom (outside leather)
    let l_cov_b0 = xform(hw_c * l_ux, hw_c * l_uy, -hd);
    let l_cov_b1 = xform(hw_c * l_ux, hw_c * l_uy, hd);
    let l_cov_b2 = xform(0.0, 0.0, hd);
    let l_cov_b3 = xform(0.0, 0.0, -hd);
    let l_cov_uv = [
        crate::texture::get_book_texture_uv(16.0, 0.0),
        crate::texture::get_book_texture_uv(16.0, 10.0),
        crate::texture::get_book_texture_uv(22.0, 10.0),
        crate::texture::get_book_texture_uv(22.0, 0.0),
    ];

    // 3. Right Cover Inside (under page)
    let r_cov_in_uv = [
        crate::texture::get_book_texture_uv(6.0, 0.0),
        crate::texture::get_book_texture_uv(6.0, 10.0),
        crate::texture::get_book_texture_uv(12.0, 10.0),
        crate::texture::get_book_texture_uv(12.0, 0.0),
    ];

    // 4. Left Cover Inside (under page)
    let l_cov_in_uv = [
        crate::texture::get_book_texture_uv(22.0, 0.0),
        crate::texture::get_book_texture_uv(22.0, 10.0),
        crate::texture::get_book_texture_uv(28.0, 10.0),
        crate::texture::get_book_texture_uv(28.0, 0.0),
    ];

    // 5. Spine back
    let sp_w = 0.5 / 16.0;
    let spine_p0 = xform(-sp_w, 0.0, -hd);
    let spine_p1 = xform(-sp_w, 0.0, hd);
    let spine_p2 = xform(sp_w, 0.0, hd);
    let spine_p3 = xform(sp_w, 0.0, -hd);
    let spine_uv = [
        crate::texture::get_book_texture_uv(12.0, 0.0),
        crate::texture::get_book_texture_uv(12.0, 10.0),
        crate::texture::get_book_texture_uv(14.0, 10.0),
        crate::texture::get_book_texture_uv(14.0, 0.0),
    ];

    // 3D Page Slabs:
    let ps0 = 0.3 / 16.0;
    let ps1 = 5.3 / 16.0;
    let p_hd = 4.0 / 16.0; // 8 model pixels tall

    // Right Page slab vertices:
    // Base (on cover):
    let r_p_base0 = [ps0 * r_ux, ps0 * r_uy, -p_hd];
    let r_p_base1 = [ps0 * r_ux, ps0 * r_uy, p_hd];
    let r_p_base2 = [ps1 * r_ux, ps1 * r_uy, p_hd];
    let r_p_base3 = [ps1 * r_ux, ps1 * r_uy, -p_hd];
    // Top elevated by th along normal:
    let r_p_top0 = xform(r_p_base0[0] + th * r_nx, r_p_base0[1] + th * r_ny, -p_hd);
    let r_p_top1 = xform(r_p_base1[0] + th * r_nx, r_p_base1[1] + th * r_ny, p_hd);
    let r_p_top2 = xform(r_p_base2[0] + th * r_nx, r_p_base2[1] + th * r_ny, p_hd);
    let r_p_top3 = xform(r_p_base3[0] + th * r_nx, r_p_base3[1] + th * r_ny, -p_hd);

    let r_wb_top2 = xform(r_p_base2[0], r_p_base2[1], p_hd);
    let r_wb_top3 = xform(r_p_base3[0], r_p_base3[1], -p_hd);

    // Left Page slab vertices:
    let l_p_base0 = [ps1 * l_ux, ps1 * l_uy, -p_hd];
    let l_p_base1 = [ps1 * l_ux, ps1 * l_uy, p_hd];
    let l_p_base2 = [ps0 * l_ux, ps0 * l_uy, p_hd];
    let l_p_base3 = [ps0 * l_ux, ps0 * l_uy, -p_hd];
    let l_p_top0 = xform(l_p_base0[0] + th * l_nx, l_p_base0[1] + th * l_ny, -p_hd);
    let l_p_top1 = xform(l_p_base1[0] + th * l_nx, l_p_base1[1] + th * l_ny, p_hd);
    let l_p_top2 = xform(l_p_base2[0] + th * l_nx, l_p_base2[1] + th * l_ny, p_hd);
    let l_p_top3 = xform(l_p_base3[0] + th * l_nx, l_p_base3[1] + th * l_ny, -p_hd);

    let l_wb_top0 = xform(l_p_base0[0], l_p_base0[1], -p_hd);
    let l_wb_top1 = xform(l_p_base1[0], l_p_base1[1], p_hd);

    // Right Page runes top:
    let r_page_uv = [
        crate::texture::get_book_texture_uv(1.0, 11.0),
        crate::texture::get_book_texture_uv(1.0, 19.0),
        crate::texture::get_book_texture_uv(6.0, 19.0),
        crate::texture::get_book_texture_uv(6.0, 11.0),
    ];
    // Left Page runes top:
    let l_page_uv = [
        crate::texture::get_book_texture_uv(13.0, 11.0),
        crate::texture::get_book_texture_uv(13.0, 19.0),
        crate::texture::get_book_texture_uv(18.0, 19.0),
        crate::texture::get_book_texture_uv(18.0, 11.0),
    ];

    // Right Page outer rim:
    let r_rim_uv = [
        crate::texture::get_book_texture_uv(0.0, 11.0),
        crate::texture::get_book_texture_uv(0.0, 19.0),
        crate::texture::get_book_texture_uv(1.0, 19.0),
        crate::texture::get_book_texture_uv(1.0, 11.0),
    ];
    // Left Page outer rim:
    let l_rim_uv = [
        crate::texture::get_book_texture_uv(12.0, 11.0),
        crate::texture::get_book_texture_uv(12.0, 19.0),
        crate::texture::get_book_texture_uv(13.0, 19.0),
        crate::texture::get_book_texture_uv(13.0, 11.0),
    ];

    let quads: [([f32; 3], [f32; 3], [f32; 3], [f32; 3], [[f32; 2]; 4]); 9] = [
        // 1. Right cover bottom (leather outside)
        (r_cov_b0, r_cov_b1, r_cov_b2, r_cov_b3, r_cov_uv),
        // 2. Left cover bottom (leather outside)
        (l_cov_b0, l_cov_b1, l_cov_b2, l_cov_b3, l_cov_uv),
        // 3. Right cover top (inside leather)
        (r_cov_b3, r_cov_b2, r_cov_b1, r_cov_b0, r_cov_in_uv),
        // 4. Left cover top (inside leather)
        (l_cov_b3, l_cov_b2, l_cov_b1, l_cov_b0, l_cov_in_uv),
        // 5. Spine back
        (spine_p0, spine_p1, spine_p2, spine_p3, spine_uv),
        // 6. Right page top (runes facing player)
        (r_p_top0, r_p_top1, r_p_top2, r_p_top3, r_page_uv),
        // 7. Left page top (runes facing player)
        (l_p_top0, l_p_top1, l_p_top2, l_p_top3, l_page_uv),
        // 8. Right page outer slab rim (paper edge)
        (r_p_top3, r_p_top2, r_wb_top2, r_wb_top3, r_rim_uv),
        // 9. Left page outer slab rim (paper edge)
        (l_wb_top0, l_wb_top1, l_p_top1, l_p_top0, l_rim_uv),
    ];

    for (p0, p1, p2, p3, uvs) in quads {
        let start_idx = mesh.vertices.len() as u32;
        for (i, pos) in [p0, p1, p2, p3].iter().enumerate() {
            mesh.vertices.push(Vertex {
                position: *pos,
                uv: uvs[i],
                color,
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

fn add_double_plant_quads(
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

    let is_top = (block.meta & 8) != 0;
    let variant = if is_top {
        let below = world.get_block(wx, wy - 1, wz);
        if below.id == 175 { below.meta & 7 } else { block.meta & 7 }
    } else {
        block.meta & 7
    };

    let effective_meta = if is_top { variant | 8 } else { variant };
    let (slot, tint) = crate::texture::get_block_slot(175, effective_meta, BlockFace::North);
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
    let (stem_y0, stem_y1, stem_v0, stem_v1) = if variant == 0 && is_top {
        // Sunflower top stem only extends to y=0.5, using bottom half of texture
        (fy, fy + 0.5, v0 + (v1 - v0) * 0.5, v1)
    } else {
        (fy, fy + 1.0, v0, v1)
    };

    let planes = [
        [
            ([x0, stem_y0, z0], [u0, stem_v1]),
            ([x1, stem_y0, z1], [u1, stem_v1]),
            ([x1, stem_y1, z1], [u1, stem_v0]),
            ([x0, stem_y1, z0], [u0, stem_v0]),
        ],
        [
            ([x1, stem_y0, z1], [u0, stem_v1]),
            ([x0, stem_y0, z0], [u1, stem_v1]),
            ([x0, stem_y1, z0], [u1, stem_v0]),
            ([x1, stem_y1, z1], [u0, stem_v0]),
        ],
        [
            ([x0, stem_y0, z1], [u0, stem_v1]),
            ([x1, stem_y0, z0], [u1, stem_v1]),
            ([x1, stem_y1, z0], [u1, stem_v0]),
            ([x0, stem_y1, z1], [u0, stem_v0]),
        ],
        [
            ([x1, stem_y0, z0], [u0, stem_v1]),
            ([x0, stem_y0, z1], [u1, stem_v1]),
            ([x0, stem_y1, z1], [u1, stem_v0]),
            ([x1, stem_y1, z0], [u0, stem_v0]),
        ],
    ];

    if !(variant == 0 && is_top) {
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

    if variant == 0 && is_top {
        let (fu0, fv0, fu1, fv1) = crate::texture::get_slot_uv(267);
        let (bu0, bv0, bu1, bv1) = crate::texture::get_slot_uv(268);

        // Sunflower flower disk: canonical Minecraft model is tilted 22.5 deg around Z
        // facing East (+X) and tilted upwards, centered exactly at the top of the stem (y=0.5, x=0.5).
        let x_bot = fx + 0.6988;
        let y_bot = fy + 0.02;
        let x_top = fx + 0.3012;
        let y_top = fy + 0.98;
        let z_min = fz + 1.0 / 16.0;
        let z_max = fz + 15.0 / 16.0;

        let head_front = [
            ([x_bot + 0.003, y_bot, z_max], [fu0, fv1]),
            ([x_bot + 0.003, y_bot, z_min], [fu1, fv1]),
            ([x_top + 0.003, y_top, z_min], [fu1, fv0]),
            ([x_top + 0.003, y_top, z_max], [fu0, fv0]),
        ];
        let head_back = [
            ([x_bot - 0.003, y_bot, z_min], [bu0, bv1]),
            ([x_bot - 0.003, y_bot, z_max], [bu1, bv1]),
            ([x_top - 0.003, y_top, z_max], [bu1, bv0]),
            ([x_top - 0.003, y_top, z_min], [bu0, bv0]),
        ];

        for quad in [head_front, head_back] {
            let start_idx = mesh.vertices.len() as u32;
            for (pos, uv) in quad {
                mesh.vertices.push(Vertex {
                    position: pos,
                    uv,
                    color: [1.0, 1.0, 1.0, 1.0],
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
}

fn add_beacon_quads(
    mesh: &mut MeshData,
    world: &World,
    wx: i32,
    wy: i32,
    wz: i32,
    _block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let full_light = [1.0, 1.0, 1.0]; // Beacon emits light 15

    // 1. Inner beacon diamond core: 12x12x12 crystal [2/16, 2/16, 2/16] to [14/16, 14/16, 14/16]
    let core_slots = [250u16; 6];
    let core_min = [fx + 2.0 / 16.0, fy + 2.0 / 16.0, fz + 2.0 / 16.0];
    let core_max = [fx + 14.0 / 16.0, fy + 14.0 / 16.0, fz + 14.0 / 16.0];
    add_sub_box_proportional(mesh, core_min, core_max, [fx, fy, fz], core_slots, full_light);

    // 2. Outer glass cube casing: [fx, fy, fz] to [fx + 1.0, fy + 1.0, fz + 1.0] (transparent mesh)
    let glass_slot = 13u16;
    let (gu0, gv0, gu1, gv1) = crate::texture::get_slot_uv(glass_slot);
    let color = [1.0, 1.0, 1.0, 1.0];
    let normal = full_light;

    let neighbors = [
        (wx, wy + 1, wz, [[fx, fy + 1.0, fz + 1.0], [fx + 1.0, fy + 1.0, fz + 1.0], [fx + 1.0, fy + 1.0, fz], [fx, fy + 1.0, fz]], 1.0f32),
        (wx, wy - 1, wz, [[fx, fy, fz], [fx + 1.0, fy, fz], [fx + 1.0, fy, fz + 1.0], [fx, fy, fz + 1.0]], 0.5f32),
        (wx, wy, wz - 1, [[fx + 1.0, fy + 1.0, fz], [fx + 1.0, fy, fz], [fx, fy, fz], [fx, fy + 1.0, fz]], 0.8f32),
        (wx, wy, wz + 1, [[fx, fy + 1.0, fz + 1.0], [fx, fy, fz + 1.0], [fx + 1.0, fy, fz + 1.0], [fx + 1.0, fy + 1.0, fz + 1.0]], 0.8f32),
        (wx - 1, wy, wz, [[fx, fy + 1.0, fz], [fx, fy, fz], [fx, fy, fz + 1.0], [fx, fy + 1.0, fz + 1.0]], 0.6f32),
        (wx + 1, wy, wz, [[fx + 1.0, fy + 1.0, fz + 1.0], [fx + 1.0, fy, fz + 1.0], [fx + 1.0, fy, fz], [fx + 1.0, fy + 1.0, fz]], 0.6f32),
    ];

    let uvs = [
        [gu0, gv1],
        [gu1, gv1],
        [gu1, gv0],
        [gu0, gv0],
    ];

    for (nx, ny, nz, pos, shade) in neighbors {
        let nb = world.get_block(nx, ny, nz);
        if nb.id != 138 && !is_opaque_cube(nb.id) {
            let face_col = [color[0] * shade, color[1] * shade, color[2] * shade, 1.0];
            let start_idx = mesh.transparent_vertices.len() as u32;
            for i in 0..4 {
                mesh.transparent_vertices.push(Vertex {
                    position: pos[i],
                    uv: uvs[i],
                    color: face_col,
                    normal,
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
        }
    }
}

fn add_end_portal_quads(
    mesh: &mut MeshData,
    wx: i32,
    wy: i32,
    wz: i32,
    _block: Block,
) {
    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let color = [1.0, 1.0, 1.0, 1.0];
    let normal = [0.0, 1.0, 0.0];
    let y = fy + 0.75;

    let quads = [
        // Top face
        [
            [fx, y, fz + 1.0],
            [fx + 1.0, y, fz + 1.0],
            [fx + 1.0, y, fz],
            [fx, y, fz],
        ],
        // Bottom face
        [
            [fx, y, fz],
            [fx + 1.0, y, fz],
            [fx + 1.0, y, fz + 1.0],
            [fx, y, fz + 1.0],
        ],
    ];

    for quad in quads {
        let start_idx = mesh.end_portal_vertices.len() as u32;
        for pos in quad {
            mesh.end_portal_vertices.push(Vertex {
                position: pos,
                uv: [pos[0], pos[2]],
                color,
                normal,
            });
        }
        mesh.end_portal_indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    }
}

fn add_end_portal_frame_quads(
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

    // Frame base: height 13/16 (0.8125)
    // Face order: Top, Bottom, North, South, West, East
    // Top = slot 217, Bottom = slot 174 (end stone), Sides = slot 218
    let frame_slots = [217, 174, 218, 218, 218, 218];
    let min = [fx, fy, fz];
    let max = [fx + 1.0, fy + 13.0 / 16.0, fz + 1.0];
    add_sub_box_proportional(mesh, min, max, [fx, fy, fz], frame_slots, light);

    // Eye of ender (if present, meta & 4 != 0):
    if (block.meta & 4) != 0 {
        let eye_slots = [219; 6];
        let eye_min = [fx + 4.0 / 16.0, fy + 13.0 / 16.0, fz + 4.0 / 16.0];
        let eye_max = [fx + 12.0 / 16.0, fy + 1.0, fz + 12.0 / 16.0];
        add_sub_box_proportional(mesh, eye_min, eye_max, [fx, fy, fz], eye_slots, light);
    }
}

fn add_brewing_stand_quads(
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

    // 1. Three base pads: height 2/16 (0.125)
    // Top = slot 237, Bottom = slot 20 (obsidian), Sides = slot 237
    let base_slots = [237, 20, 237, 237, 237, 237];
    // East pad: [9/16, 0, 5/16] to [15/16, 2/16, 11/16]
    add_sub_box_proportional(
        mesh,
        [fx + 9.0 / 16.0, fy, fz + 5.0 / 16.0],
        [fx + 15.0 / 16.0, fy + 2.0 / 16.0, fz + 11.0 / 16.0],
        [fx, fy, fz],
        base_slots,
        light,
    );
    // Northwest pad: [1/16, 0, 1/16] to [7/16, 2/16, 7/16]
    add_sub_box_proportional(
        mesh,
        [fx + 1.0 / 16.0, fy, fz + 1.0 / 16.0],
        [fx + 7.0 / 16.0, fy + 2.0 / 16.0, fz + 7.0 / 16.0],
        [fx, fy, fz],
        base_slots,
        light,
    );
    // Southwest pad: [1/16, 0, 9/16] to [7/16, 2/16, 15/16]
    add_sub_box_proportional(
        mesh,
        [fx + 1.0 / 16.0, fy, fz + 9.0 / 16.0],
        [fx + 7.0 / 16.0, fy + 2.0 / 16.0, fz + 15.0 / 16.0],
        [fx, fy, fz],
        base_slots,
        light,
    );

    // 2. Central blaze rod / post: [7/16, 0, 7/16] to [9/16, 14/16, 9/16]
    let rod_slots = [236, 236, 236, 236, 236, 236];
    add_sub_box_proportional(
        mesh,
        [fx + 7.0 / 16.0, fy, fz + 7.0 / 16.0],
        [fx + 9.0 / 16.0, fy + 14.0 / 16.0, fz + 9.0 / 16.0],
        [fx, fy, fz],
        rod_slots,
        light,
    );

    // 3. Three bottle arms:
    // Slot 0 (East): meta & 1 != 0
    // Slot 1 (Southwest): meta & 2 != 0
    // Slot 2 (Northwest): meta & 4 != 0
    let (u0, v0, u1, v1) = crate::texture::get_slot_uv(236);
    let u_mid = (u0 + u1) * 0.5;
    let col = [1.0, 1.0, 1.0, 1.0];

    let pc0 = [fx + 0.5, fy, fz + 0.5];
    let pc1 = [fx + 0.5, fy + 1.0, fz + 0.5];

    let arms = [
        ([fx + 1.0, fy, fz + 0.5], [fx + 1.0, fy + 1.0, fz + 0.5], (block.meta & 1) != 0),
        ([fx + 0.146, fy, fz + 0.854], [fx + 0.146, fy + 1.0, fz + 0.854], (block.meta & 2) != 0),
        ([fx + 0.146, fy, fz + 0.146], [fx + 0.146, fy + 1.0, fz + 0.146], (block.meta & 4) != 0),
    ];

    for (p_out0, p_out1, has_bottle) in arms {
        // When has_bottle: UV uses u0..u_mid (bottle pixels)
        // When empty: UV uses u_mid..u1 (metal bracket without bottle)
        let (ua, ub) = if has_bottle {
            (u0, u_mid)
        } else {
            (u_mid, u1)
        };

        let quads = [
            // Front face
            [
                (pc0, [ua, v1]),
                (p_out0, [ub, v1]),
                (p_out1, [ub, v0]),
                (pc1, [ua, v0]),
            ],
            // Back face
            [
                (p_out0, [ub, v1]),
                (pc0, [ua, v1]),
                (pc1, [ua, v0]),
                (p_out1, [ub, v0]),
            ],
        ];

        for quad in quads {
            let start_idx = mesh.vertices.len() as u32;
            for (pos, uv) in quad {
                mesh.vertices.push(Vertex {
                    position: pos,
                    uv,
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
}

fn add_rail_quads(
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
    let col = [1.0, 1.0, 1.0, 1.0];

    let (slot, tint) = crate::texture::get_block_slot(block.id, block.meta, BlockFace::Top);
    let (u0, v0, u1, v1) = crate::texture::get_slot_uv(slot);
    let final_color = match tint {
        Some(t) => [t[0], t[1], t[2], 1.0],
        None => col,
    };

    let shape = if block.id == 66 {
        block.meta & 15
    } else {
        block.meta & 7
    };

    let h = 0.02f32;

    let (quad, uvs) = match shape {
        // 0: North-South straight
        0 => (
            [
                [fx, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz],
                [fx, fy + h, fz],
            ],
            [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
        ),
        // 1: East-West straight (rotated 90 deg)
        1 => (
            [
                [fx, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz],
                [fx, fy + h, fz],
            ],
            [[u1, v0], [u1, v1], [u0, v1], [u0, v0]],
        ),
        // 2: Sloped ascending East (rises from x=0 to x=1)
        2 => (
            [
                [fx, fy + h, fz + 1.0],
                [fx + 1.0, fy + 1.0 + h, fz + 1.0],
                [fx + 1.0, fy + 1.0 + h, fz],
                [fx, fy + h, fz],
            ],
            [[u1, v0], [u1, v1], [u0, v1], [u0, v0]],
        ),
        // 3: Sloped ascending West (rises from x=1 to x=0)
        3 => (
            [
                [fx, fy + 1.0 + h, fz + 1.0],
                [fx + 1.0, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz],
                [fx, fy + 1.0 + h, fz],
            ],
            [[u1, v0], [u1, v1], [u0, v1], [u0, v0]],
        ),
        // 4: Sloped ascending North (rises from z=1 to z=0)
        4 => (
            [
                [fx, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz + 1.0],
                [fx + 1.0, fy + 1.0 + h, fz],
                [fx, fy + 1.0 + h, fz],
            ],
            [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
        ),
        // 5: Sloped ascending South (rises from z=0 to z=1)
        5 => (
            [
                [fx, fy + 1.0 + h, fz + 1.0],
                [fx + 1.0, fy + 1.0 + h, fz + 1.0],
                [fx + 1.0, fy + h, fz],
                [fx, fy + h, fz],
            ],
            [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
        ),
        // 6: Curved South-East (rot 0)
        6 => (
            [
                [fx, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz],
                [fx, fy + h, fz],
            ],
            [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
        ),
        // 7: Curved South-West (rot 1)
        7 => (
            [
                [fx, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz],
                [fx, fy + h, fz],
            ],
            [[u1, v1], [u1, v0], [u0, v0], [u0, v1]],
        ),
        // 8: Curved North-West (rot 2)
        8 => (
            [
                [fx, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz],
                [fx, fy + h, fz],
            ],
            [[u1, v0], [u0, v0], [u0, v1], [u1, v1]],
        ),
        // 9: Curved North-East (rot 3)
        9 => (
            [
                [fx, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz],
                [fx, fy + h, fz],
            ],
            [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
        ),
        _ => (
            [
                [fx, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz + 1.0],
                [fx + 1.0, fy + h, fz],
                [fx, fy + h, fz],
            ],
            [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
        ),
    };

    let start_idx = mesh.vertices.len() as u32;
    for i in 0..4 {
        mesh.vertices.push(Vertex {
            position: quad[i],
            uv: uvs[i],
            color: final_color,
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
        start_idx,
        start_idx + 2,
        start_idx + 1,
        start_idx,
        start_idx + 3,
        start_idx + 2,
    ]);
}

fn add_cauldron_quads(
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

    // Cauldron textures:
    // Side: 232, Top: 233, Inner: 234, Bottom: 235
    // 1. Four corner legs (each 2x2x3 pixels)
    let leg_slots = [235, 235, 232, 232, 232, 232];
    add_sub_box_proportional(mesh, [fx, fy, fz], [fx + 2.0 / 16.0, fy + 3.0 / 16.0, fz + 2.0 / 16.0], [fx, fy, fz], leg_slots, light);
    add_sub_box_proportional(mesh, [fx + 14.0 / 16.0, fy, fz], [fx + 1.0, fy + 3.0 / 16.0, fz + 2.0 / 16.0], [fx, fy, fz], leg_slots, light);
    add_sub_box_proportional(mesh, [fx, fy, fz + 14.0 / 16.0], [fx + 2.0 / 16.0, fy + 3.0 / 16.0, fz + 1.0], [fx, fy, fz], leg_slots, light);
    add_sub_box_proportional(mesh, [fx + 14.0 / 16.0, fy, fz + 14.0 / 16.0], [fx + 1.0, fy + 3.0 / 16.0, fz + 1.0], [fx, fy, fz], leg_slots, light);

    // 2. Basin bottom plate: y from 3/16 to 5/16
    let bottom_slots = [234, 235, 232, 232, 232, 232];
    add_sub_box_proportional(mesh, [fx, fy + 3.0 / 16.0, fz], [fx + 1.0, fy + 5.0 / 16.0, fz + 1.0], [fx, fy, fz], bottom_slots, light);

    // 3. Four walls from y: 5/16 to 1.0 (thickness 2/16 = 0.125)
    // North wall (z in 0..2/16):
    let n_slots = [233, 235, 232, 234, 232, 232];
    add_sub_box_proportional(mesh, [fx, fy + 5.0 / 16.0, fz], [fx + 1.0, fy + 1.0, fz + 2.0 / 16.0], [fx, fy, fz], n_slots, light);

    // South wall (z in 14/16..1):
    let s_slots = [233, 235, 234, 232, 232, 232];
    add_sub_box_proportional(mesh, [fx, fy + 5.0 / 16.0, fz + 14.0 / 16.0], [fx + 1.0, fy + 1.0, fz + 1.0], [fx, fy, fz], s_slots, light);

    // West wall (x in 0..2/16, z in 2/16..14/16):
    let w_slots = [233, 235, 232, 232, 232, 234];
    add_sub_box_proportional(mesh, [fx, fy + 5.0 / 16.0, fz + 2.0 / 16.0], [fx + 2.0 / 16.0, fy + 1.0, fz + 14.0 / 16.0], [fx, fy, fz], w_slots, light);

    // East wall (x in 14/16..1, z in 2/16..14/16):
    let e_slots = [233, 235, 232, 232, 234, 232];
    add_sub_box_proportional(mesh, [fx + 14.0 / 16.0, fy + 5.0 / 16.0, fz + 2.0 / 16.0], [fx + 1.0, fy + 1.0, fz + 14.0 / 16.0], [fx, fy, fz], e_slots, light);

    // 4. If filled with water (meta > 0, 1..3 in vanilla):
    let level = (block.meta & 3).min(3);
    if level > 0 {
        let water_y = fy + (4.0 + (level as f32) * 3.5) / 16.0;
        let (wu0, wv0, wu1, wv1) = crate::texture::get_slot_uv(29);
        let w_quad = [
            ([fx + 2.0 / 16.0, water_y, fz + 14.0 / 16.0], [wu0, wv1]),
            ([fx + 14.0 / 16.0, water_y, fz + 14.0 / 16.0], [wu1, wv1]),
            ([fx + 14.0 / 16.0, water_y, fz + 2.0 / 16.0], [wu1, wv0]),
            ([fx + 2.0 / 16.0, water_y, fz + 2.0 / 16.0], [wu0, wv0]),
        ];
        let start_idx = mesh.vertices.len() as u32;
        for (pos, uv) in w_quad {
            mesh.vertices.push(Vertex {
                position: pos,
                uv,
                color: [0.2, 0.4, 0.9, 1.0],
                normal: light,
            });
        }
        mesh.indices.extend_from_slice(&[
            start_idx, start_idx + 1, start_idx + 2,
            start_idx, start_idx + 2, start_idx + 3,
        ]);
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

                    if is_rail(block_id) {
                        add_rail_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if is_flat_floor(block_id) {
                        let h = if block_id == 171 { 1.0 / 16.0 } else { 0.02 };
                        add_quad(&mut mesh, world, wx, wy, wz, BlockFace::Top, block, h);
                        continue;
                    }

                    if block_id == 51 {
                        add_fire_quads(&mut mesh, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 90 {
                        add_nether_portal_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 117 {
                        add_brewing_stand_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 118 {
                        add_cauldron_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 119 {
                        add_end_portal_quads(&mut mesh, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 120 {
                        add_end_portal_frame_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 65 {
                        add_ladder_quads(&mut mesh, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 81 {
                        add_cactus_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 145 {
                        add_anvil_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 54 || block_id == 130 || block_id == 146 {
                        add_chest_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 151 || block_id == 178 {
                        add_daylight_sensor_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 116 {
                        add_enchanting_table_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 175 {
                        add_double_plant_quads(&mut mesh, world, wx, wy, wz, block);
                        continue;
                    }

                    if block_id == 138 {
                        add_beacon_quads(&mut mesh, world, wx, wy, wz, block);
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

                    let top_visible = if block_id == 95 && id_above == 95 { world.get_block(wx, wy + 1, wz).meta != block.meta } else { should_render_face(block_id, id_above) };
                    let bottom_visible = if block_id == 95 && id_below == 95 { world.get_block(wx, wy - 1, wz).meta != block.meta } else { should_render_face(block_id, id_below) };
                    let west_visible = if block_id == 95 && id_west == 95 { world.get_block(wx - 1, wy, wz).meta != block.meta } else { should_render_face(block_id, id_west) };
                    let east_visible = if block_id == 95 && id_east == 95 { world.get_block(wx + 1, wy, wz).meta != block.meta } else { should_render_face(block_id, id_east) };
                    let north_visible = if block_id == 95 && id_north == 95 { world.get_block(wx, wy, wz - 1).meta != block.meta } else { should_render_face(block_id, id_north) };
                    let south_visible = if block_id == 95 && id_south == 95 { world.get_block(wx, wy, wz + 1).meta != block.meta } else { should_render_face(block_id, id_south) };

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

fn compute_smooth_lighting(world: &World, wx: i32, wy: i32, wz: i32, face: BlockFace) -> [(f32, f32, f32); 4] {
    let get_light = |dx: i32, dy: i32, dz: i32| -> (bool, f32, f32) {
        let b = world.get_block(wx + dx, wy + dy, wz + dz);
        let emit = crate::world::block_emission(b.id);
        (is_opaque_cube(b.id), b.sky_light as f32, b.block_light.max(emit) as f32)
    };

    let calc = |dx1: i32, dy1: i32, dz1: i32,
                dx2: i32, dy2: i32, dz2: i32,
                cdx: i32, cdy: i32, cdz: i32,
                adx: i32, ady: i32, adz: i32| -> (f32, f32, f32) {
        let (s1, sky1, blk1) = get_light(dx1, dy1, dz1);
        let (s2, sky2, blk2) = get_light(dx2, dy2, dz2);
        let (c, sky3, blk3) = get_light(cdx, cdy, cdz);
        let (_adj_s, sky0, blk0) = get_light(adx, ady, adz);

        let ao = if s1 && s2 {
            0.45
        } else {
            match (s1 as u8) + (s2 as u8) + (c as u8) {
                0 => 1.0,
                1 => 0.8,
                2 => 0.65,
                _ => 0.45,
            }
        };

        let mut sky = sky0;
        let mut blk = blk0;
        let mut count = 1.0;
        
        if !s1 { sky += sky1; blk += blk1; count += 1.0; } else { sky += sky0; blk += blk0; count += 1.0; }
        if !s2 { sky += sky2; blk += blk2; count += 1.0; } else { sky += sky0; blk += blk0; count += 1.0; }
        if !c { sky += sky3; blk += blk3; count += 1.0; } else { sky += sky0; blk += blk0; count += 1.0; }

        (ao, sky / count / 15.0, blk / count / 15.0)
    };

    match face {
        BlockFace::Top => {
            let v0 = calc(-1, 1, 0, 0, 1, 1, -1, 1, 1, 0, 1, 0);
            let v1 = calc(1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0);
            let v2 = calc(1, 1, 0, 0, 1, -1, 1, 1, -1, 0, 1, 0);
            let v3 = calc(-1, 1, 0, 0, 1, -1, -1, 1, -1, 0, 1, 0);
            [v0, v1, v2, v3]
        }
        BlockFace::Bottom => {
            let v0 = calc(-1, -1, 0, 0, -1, -1, -1, -1, -1, 0, -1, 0);
            let v1 = calc(1, -1, 0, 0, -1, -1, 1, -1, -1, 0, -1, 0);
            let v2 = calc(1, -1, 0, 0, -1, 1, 1, -1, 1, 0, -1, 0);
            let v3 = calc(-1, -1, 0, 0, -1, 1, -1, -1, 1, 0, -1, 0);
            [v0, v1, v2, v3]
        }
        BlockFace::North => {
            let v0 = calc(1, 0, -1, 0, 1, -1, 1, 1, -1, 0, 0, -1);
            let v1 = calc(1, 0, -1, 0, -1, -1, 1, -1, -1, 0, 0, -1);
            let v2 = calc(-1, 0, -1, 0, -1, -1, -1, -1, -1, 0, 0, -1);
            let v3 = calc(-1, 0, -1, 0, 1, -1, -1, 1, -1, 0, 0, -1);
            [v0, v1, v2, v3]
        }
        BlockFace::South => {
            let v0 = calc(-1, 0, 1, 0, 1, 1, -1, 1, 1, 0, 0, 1);
            let v1 = calc(-1, 0, 1, 0, -1, 1, -1, -1, 1, 0, 0, 1);
            let v2 = calc(1, 0, 1, 0, -1, 1, 1, -1, 1, 0, 0, 1);
            let v3 = calc(1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1);
            [v0, v1, v2, v3]
        }
        BlockFace::West => {
            let v0 = calc(-1, 0, -1, -1, 1, 0, -1, 1, -1, -1, 0, 0);
            let v1 = calc(-1, 0, -1, -1, -1, 0, -1, -1, -1, -1, 0, 0);
            let v2 = calc(-1, 0, 1, -1, -1, 0, -1, -1, 1, -1, 0, 0);
            let v3 = calc(-1, 0, 1, -1, 1, 0, -1, 1, 1, -1, 0, 0);
            [v0, v1, v2, v3]
        }
        BlockFace::East => {
            let v0 = calc(1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0);
            let v1 = calc(1, 0, 1, 1, -1, 0, 1, -1, 1, 1, 0, 0);
            let v2 = calc(1, 0, -1, 1, -1, 0, 1, -1, -1, 1, 0, 0);
            let v3 = calc(1, 0, -1, 1, 1, 0, 1, 1, -1, 1, 0, 0);
            [v0, v1, v2, v3]
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

    let v_light = if is_water {
        let adj_b = world.get_block(adj_x, adj_y, adj_z);
        let emit = crate::world::block_emission(block.id);
        let sky_l = (adj_b.sky_light as f32 / 15.0).clamp(0.0, 1.0);
        let block_l = ((adj_b.block_light.max(emit)) as f32 / 15.0).clamp(0.0, 1.0);
        [(1.0, sky_l, block_l); 4]
    } else {
        compute_smooth_lighting(world, wx, wy, wz, face)
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
            let (ao, sky_l, block_l) = v_light[i];
            mesh.transparent_vertices.push(Vertex {
                position: pos,
                uv,
                color: final_color,
                normal: [sky_l, block_l, ao],
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
            let (ao, sky_l, block_l) = v_light[i];
            mesh.vertices.push(Vertex {
                position: pos,
                uv,
                color: final_color,
                normal: [sky_l, block_l, ao],
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
        assert!(should_render_face(95, 20));
        assert!(should_render_face(20, 95));
        // Stained glass adjacent to air renders its face
        assert!(should_render_face(95, 0));
        // Stained glass against solid stone is culled (stone covers it)
        assert!(!should_render_face(95, 1));
        // Stone against stained glass renders its face (visible through glass)
        assert!(should_render_face(1, 95));
    }

    #[test]
    fn test_glass_culling_and_visibility() {
        // Normal glass against normal glass culls inner face
        assert!(!should_render_face(20, 20));
        // Normal glass against air renders face
        assert!(should_render_face(20, 0));
        // Normal glass against solid stone is culled
        assert!(!should_render_face(20, 1));
        // Solid stone against normal glass renders its face (terrain visible through glass)
        assert!(should_render_face(1, 20));
        // Dirt against normal glass renders its face
        assert!(should_render_face(3, 20));
        // Solid stone against glass pane renders its face
        assert!(should_render_face(1, 102));
        // Solid stone against stained glass pane renders its face
        assert!(should_render_face(1, 160));
    }

    #[test]
    fn test_pane_connectivity() {
        assert!(can_pane_connect(1)); // Stone
        assert!(can_pane_connect(20)); // Glass block
        assert!(can_pane_connect(95)); // Stained glass block
        assert!(can_pane_connect(101)); // Iron bars
        assert!(can_pane_connect(102)); // Glass pane
        assert!(can_pane_connect(160)); // Stained glass pane
        assert!(!can_pane_connect(0)); // Air
        assert!(!can_pane_connect(6)); // Sapling
        assert!(!can_pane_connect(53)); // Stairs
    }

    #[test]
    fn test_pane_connected_mesh_continuity() {
        let mut world = World::new();
        world.insert_chunk(ChunkColumn::new(0, 0));
        // Place stone at North (10, 64, 9) and South (10, 64, 11)
        world.set_block(10, 64, 9, Block { id: 1, meta: 0, sky_light: 15, block_light: 0 });
        world.set_block(10, 64, 11, Block { id: 1, meta: 0, sky_light: 15, block_light: 0 });

        let mut mesh = MeshData::new();
        let pane_block = Block { id: 102, meta: 0, sky_light: 15, block_light: 0 };
        add_pane_quads(&mut mesh, &world, 10, 64, 10, pane_block);

        // Connected North and South:
        // No end caps at North (touches stone) and South (touches stone)
        // No inner face at z = 10.5
        // West face (North arm: 10.0..10.5, South arm: 10.5..11.0)
        // East face (North arm: 10.0..10.5, South arm: 10.5..11.0)
        // Top face (North arm: 10.0..10.5, South arm: 10.5..11.0)
        // Bottom face (North arm: 10.0..10.5, South arm: 10.5..11.0)
        // 8 quads total = 32 vertices, 48 indices
        assert_eq!(mesh.vertices.len(), 32);
        assert_eq!(mesh.indices.len(), 48);
        assert!(mesh.transparent_vertices.is_empty());

        // Verify bounds: Z goes from 10.0 to 11.0 continuously
        let min_z = mesh.vertices.iter().map(|v| v.position[2]).fold(f32::INFINITY, f32::min);
        let max_z = mesh.vertices.iter().map(|v| v.position[2]).fold(f32::NEG_INFINITY, f32::max);
        assert!((min_z - 10.0).abs() < 1e-4);
        assert!((max_z - 11.0).abs() < 1e-4);
    }

    #[test]
    fn test_stained_glass_pane_routed_to_transparent_mesh() {
        let world = World::new();
        let mut mesh = MeshData::new();
        let stained_pane = Block { id: 160, meta: 0, sky_light: 15, block_light: 0 };
        add_pane_quads(&mut mesh, &world, 10, 64, 10, stained_pane);

        // Stained glass pane must go into transparent_vertices, NOT opaque vertices
        assert!(mesh.vertices.is_empty());
        assert!(!mesh.transparent_vertices.is_empty());
        assert!(!mesh.transparent_indices.is_empty());
    }

    #[test]
    fn test_non_opaque_cube_blocks() {
        assert!(!is_opaque_cube(90)); // Nether Portal
        assert!(!is_opaque_cube(92)); // Cake
        assert!(!is_opaque_cube(93)); // Repeater off
        assert!(!is_opaque_cube(94)); // Repeater on
        assert!(!is_opaque_cube(119)); // End Portal
        assert!(!is_opaque_cube(120)); // End Portal Frame
        assert!(!is_opaque_cube(122)); // Dragon Egg
        assert!(!is_opaque_cube(138)); // Beacon
        assert!(!is_opaque_cube(139)); // Cobblestone Wall
        assert!(!is_opaque_cube(157)); // Activator Rail
        assert!(!is_opaque_cube(167)); // Iron Trapdoor
    }

    #[test]
    fn test_end_portal_frame_mesh() {
        let world = World::new();
        let mut mesh = MeshData::new();
        // Frame without eye
        let frame_no_eye = Block { id: 120, meta: 0, sky_light: 15, block_light: 0 };
        add_end_portal_frame_quads(&mut mesh, &world, 10, 64, 10, frame_no_eye);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);

        // Frame with eye
        let mut mesh_eye = MeshData::new();
        let frame_with_eye = Block { id: 120, meta: 4, sky_light: 15, block_light: 0 };
        add_end_portal_frame_quads(&mut mesh_eye, &world, 10, 64, 10, frame_with_eye);
        assert_eq!(mesh_eye.vertices.len(), 48); // 24 frame + 24 eye
        assert_eq!(mesh_eye.indices.len(), 72);
    }

    #[test]
    fn test_nether_portal_mesh() {
        let world = World::new();
        let mut mesh = MeshData::new();
        let portal = Block { id: 90, meta: 0, sky_light: 15, block_light: 0 };
        add_nether_portal_quads(&mut mesh, &world, 10, 64, 10, portal);
        // Nether portal must be in transparent_vertices (2 quads double-sided = 8 vertices, 12 indices)
        assert!(mesh.vertices.is_empty());
        assert_eq!(mesh.transparent_vertices.len(), 8);
        assert_eq!(mesh.transparent_indices.len(), 12);
    }

    #[test]
    fn test_end_portal_mesh() {
        let mut mesh = MeshData::new();
        let portal = Block { id: 119, meta: 0, sky_light: 15, block_light: 0 };
        add_end_portal_quads(&mut mesh, 10, 64, 10, portal);
        // End portal has top and bottom quad in end_portal_vertices at y = 64.75
        assert_eq!(mesh.end_portal_vertices.len(), 8);
        assert_eq!(mesh.end_portal_indices.len(), 12);
        for v in &mesh.end_portal_vertices {
            assert!((v.position[1] - 64.75).abs() < 1e-4);
        }
    }

    #[test]
    fn test_ladder_mesh() {
        let mut mesh = MeshData::new();
        let ladder = Block { id: 65, meta: 2, sky_light: 15, block_light: 0 }; // Attached to South wall
        add_ladder_quads(&mut mesh, 10, 64, 10, ladder);
        // Single double-sided quad: 4 vertices, 12 indices
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices.len(), 12);
        // Attached to south wall: z = 10 + 1.0 - 0.05 = 10.95
        for v in &mesh.vertices {
            assert!((v.position[2] - 10.95).abs() < 1e-4);
        }
    }

    #[test]
    fn test_cactus_mesh() {
        let world = World::new();
        let mut mesh = MeshData::new();
        let cactus = Block { id: 81, meta: 0, sky_light: 15, block_light: 0 };
        add_cactus_quads(&mut mesh, &world, 10, 64, 10, cactus);
        // Standalone cactus has 6 faces = 24 vertices, 36 indices
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
        // North/South faces span [10.0, 11.0] in X with inset Z; West/East span [10.0, 11.0] in Z with inset X
        // Thorns protrude to the block boundary [10.0, 11.0]
        let min_x = mesh.vertices.iter().map(|v| v.position[0]).fold(f32::INFINITY, f32::min);
        let max_x = mesh.vertices.iter().map(|v| v.position[0]).fold(f32::NEG_INFINITY, f32::max);
        let min_z = mesh.vertices.iter().map(|v| v.position[2]).fold(f32::INFINITY, f32::min);
        let max_z = mesh.vertices.iter().map(|v| v.position[2]).fold(f32::NEG_INFINITY, f32::max);
        assert!((min_x - 10.0).abs() < 1e-4);
        assert!((max_x - 11.0).abs() < 1e-4);
        assert!((min_z - 10.0).abs() < 1e-4);
        assert!((max_z - 11.0).abs() < 1e-4);
    }

    #[test]
    fn test_chest_mesh() {
        let world = World::new();
        let mut mesh = MeshData::new();
        let chest = Block { id: 54, meta: 2, sky_light: 15, block_light: 0 };
        add_chest_quads(&mut mesh, &world, 10, 64, 10, chest);
        // 6 faces = 24 vertices, 36 indices
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
        // Bounds: [1/16, 0, 1/16] to [15/16, 14/16, 15/16]
        for v in &mesh.vertices {
            assert!(v.position[0] >= 10.0624 && v.position[0] <= 10.9376);
            assert!(v.position[1] >= 64.0 && v.position[1] <= 64.8751);
            assert!(v.position[2] >= 10.0624 && v.position[2] <= 10.9376);
        }
    }

    #[test]
    fn test_double_chest_mesh() {
        let mut world = World::new();
        world.insert_chunk(ChunkColumn::new(0, 0));
        world.set_block(10, 64, 10, Block { id: 54, meta: 2, sky_light: 15, block_light: 0 });
        world.set_block(11, 64, 10, Block { id: 54, meta: 2, sky_light: 15, block_light: 0 });

        let mut mesh_l = MeshData::new();
        let mut mesh_r = MeshData::new();
        add_chest_quads(&mut mesh_l, &world, 10, 64, 10, world.get_block(10, 64, 10));
        add_chest_quads(&mut mesh_r, &world, 11, 64, 10, world.get_block(11, 64, 10));

        // Each half has 6 faces (24 vertices, 36 indices)
        assert_eq!(mesh_l.vertices.len(), 24);
        assert_eq!(mesh_r.vertices.len(), 24);

        // Left half (at x=10) spans [10.0625, 11.0]
        let max_x_l = mesh_l.vertices.iter().map(|v| v.position[0]).fold(f32::NEG_INFINITY, f32::max);
        let min_x_l = mesh_l.vertices.iter().map(|v| v.position[0]).fold(f32::INFINITY, f32::min);
        assert!((min_x_l - 10.0625).abs() < 1e-4);
        assert!((max_x_l - 11.0).abs() < 1e-4);

        // Right half (at x=11) spans [11.0, 11.9375]
        let min_x_r = mesh_r.vertices.iter().map(|v| v.position[0]).fold(f32::INFINITY, f32::min);
        let max_x_r = mesh_r.vertices.iter().map(|v| v.position[0]).fold(f32::NEG_INFINITY, f32::max);
        assert!((min_x_r - 11.0).abs() < 1e-4);
        assert!((max_x_r - 11.9375).abs() < 1e-4);
    }

    #[test]
    fn test_anvil_mesh() {
        let world = World::new();
        let mut mesh = MeshData::new();
        let anvil = Block { id: 145, meta: 0, sky_light: 15, block_light: 0 };
        add_anvil_quads(&mut mesh, &world, 10, 64, 10, anvil);
        // 4 sub-boxes: base (24) + lower step (24) + waist (24) + head (24) = 96 vertices
        assert_eq!(mesh.vertices.len(), 96);
        assert_eq!(mesh.indices.len(), 144);
    }

    #[test]
    fn test_sunflower_mesh() {
        let world = World::new();
        let mut mesh = MeshData::new();
        let sunflower_top = Block { id: 175, meta: 8, sky_light: 15, block_light: 0 }; // variant 0, top
        add_double_plant_quads(&mut mesh, &world, 10, 65, 10, sunflower_top);
        // Angled head (8 vertices, 12 indices) ONLY (no crossed stem to avoid clipping)
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.indices.len(), 12);
    }

    #[test]
    fn test_fire_mesh() {
        let mut mesh = MeshData::new();
        let fire = Block { id: 51, meta: 0, sky_light: 15, block_light: 0 };
        add_fire_quads(&mut mesh, 10, 64, 10, fire);
        // 8 planes (4 double-sided: 2 diagonal + 2 lateral) = 32 vertices, 48 indices
        assert_eq!(mesh.vertices.len(), 32);
        assert_eq!(mesh.indices.len(), 48);
    }

    #[test]
    fn test_brewing_stand_mesh() {
        let world = World::new();
        let mut mesh = MeshData::new();
        let stand = Block { id: 117, meta: 0, sky_light: 15, block_light: 0 };
        add_brewing_stand_quads(&mut mesh, &world, 10, 64, 10, stand);
        // 3 base pads (72) + 1 center rod (24) + 3 two-sided arm quads (24) = 120 vertices
        assert_eq!(mesh.vertices.len(), 120);
        assert_eq!(mesh.indices.len(), 180);
    }

    #[test]
    fn test_rail_mesh() {
        let world = World::new();
        let mut mesh_straight = MeshData::new();
        let straight_rail = Block { id: 66, meta: 0, sky_light: 15, block_light: 0 };
        add_rail_quads(&mut mesh_straight, &world, 10, 64, 10, straight_rail);
        assert_eq!(mesh_straight.vertices.len(), 4);
        assert_eq!(mesh_straight.indices.len(), 12);

        let mut mesh_curved = MeshData::new();
        let curved_rail = Block { id: 66, meta: 6, sky_light: 15, block_light: 0 };
        add_rail_quads(&mut mesh_curved, &world, 10, 64, 10, curved_rail);
        assert_eq!(mesh_curved.vertices.len(), 4);
        assert_eq!(mesh_curved.indices.len(), 12);

        let mut mesh_sloped = MeshData::new();
        let sloped_rail = Block { id: 66, meta: 2, sky_light: 15, block_light: 0 };
        add_rail_quads(&mut mesh_sloped, &world, 10, 64, 10, sloped_rail);
        assert_eq!(mesh_sloped.vertices.len(), 4);
        assert_eq!(mesh_sloped.indices.len(), 12);
        // Eastern vertices are raised by 1.0
        let max_y = mesh_sloped.vertices.iter().map(|v| v.position[1]).fold(0.0f32, f32::max);
        assert!((max_y - 65.02).abs() < 1e-4);
    }

    #[test]
    fn test_cauldron_mesh() {
        let world = World::new();
        let mut mesh = MeshData::new();
        let empty_cauldron = Block { id: 118, meta: 0, sky_light: 15, block_light: 0 };
        add_cauldron_quads(&mut mesh, &world, 10, 64, 10, empty_cauldron);
        // 4 legs (4 * 24 = 96) + 1 bottom (24) + 4 walls (4 * 24 = 96) = 216 vertices
        assert_eq!(mesh.vertices.len(), 216);
        assert_eq!(mesh.indices.len(), 324);

        // Water filled cauldron has additional water quad (4 verts)
        let mut mesh_water = MeshData::new();
        let filled_cauldron = Block { id: 118, meta: 3, sky_light: 15, block_light: 0 };
        add_cauldron_quads(&mut mesh_water, &world, 10, 64, 10, filled_cauldron);
        assert_eq!(mesh_water.vertices.len(), 220);
        assert_eq!(mesh_water.indices.len(), 330);
    }

    #[test]
    fn test_beacon_mesh() {
        let world = World::new();
        let mut mesh = MeshData::new();
        let beacon = Block { id: 138, meta: 0, sky_light: 15, block_light: 0 };
        add_beacon_quads(&mut mesh, &world, 10, 64, 10, beacon);
        // Inner diamond core (24) = 24 opaque vertices
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
        // Glass cube casing: 6 faces * 4 vertices = 24 transparent vertices
        assert_eq!(mesh.transparent_vertices.len(), 24);
        assert_eq!(mesh.transparent_indices.len(), 36);
    }

    #[test]
    fn test_enchanting_table_mesh() {
        let world = World::new();
        let mut mesh = MeshData::new();
        let table = Block { id: 116, meta: 0, sky_light: 15, block_light: 0 };
        add_enchanting_table_quads(&mut mesh, &world, 10, 64, 10, table);
        // Base box only (24 vertices, 36 indices) -- book is rendered dynamically
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
    }

    #[test]
    fn test_enchanting_book_model() {
        let mut mesh_closed = MeshData::new();
        build_enchanting_book_model(&mut mesh_closed, 10.0, 64.0, 10.0, 0.0, 0.0, 0.0, [1.0, 1.0, 1.0]);
        // 9 quads (floating open book with 3D page slabs) = 36 vertices, 54 indices
        assert_eq!(mesh_closed.vertices.len(), 36);
        assert_eq!(mesh_closed.indices.len(), 54);

        let mut mesh_open = MeshData::new();
        build_enchanting_book_model(&mut mesh_open, 10.0, 64.0, 10.0, 0.0, 1.0, 0.0, [1.0, 1.0, 1.0]);
        // 9 quads (floating open book with 3D page slabs) = 36 vertices, 54 indices
        assert_eq!(mesh_open.vertices.len(), 36);
        assert_eq!(mesh_open.indices.len(), 54);
    }
}

