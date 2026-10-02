mod camera;
mod font;
mod hud;
mod mesh;
mod physics;
mod protocol;
mod raycast;
mod render;
mod skin;
mod sky;
mod texture;
mod world;
mod resource_pack;

use mesh::WorldMeshManager;
use render::RenderState;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};
use world::{Block, ChunkColumn, World};

struct BulkChunkMeta {
    x: i32,
    z: i32,
    primary_bitmap: u16,
    add_bitmap: u16,
}

#[derive(Clone, Debug)]
pub struct RemotePlayer {
    pub entity_id: i32,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Default, Clone, Copy)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub left_mouse: bool,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct MiningState {
    pub active: bool,
    pub block_x: i32,
    pub block_y: i32,
    pub block_z: i32,
    pub face: u8,
    pub ticks: u32,
    pub required_ticks: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SlotItem {
    pub item_id: i16,
    pub count: u8,
    pub damage: i16,
}

pub fn parse_slot_data(buf: &[u8], offset: &mut usize) -> Option<SlotItem> {
    if *offset + 2 > buf.len() {
        return None;
    }
    let item_id = i16::from_be_bytes([buf[*offset], buf[*offset + 1]]);
    *offset += 2;
    if item_id == -1 {
        return None;
    }
    if *offset + 3 > buf.len() {
        return None;
    }
    let count = buf[*offset];
    let damage = i16::from_be_bytes([buf[*offset + 1], buf[*offset + 2]]);
    *offset += 3;
    if *offset + 2 > buf.len() {
        return Some(SlotItem { item_id, count, damage });
    }
    let nbt_len = i16::from_be_bytes([buf[*offset], buf[*offset + 1]]);
    *offset += 2;
    if nbt_len > 0 {
        *offset = (*offset + nbt_len as usize).min(buf.len());
    }
    Some(SlotItem { item_id, count, damage })
}

pub fn item_to_block_id(item_id: i16) -> Option<u16> {
    if item_id > 0 && item_id < 256 {
        return Some(item_id as u16);
    }
    match item_id {
        259 => Some(51),  // Flint and Steel -> Fire
        295 => Some(59),  // Wheat Seeds -> Wheat Crops
        323 => Some(63),  // Sign -> Standing Sign
        324 => Some(64),  // Wooden Door -> Wooden Door Block
        326 => Some(8),   // Water Bucket -> Flowing Water
        327 => Some(10),  // Lava Bucket -> Flowing Lava
        330 => Some(71),  // Iron Door -> Iron Door Block
        331 => Some(55),  // Redstone Dust -> Redstone Wire
        338 => Some(83),  // Reeds / Sugar Canes -> Sugar Canes Block
        354 => Some(92),  // Cake -> Cake Block
        355 => Some(26),  // Bed -> Bed Block
        356 => Some(93),  // Redstone Repeater -> Unpowered Repeater Block
        361 => Some(104), // Pumpkin Seeds -> Pumpkin Stem
        362 => Some(105), // Melon Seeds -> Melon Stem
        372 => Some(115), // Nether Wart -> Nether Wart Block
        379 => Some(117), // Brewing Stand -> Brewing Stand Block
        380 => Some(118), // Cauldron -> Cauldron Block
        390 => Some(140), // Flower Pot -> Flower Pot Block
        391 => Some(141), // Carrot -> Carrots Block
        392 => Some(142), // Potato -> Potatoes Block
        404 => Some(149), // Comparator -> Unpowered Comparator Block
        _ => None,
    }
}

pub fn check_and_activate_end_portal(
    world: &mut World,
    fx: i32,
    fy: i32,
    fz: i32,
    mesh_manager: &mut WorldMeshManager,
) {
    for dx in -2..=2 {
        for dz in -2..=2 {
            let cx = fx + dx;
            let cz = fz + dz;

            let frames = [
                (cx - 1, cz - 2), (cx, cz - 2), (cx + 1, cz - 2),
                (cx - 1, cz + 2), (cx, cz + 2), (cx + 1, cz + 2),
                (cx - 2, cz - 1), (cx - 2, cz), (cx - 2, cz + 1),
                (cx + 2, cz - 1), (cx + 2, cz), (cx + 2, cz + 1),
            ];

            let mut valid = true;
            for (px, pz) in frames {
                let b = world.get_block(px, fy, pz);
                if b.id != 120 || (b.meta & 4) == 0 {
                    valid = false;
                    break;
                }
            }

            if valid {
                for ix in (cx - 1)..=(cx + 1) {
                    for iz in (cz - 1)..=(cz + 1) {
                        world.set_block(
                            ix,
                            fy,
                            iz,
                            Block {
                                id: 119,
                                meta: 0,
                                block_light: 15,
                                sky_light: 15,
                            },
                        );
                        let mcx = ix.div_euclid(16);
                        let mcz = iz.div_euclid(16);
                        mesh_manager.mark_dirty(mcx, mcz);
                    }
                }
                return;
            }
        }
    }
}

pub fn is_interactive_block(id: u16) -> bool {
    matches!(
        id,
        23 | 25 | 26 | 54 | 58 | 61 | 62 | 64 | 69 | 71 | 77 | 84 | 92 | 93 | 94 | 96 | 107
        | 116 | 117 | 130 | 143 | 145 | 146 | 149 | 150 | 151 | 154 | 158 | 167
    )
}

pub fn compute_rail_meta(world: &World, x: i32, y: i32, z: i32, rail_id: u16) -> u8 {
    let is_r = |bx, by, bz| {
        let b = world.get_block(bx, by, bz);
        crate::mesh::is_rail(b.id)
    };
    let has_n = is_r(x, y, z - 1) || is_r(x, y + 1, z - 1) || is_r(x, y - 1, z - 1);
    let has_s = is_r(x, y, z + 1) || is_r(x, y + 1, z + 1) || is_r(x, y - 1, z + 1);
    let has_w = is_r(x - 1, y, z) || is_r(x - 1, y + 1, z) || is_r(x - 1, y - 1, z);
    let has_e = is_r(x + 1, y, z) || is_r(x + 1, y + 1, z) || is_r(x + 1, y - 1, z);

    if rail_id == 66 {
        if has_s && has_e {
            return 6;
        }
        if has_s && has_w {
            return 7;
        }
        if has_n && has_w {
            return 8;
        }
        if has_n && has_e {
            return 9;
        }
    }
    if has_e || has_w {
        1
    } else {
        0
    }
}

pub fn update_neighbor_rails(
    world: &mut World,
    x: i32,
    y: i32,
    z: i32,
    mesh_manager: &mut WorldMeshManager,
) {
    for (nx, nz) in [(x, z - 1), (x, z + 1), (x - 1, z), (x + 1, z)] {
        for ny in [y, y + 1, y - 1] {
            let mut nb = world.get_block(nx, ny, nz);
            if nb.id == 66 {
                let new_m = compute_rail_meta(world, nx, ny, nz, nb.id);
                if new_m != nb.meta {
                    nb.meta = new_m;
                    world.set_block(nx, ny, nz, nb);
                    mesh_manager.mark_dirty(nx.div_euclid(16), nz.div_euclid(16));
                }
            }
        }
    }
}

pub struct PlayerState {
    pub prev_x: f64,
    pub prev_y: f64,
    pub prev_z: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub motion_x: f64,
    pub motion_y: f64,
    pub motion_z: f64,
    pub on_ground: bool,
    pub jump_ticks: u8,
    pub spawned: bool,
    pub input: InputState,
    pub last_w_press: Option<Instant>,
    pub last_tick_time: Instant,
    pub ctrl_held: bool,
    pub mining: MiningState,
    pub camera_mode: u8,
    pub held_slot: u8,
    pub hotbar: [Option<SlotItem>; 9],
    pub sneaking: bool,
    pub sprinting: bool,
    pub health: f32,
    pub food: i16,
    pub food_saturation: f32,
    pub is_dead: bool,
    pub show_debug: bool,
    pub world_time: i64,
}

fn get_break_ticks(block_id: u16) -> u32 {
    match block_id {
        0 => 0,
        31 | 37 | 38 | 50 | 59 | 83 | 175 => 1,
        18 | 161 => 4,
        2 | 3 | 12 | 13 | 82 => 12,
        17 | 162 | 5 | 53 | 134 | 135 | 136 => 24,
        1 | 4 | 24 | 98 | 45 | 48 => 32,
        _ => 20,
    }
}

struct App {
    window: Option<Arc<Window>>,
    render_state: Option<RenderState>,
    world: Arc<Mutex<World>>,
    player: Arc<Mutex<PlayerState>>,
    entities: Arc<Mutex<HashMap<i32, RemotePlayer>>>,
    mesh_manager: WorldMeshManager,
    tx: mpsc::Sender<Vec<u8>>,
    cursor_locked: bool,
    fps_timer: Instant,
    fps_frame_count: u32,
    current_fps: u32,
    pending_skins: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attrs = Window::default_attributes()
                .with_title("Rustcraft - Minecraft 1.7.10 Client")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

            let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
            let render_state = pollster::block_on(RenderState::new(window.clone(), self.pending_skins.clone()));

            self.window = Some(window);
            self.render_state = Some(render_state);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(ref mut rs) = self.render_state {
                    rs.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                let (pos, yaw, pitch, cx, cz, camera_mode, p_x, p_y, p_z, show_debug, eye_height, on_ground, world_time) = {
                    let p = self.player.lock().unwrap();
                    let eye_height = if p.sneaking { 1.54 } else { 1.62 };

                    let elapsed = p.last_tick_time.elapsed().as_secs_f64();
                    let alpha = (elapsed / 0.050).clamp(0.0, 1.0);
                    let interp_x = p.prev_x + (p.x - p.prev_x) * alpha;
                    let interp_y = p.prev_y + (p.y - p.prev_y) * alpha;
                    let interp_z = p.prev_z + (p.z - p.prev_z) * alpha;

                    (
                        glam::Vec3::new(interp_x as f32, (interp_y + eye_height) as f32, interp_z as f32),
                        p.yaw,
                        p.pitch,
                        (interp_x.floor() as i32).div_euclid(16),
                        (interp_z.floor() as i32).div_euclid(16),
                        p.camera_mode,
                        interp_x,
                        interp_y,
                        interp_z,
                        p.show_debug,
                        eye_height,
                        p.on_ground,
                        p.world_time,
                    )
                };

                self.fps_frame_count += 1;
                let elapsed_fps = self.fps_timer.elapsed().as_secs_f32();
                if elapsed_fps >= 0.5 {
                    self.current_fps = (self.fps_frame_count as f32 / elapsed_fps).round() as u32;
                    self.fps_frame_count = 0;
                    self.fps_timer = Instant::now();
                }

                let (cam_pos, cam_yaw, cam_pitch) = match camera_mode {
                    0 => (pos, yaw, pitch),
                    1 => {
                        let dir = raycast::get_look_vector(yaw, pitch);
                        let back = glam::Vec3::new(dir.0 as f32, dir.1 as f32, dir.2 as f32) * 3.5;
                        (pos - back, yaw, pitch)
                    }
                    _ => {
                        let dir = raycast::get_look_vector(yaw, pitch);
                        let front = glam::Vec3::new(dir.0 as f32, dir.1 as f32, dir.2 as f32) * 3.5;
                        (pos + front, yaw + 180.0, -pitch)
                    }
                };

                let updated_meshes = {
                    let mut w = self.world.lock().unwrap();
                    self.mesh_manager.update(&mut w, cx, cz, 4, 4)
                };

                let mut active_players = std::collections::HashSet::new();

                // Render local player model in 3rd person mode
                if camera_mode != 0 {
                    let self_model = mesh::build_player_model(
                        p_x as f32,
                        p_y as f32,
                        p_z as f32,
                        yaw,
                        pitch,
                    );
                    if let Some(ref mut rs) = self.render_state {
                        rs.upload_player_mesh("local".to_string(), &self_model);
                    }
                    active_players.insert("local".to_string());
                }

                // Render remote players
                {
                    let ents = self.entities.lock().unwrap();
                    for entity in ents.values() {
                        let emodel = mesh::build_player_model(
                            entity.x as f32,
                            entity.y as f32,
                            entity.z as f32,
                            entity.yaw,
                            entity.pitch,
                        );
                        if let Some(ref mut rs) = self.render_state {
                            rs.upload_player_mesh(entity.name.clone(), &emodel);
                        }
                        active_players.insert(entity.name.clone());
                    }
                }

                if let Some(ref mut rs) = self.render_state {
                    rs.update_camera(cam_pos, cam_yaw, cam_pitch, world_time);
                    rs.prune_distant_chunks(cx, cz, 6);
                    rs.prune_player_meshes(&active_players);
                    for ((mcx, mcz), mesh_data) in updated_meshes {
                        rs.upload_chunk_mesh(mcx, mcz, &mesh_data);
                    }

                    // Build dynamic enchanting table book models facing the player
                    {
                        let w = self.world.lock().unwrap();
                        let mut book_mesh = mesh::MeshData::new();
                        let player_x = p_x as f32;
                        let player_z = p_z as f32;
                        for &(tx, ty, tz) in &w.enchanting_tables {
                            let ftx = tx as f32;
                            let fty = ty as f32;
                            let ftz = tz as f32;
                            let dx = player_x - (ftx + 0.5);
                            let dz = player_z - (ftz + 0.5);
                            let dist = (dx * dx + dz * dz).sqrt();
                            if dist < 48.0 {
                                let book_yaw = (-dx).atan2(-dz);
                                // Minecraft: opens within 3 blocks
                                let open_factor = if dist <= 2.5 {
                                    1.0
                                } else if dist <= 3.5 {
                                    3.5 - dist
                                } else {
                                    0.0
                                };
                                mesh::build_enchanting_book_model(
                                    &mut book_mesh,
                                    ftx,
                                    fty,
                                    ftz,
                                    book_yaw,
                                    open_factor,
                                );
                            }
                        }
                        rs.upload_entities_mesh(&book_mesh);
                    }

                    if show_debug {
                        let floor_x = p_x.floor() as i32;
                        let floor_y = p_y.floor() as i32;
                        let floor_z = p_z.floor() as i32;
                        let chunk_x = floor_x.div_euclid(16);
                        let chunk_z = floor_z.div_euclid(16);
                        let in_chunk_x = floor_x.rem_euclid(16);
                        let in_chunk_z = floor_z.rem_euclid(16);

                        let facing_index = ((((yaw * 4.0 / 360.0) + 0.5).floor() as i32) & 3).rem_euclid(4);
                        let (facing_name, facing_axis) = match facing_index {
                            0 => ("South", "+Z"),
                            1 => ("West", "-X"),
                            2 => ("North", "-Z"),
                            3 => ("East", "+X"),
                            _ => ("South", "+Z"),
                        };

                        let mut norm_yaw = yaw % 360.0;
                        if norm_yaw > 180.0 {
                            norm_yaw -= 360.0;
                        } else if norm_yaw < -180.0 {
                            norm_yaw += 360.0;
                        }

                        let total_chunks = {
                            let w = self.world.lock().unwrap();
                            w.chunk_count()
                        };
                        let rendered_chunks = rs.chunk_meshes.len();

                        let ent_count = {
                            let e = self.entities.lock().unwrap();
                            e.len()
                        };

                        let (bl, sl) = {
                            let w = self.world.lock().unwrap();
                            let b = w.get_block(floor_x, floor_y, floor_z);
                            (b.block_light, b.sky_light)
                        };
                        let rl = bl.max(sl);

                        let mut lines = vec![
                            format!("Minecraft 1.7.10 ({} fps)", self.current_fps),
                            format!("C: {}/{} (f: 0, st: 0)", rendered_chunks, total_chunks),
                            format!("E: {}/{}", ent_count, ent_count),
                            format!("x: {:.5} ({}) // c: {} ({})", p_x, floor_x, chunk_x, in_chunk_x),
                            format!("y: {:.5} (feet pos, {:.3} eyes pos)", p_y, p_y + eye_height),
                            format!("z: {:.5} ({}) // c: {} ({})", p_z, floor_z, chunk_z, in_chunk_z),
                            format!("f: {} ({}) ({}) / ({:.1} / {:.1})", facing_index, facing_name, facing_axis, norm_yaw, pitch),
                            format!("b: Plains bl: {} sl: {} rl: {}", bl, sl, rl),
                            format!("ws: 0.100, fs: 0.130, g: {}", on_ground),
                        ];

                        {
                            let w = self.world.lock().unwrap();
                            let eye_pos = (p_x, p_y + eye_height, p_z);
                            let dir = raycast::get_look_vector(yaw, pitch);
                            if let Some(hit) = raycast::raycast_world(&w, eye_pos, dir, 5.0) {
                                let block = w.get_block(hit.block_x, hit.block_y, hit.block_z);
                                lines.push(format!("Looking at: {}, {}, {} (id: {}, {})", hit.block_x, hit.block_y, hit.block_z, block.id, block.name()));
                            }
                        }

                        {
                            let p = self.player.lock().unwrap();
                            if let Some(Some(item)) = p.hotbar.get(p.held_slot as usize) {
                                let item_name = crate::world::get_item_name(item.item_id);
                                lines.push(format!("Held item: {} (id: {}, count: {})", item_name, item.item_id, item.count));
                            }
                        }

                        rs.update_hud(&lines);
                    } else {
                        rs.update_hud(&[]);
                    }

                    let _ = rs.render(camera_mode == 0);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                let is_pressed = state == ElementState::Pressed;
                if is_pressed && key_code == KeyCode::Escape {
                    if let Some(ref win) = self.window {
                        let _ = win.set_cursor_grab(CursorGrabMode::None);
                        win.set_cursor_visible(true);
                        self.cursor_locked = false;
                    }
                    return;
                }

                if is_pressed && key_code == KeyCode::F3 {
                    let mut p = self.player.lock().unwrap();
                    p.show_debug = !p.show_debug;
                    return;
                }

                if is_pressed && key_code == KeyCode::F5 {
                    let mut p = self.player.lock().unwrap();
                    p.camera_mode = (p.camera_mode + 1) % 3;
                    return;
                }

                // Hotbar 1-9
                if is_pressed {
                    let slot_opt = match key_code {
                        KeyCode::Digit1 => Some(0),
                        KeyCode::Digit2 => Some(1),
                        KeyCode::Digit3 => Some(2),
                        KeyCode::Digit4 => Some(3),
                        KeyCode::Digit5 => Some(4),
                        KeyCode::Digit6 => Some(5),
                        KeyCode::Digit7 => Some(6),
                        KeyCode::Digit8 => Some(7),
                        KeyCode::Digit9 => Some(8),
                        _ => None,
                    };
                    if let Some(slot) = slot_opt {
                        let mut p = self.player.lock().unwrap();
                        p.held_slot = slot;
                        let mut held_item = Vec::new();
                        protocol::write_varint_sync(&mut held_item, 0x09);
                        protocol::write_i16_sync(&mut held_item, slot as i16);
                        let _ = self.tx.try_send(held_item);
                        return;
                    }
                }

                // Sneak (Shift)
                if key_code == KeyCode::ShiftLeft || key_code == KeyCode::ShiftRight {
                    let mut p = self.player.lock().unwrap();
                    if p.sneaking != is_pressed {
                        p.sneaking = is_pressed;
                        let mut action = Vec::new();
                        protocol::write_varint_sync(&mut action, 0x0B);
                        protocol::write_i32_sync(&mut action, 0);
                        action.push(if is_pressed { 1 } else { 2 });
                        protocol::write_i32_sync(&mut action, 0);
                        let _ = self.tx.try_send(action);
                    }
                }

                // Sprint (Control)
                if key_code == KeyCode::ControlLeft || key_code == KeyCode::ControlRight {
                    let mut p = self.player.lock().unwrap();
                    p.ctrl_held = is_pressed;
                    let should_sprint = is_pressed && p.input.forward;
                    if p.sprinting != should_sprint {
                        p.sprinting = should_sprint;
                        let mut action = Vec::new();
                        protocol::write_varint_sync(&mut action, 0x0B);
                        protocol::write_i32_sync(&mut action, 0);
                        action.push(if should_sprint { 4 } else { 5 });
                        protocol::write_i32_sync(&mut action, 0);
                        let _ = self.tx.try_send(action);
                    }
                }

                let mut p = self.player.lock().unwrap();
                match key_code {
                    KeyCode::KeyW => {
                        let was_pressed = p.input.forward;
                        p.input.forward = is_pressed;

                        if is_pressed && !was_pressed {
                            let now = Instant::now();
                            if let Some(last_press) = p.last_w_press {
                                if now.duration_since(last_press).as_millis() <= 300 {
                                    if !p.sprinting {
                                        p.sprinting = true;
                                        let mut action = Vec::new();
                                        protocol::write_varint_sync(&mut action, 0x0B);
                                        protocol::write_i32_sync(&mut action, 0);
                                        action.push(4);
                                        protocol::write_i32_sync(&mut action, 0);
                                        let _ = self.tx.try_send(action);
                                    }
                                }
                            }
                            p.last_w_press = Some(now);
                        } else if !is_pressed && was_pressed {
                            if p.sprinting && !p.ctrl_held {
                                p.sprinting = false;
                                let mut action = Vec::new();
                                protocol::write_varint_sync(&mut action, 0x0B);
                                protocol::write_i32_sync(&mut action, 0);
                                action.push(5);
                                protocol::write_i32_sync(&mut action, 0);
                                let _ = self.tx.try_send(action);
                            }
                        }
                    }
                    KeyCode::KeyS => p.input.backward = is_pressed,
                    KeyCode::KeyA => p.input.left = is_pressed,
                    KeyCode::KeyD => p.input.right = is_pressed,
                    KeyCode::Space => p.input.jump = is_pressed,
                    _ => {}
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                let is_pressed = state == ElementState::Pressed;

                if is_pressed && !self.cursor_locked {
                    if let Some(ref win) = self.window {
                        let _ = win
                            .set_cursor_grab(CursorGrabMode::Locked)
                            .or_else(|_| win.set_cursor_grab(CursorGrabMode::Confined));
                        win.set_cursor_visible(false);
                        self.cursor_locked = true;
                    }
                }

                let (eye_pos, look) = {
                    let p = self.player.lock().unwrap();
                    let eye_height = if p.sneaking { 1.54 } else { 1.62 };
                    ((p.x, p.y + eye_height, p.z), (p.yaw, p.pitch))
                };
                let dir = raycast::get_look_vector(look.0, look.1);

                match button {
                    MouseButton::Left => {
                        let mut p = self.player.lock().unwrap();
                        p.input.left_mouse = is_pressed;

                        if is_pressed {
                            // Send animation swing packet (0x0A)
                            let mut anim = Vec::new();
                            protocol::write_varint_sync(&mut anim, 0x0A);
                            protocol::write_i32_sync(&mut anim, 0);
                            anim.push(1);
                            let _ = self.tx.try_send(anim);

                            let w = self.world.lock().unwrap();
                            if let Some(hit) = raycast::raycast_world(&w, eye_pos, dir, 5.0) {
                                let block = w.get_block(hit.block_x, hit.block_y, hit.block_z);
                                let req_ticks = get_break_ticks(block.id);

                                let mut start_dig = Vec::new();
                                protocol::write_varint_sync(&mut start_dig, 0x07);
                                start_dig.push(0);
                                protocol::write_i32_sync(&mut start_dig, hit.block_x);
                                start_dig.push(hit.block_y as u8);
                                protocol::write_i32_sync(&mut start_dig, hit.block_z);
                                start_dig.push(hit.face as u8);
                                let _ = self.tx.try_send(start_dig);

                                if req_ticks <= 1 {
                                    let mut finish_dig = Vec::new();
                                    protocol::write_varint_sync(&mut finish_dig, 0x07);
                                    finish_dig.push(2);
                                    protocol::write_i32_sync(&mut finish_dig, hit.block_x);
                                    finish_dig.push(hit.block_y as u8);
                                    protocol::write_i32_sync(&mut finish_dig, hit.block_z);
                                    finish_dig.push(hit.face as u8);
                                    let _ = self.tx.try_send(finish_dig);

                                    drop(w);
                                    let mut w_mut = self.world.lock().unwrap();
                                    w_mut.set_block(hit.block_x, hit.block_y, hit.block_z, Block::AIR);
                                    let cx = hit.block_x.div_euclid(16);
                                    let cz = hit.block_z.div_euclid(16);
                                    self.mesh_manager.mark_dirty(cx, cz);
                                } else {
                                    p.mining = MiningState {
                                        active: true,
                                        block_x: hit.block_x,
                                        block_y: hit.block_y,
                                        block_z: hit.block_z,
                                        face: hit.face as u8,
                                        ticks: 0,
                                        required_ticks: req_ticks,
                                    };
                                }
                            }
                        } else if p.mining.active {
                            let mut cancel_dig = Vec::new();
                            protocol::write_varint_sync(&mut cancel_dig, 0x07);
                            cancel_dig.push(1);
                            protocol::write_i32_sync(&mut cancel_dig, p.mining.block_x);
                            cancel_dig.push(p.mining.block_y as u8);
                            protocol::write_i32_sync(&mut cancel_dig, p.mining.block_z);
                            cancel_dig.push(p.mining.face);
                            let _ = self.tx.try_send(cancel_dig);
                            p.mining.active = false;
                        }
                    }
                    MouseButton::Right => {
                        if is_pressed {
                            let mut w = self.world.lock().unwrap();
                            if let Some(hit) = raycast::raycast_world(&w, eye_pos, dir, 5.0) {
                                let (px, py, pz) = match hit.face {
                                    raycast::BlockFace::Bottom => (hit.block_x, hit.block_y - 1, hit.block_z),
                                    raycast::BlockFace::Top => (hit.block_x, hit.block_y + 1, hit.block_z),
                                    raycast::BlockFace::North => (hit.block_x, hit.block_y, hit.block_z - 1),
                                    raycast::BlockFace::South => (hit.block_x, hit.block_y, hit.block_z + 1),
                                    raycast::BlockFace::West => (hit.block_x - 1, hit.block_y, hit.block_z),
                                    raycast::BlockFace::East => (hit.block_x + 1, hit.block_y, hit.block_z),
                                };

                                let clicked_block = w.get_block(hit.block_x, hit.block_y, hit.block_z);
                                let p = self.player.lock().unwrap();
                                let is_interactive = !p.sneaking && is_interactive_block(clicked_block.id);
                                let held = p.hotbar[p.held_slot as usize];
                                drop(p);

                                let mut place_pkt = Vec::new();
                                protocol::write_varint_sync(&mut place_pkt, 0x08);
                                protocol::write_i32_sync(&mut place_pkt, hit.block_x);
                                place_pkt.push(hit.block_y as u8);
                                protocol::write_i32_sync(&mut place_pkt, hit.block_z);
                                place_pkt.push(hit.face as u8);

                                if let Some(item) = held {
                                    protocol::write_i16_sync(&mut place_pkt, item.item_id);
                                    place_pkt.push(item.count);
                                    protocol::write_i16_sync(&mut place_pkt, item.damage);
                                    protocol::write_i16_sync(&mut place_pkt, -1);
                                } else {
                                    protocol::write_i16_sync(&mut place_pkt, -1);
                                }
                                place_pkt.push(8);
                                place_pkt.push(8);
                                place_pkt.push(8);
                                let _ = self.tx.try_send(place_pkt);

                                if clicked_block.id == 120 && (clicked_block.meta & 4) == 0 && held.map_or(false, |it| it.item_id == 381) {
                                    // Eye of Ender placed into frame
                                    w.set_block(
                                        hit.block_x,
                                        hit.block_y,
                                        hit.block_z,
                                        Block {
                                            id: 120,
                                            meta: clicked_block.meta | 4,
                                            block_light: clicked_block.block_light,
                                            sky_light: clicked_block.sky_light,
                                        },
                                    );
                                    let cx = hit.block_x.div_euclid(16);
                                    let cz = hit.block_z.div_euclid(16);
                                    self.mesh_manager.mark_dirty(cx, cz);
                                    check_and_activate_end_portal(&mut w, hit.block_x, hit.block_y, hit.block_z, &mut self.mesh_manager);
                                } else if !is_interactive {
                                    if let Some(item) = held {
                                        if let Some(block_id) = item_to_block_id(item.item_id) {
                                            let mut meta = if item.item_id < 256 {
                                                (item.damage & 0x0F) as u8
                                            } else {
                                                0
                                            };

                                            let player_yaw = self.player.lock().unwrap().yaw;
                                            if block_id == 86 || block_id == 91 {
                                                // Minecraft pumpkin placement faces the player: (yaw * 4 / 360 + 2.5) & 3
                                                let p_facing = (((player_yaw * 4.0 / 360.0 + 2.5).floor() as i32) % 4 + 4) % 4;
                                                meta = p_facing as u8;
                                            } else if block_id == 61 || block_id == 62 || block_id == 23 || block_id == 158 {
                                                // Furnace / Dispenser / Dropper facing: 2=North, 3=South, 4=West, 5=East
                                                let p_facing = (((player_yaw * 4.0 / 360.0 + 0.5).floor() as i32) % 4 + 4) % 4;
                                                meta = match p_facing {
                                                    0 => 3, // South
                                                    1 => 4, // West
                                                    2 => 2, // North
                                                    3 => 5, // East
                                                    _ => 2,
                                                };
                                            } else if crate::mesh::is_rail(block_id) {
                                                meta = compute_rail_meta(&w, px, py, pz, block_id);
                                            }

                                            let existing = w.get_block(px, py, pz);
                                            w.set_block(
                                                px,
                                                py,
                                                pz,
                                                Block {
                                                    id: block_id,
                                                    meta,
                                                    block_light: existing.block_light,
                                                    sky_light: existing.sky_light,
                                                },
                                            );
                                            let cx = px.div_euclid(16);
                                            let cz = pz.div_euclid(16);
                                            self.mesh_manager.mark_dirty(cx, cz);

                                            if crate::mesh::is_rail(block_id) {
                                                update_neighbor_rails(&mut w, px, py, pz, &mut self.mesh_manager);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if self.cursor_locked {
            if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
                let mut p = self.player.lock().unwrap();
                p.yaw += (dx as f32) * 0.12;
                p.pitch = (p.pitch + (dy as f32) * 0.12).clamp(-89.0, 89.0);
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();
    let args: Vec<String> = std::env::args().collect();
    let mut player_name = "RustPlayer".to_string();
    let mut pack_arg: Option<String> = None;

    let mut i = 1;
    let mut positional = 0;
    while i < args.len() {
        if args[i] == "--pack" || args[i] == "-p" || args[i] == "--resource-pack" {
            if i + 1 < args.len() {
                pack_arg = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if args[i].starts_with("--pack=") {
            pack_arg = Some(args[i]["--pack=".len()..].to_string());
        } else if args[i].starts_with("--resource-pack=") {
            pack_arg = Some(args[i]["--resource-pack=".len()..].to_string());
        } else if !args[i].starts_with('-') {
            if positional == 0 {
                player_name = args[i].clone();
                positional += 1;
            }
        }
        i += 1;
    }

    crate::resource_pack::init(pack_arg.as_deref());

    let now = Instant::now();
    let world = Arc::new(Mutex::new(World::new()));
    let player = Arc::new(Mutex::new(PlayerState {
        prev_x: 0.0,
        prev_y: 0.0,
        prev_z: 0.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        yaw: 0.0,
        pitch: 0.0,
        motion_x: 0.0,
        motion_y: 0.0,
        motion_z: 0.0,
        on_ground: false,
        jump_ticks: 0,
        spawned: false,
        input: InputState::default(),
        last_w_press: None,
        last_tick_time: now,
        ctrl_held: false,
        mining: MiningState::default(),
        camera_mode: 0,
        held_slot: 0,
        hotbar: [None; 9],
        sneaking: false,
        sprinting: false,
        health: 20.0,
        food: 20,
        food_saturation: 5.0,
        is_dead: false,
        show_debug: false,
        world_time: 18000,
    }));
    let entities = Arc::new(Mutex::new(HashMap::<i32, RemotePlayer>::new()));

    let (tx, rx) = mpsc::channel::<Vec<u8>>(256);

    let pending_skins = Arc::new(Mutex::new(Vec::<(String, Vec<u8>)>::new()));

    if skin::load_local_custom_skin().is_none() {
        skin::fetch_skin_async("local".to_string(), None, player_name.clone(), pending_skins.clone());
    }

    let world_net = world.clone();
    let player_net = player.clone();
    let entities_net = entities.clone();
    let tx_net = tx.clone();
    let login_username = player_name.clone();
    let pending_skins_net = pending_skins.clone();

    rt.spawn(async move {
        let _ = run_network_client(world_net, player_net, entities_net, tx_net, rx, login_username, pending_skins_net).await;
    });

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        render_state: None,
        world,
        player,
        entities,
        mesh_manager: WorldMeshManager::new(),
        tx,
        cursor_locked: false,
        fps_timer: Instant::now(),
        fps_frame_count: 0,
        current_fps: 0,
        pending_skins,
    };

    let _ = event_loop.run_app(&mut app);

    Ok(())
}

async fn run_network_client(
    world: Arc<Mutex<World>>,
    player: Arc<Mutex<PlayerState>>,
    entities: Arc<Mutex<HashMap<i32, RemotePlayer>>>,
    tx: mpsc::Sender<Vec<u8>>,
    mut rx: mpsc::Receiver<Vec<u8>>,
    username: String,
    pending_skins: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
) -> std::io::Result<()> {
    println!("Connecting to Minecraft 1.7.10 server as {}...", username);
    let stream = TcpStream::connect("127.0.0.1:25565").await?;
    let (mut read_half, mut write_half) = stream.into_split();
    println!("Connected successfully!");

    tokio::spawn(async move {
        while let Some(packet) = rx.recv().await {
            if protocol::write_varint(&mut write_half, packet.len() as i32).await.is_err() {
                break;
            }
            if write_half.write_all(&packet).await.is_err() {
                break;
            }
        }
    });

    // 1. Handshake
    let mut handshake = Vec::new();
    protocol::write_varint_sync(&mut handshake, 0x00);
    protocol::write_varint_sync(&mut handshake, 5);
    protocol::write_string_sync(&mut handshake, "127.0.0.1");
    protocol::write_ushort_sync(&mut handshake, 25565);
    protocol::write_varint_sync(&mut handshake, 2);
    tx.send(handshake).await.unwrap();

    // 2. Login Start
    let mut login_start = Vec::new();
    protocol::write_varint_sync(&mut login_start, 0x00);
    protocol::write_string_sync(&mut login_start, &username);
    tx.send(login_start).await.unwrap();

    // 3. Login Response
    let _packet_len = protocol::read_varint(&mut read_half).await?;
    let packet_id = protocol::read_varint(&mut read_half).await?;

    if packet_id != 0x02 {
        println!("Failed to log in. Packet ID: 0x{:02X}", packet_id);
        return Ok(());
    }

    let uuid = protocol::read_string(&mut read_half).await?;
    let logged_username = protocol::read_string(&mut read_half).await?;
    println!("SUCCESS: Logged in as {} (UUID: {})", logged_username, uuid);

    // 20Hz Player Tick Task
    let tx_tick = tx.clone();
    let player_tick = player.clone();
    let world_tick = world.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            interval.tick().await;

            let mut block_to_break = None;

            let result = {
                let mut p = player_tick.lock().unwrap();
                if !p.spawned || p.is_dead {
                    None
                } else {
                    let world_guard = world_tick.lock().unwrap();
                    let current_chunk_x = (p.x.floor() as i32).div_euclid(16);
                    let current_chunk_z = (p.z.floor() as i32).div_euclid(16);
                    let chunk_loaded = world_guard.get_chunk(current_chunk_x, current_chunk_z).is_some();

                    if !chunk_loaded {
                        p.motion_x = 0.0;
                        p.motion_y = 0.0;
                        p.motion_z = 0.0;
                    } else {
                        let res = physics::tick_movement(
                            &world_guard,
                            p.x,
                            p.y,
                            p.z,
                            p.motion_x,
                            p.motion_y,
                            p.motion_z,
                            p.yaw,
                            p.pitch,
                            p.on_ground,
                            p.input.forward,
                            p.input.backward,
                            p.input.left,
                            p.input.right,
                            if p.jump_ticks == 0 { p.input.jump } else { false },
                            p.sneaking,
                            p.sprinting,
                        );
                        
                        if !p.input.jump { p.jump_ticks = 0; }

                        p.prev_x = p.x;
                        p.prev_y = p.y;
                        p.prev_z = p.z;
                        p.x = res.new_x;
                        p.y = res.new_y;
                        p.z = res.new_z;
                        p.motion_x = res.new_motion_x;
                        p.motion_y = res.new_motion_y;
                        p.motion_z = res.new_motion_z;
                        p.on_ground = res.on_ground;
                        
                        if res.did_jump {
                            p.jump_ticks = 10;
                        } else if p.jump_ticks > 0 {
                            p.jump_ticks -= 1;
                        }
                        p.world_time = p.world_time.wrapping_add(1);
                        p.last_tick_time = Instant::now();

                        let pbox = physics::AABB::for_player(res.new_x, res.new_y, res.new_z);
                        let in_liquid = physics::is_in_water(&world_guard, &pbox) || physics::is_in_lava(&world_guard, &pbox);

                        if (in_liquid || (res.collided_horizontally && !p.ctrl_held)) && p.sprinting {
                            p.sprinting = false;
                            let mut action = Vec::new();
                            protocol::write_varint_sync(&mut action, 0x0B);
                            protocol::write_i32_sync(&mut action, 0);
                            action.push(5);
                            protocol::write_i32_sync(&mut action, 0);
                            let _ = tx_tick.try_send(action);
                        }
                    }

                    if p.mining.active {
                        if !p.input.left_mouse {
                            p.mining.active = false;
                        } else {
                            p.mining.ticks += 1;
                            if p.mining.ticks >= p.mining.required_ticks {
                                block_to_break = Some((p.mining.block_x, p.mining.block_y, p.mining.block_z, p.mining.face));
                                p.mining.active = false;
                            }
                        }
                    }

                    drop(world_guard);

                    let eye_height = if p.sneaking { 1.54 } else { 1.62 };
                    let mut pos_look = Vec::new();
                    protocol::write_varint_sync(&mut pos_look, 0x06);
                    pos_look.extend_from_slice(&p.x.to_be_bytes());
                    pos_look.extend_from_slice(&p.y.to_be_bytes());
                    let stance = p.y + eye_height;
                    pos_look.extend_from_slice(&stance.to_be_bytes());
                    pos_look.extend_from_slice(&p.z.to_be_bytes());
                    pos_look.extend_from_slice(&p.yaw.to_be_bytes());
                    pos_look.extend_from_slice(&p.pitch.to_be_bytes());
                    pos_look.push(if p.on_ground { 1 } else { 0 });

                    Some(pos_look)
                }
            };

            if let Some((bx, by, bz, face)) = block_to_break {
                let mut finish_dig = Vec::new();
                protocol::write_varint_sync(&mut finish_dig, 0x07);
                finish_dig.push(2);
                protocol::write_i32_sync(&mut finish_dig, bx);
                finish_dig.push(by as u8);
                protocol::write_i32_sync(&mut finish_dig, bz);
                finish_dig.push(face);
                let _ = tx_tick.send(finish_dig).await;

                let mut w = world_tick.lock().unwrap();
                w.set_block(bx, by, bz, Block::AIR);
            }

            if let Some(pos_look) = result {
                if tx_tick.send(pos_look).await.is_err() {
                    break;
                }
            }
        }
    });

    // 4. Main Packet Reader Loop
    loop {
        let packet_length = match protocol::read_varint(&mut read_half).await {
            Ok(len) => len,
            Err(_) => {
                println!("Disconnected from server.");
                break;
            }
        };

        let packet_id = protocol::read_varint(&mut read_half).await?;

        let id_length = if packet_id == 0 {
            1
        } else {
            let mut tmp = Vec::new();
            protocol::write_varint_sync(&mut tmp, packet_id);
            tmp.len() as i32
        };
        let payload_len = (packet_length - id_length).max(0) as usize;

        match packet_id {
            0x00 => {
                let keep_alive_id = protocol::read_i32(&mut read_half).await?;
                let mut resp = Vec::new();
                protocol::write_varint_sync(&mut resp, 0x00);
                protocol::write_i32_sync(&mut resp, keep_alive_id);
                let _ = tx.send(resp).await;
            }
            0x01 => {
                let entity_id = protocol::read_i32(&mut read_half).await?;
                println!("[PACKET] Join Game received. Entity ID: {}", entity_id);

                let remaining = payload_len.saturating_sub(4);
                let mut buf = vec![0u8; remaining];
                read_half.read_exact(&mut buf).await?;

                // 1. Send Client Settings (0x15)
                let mut client_settings = Vec::new();
                protocol::write_varint_sync(&mut client_settings, 0x15);
                protocol::write_string_sync(&mut client_settings, "en_US");
                client_settings.push(8);
                client_settings.push(0);
                client_settings.push(1);
                client_settings.push(2);
                client_settings.push(1);
                let _ = tx.send(client_settings).await;

                // 2. Send Plugin Message MC|Brand (0x17)
                let mut brand = Vec::new();
                protocol::write_varint_sync(&mut brand, 0x17);
                protocol::write_string_sync(&mut brand, "MC|Brand");
                let brand_name = "vanilla";
                protocol::write_i16_sync(&mut brand, brand_name.len() as i16);
                brand.extend_from_slice(brand_name.as_bytes());
                let _ = tx.send(brand).await;

                // 3. Send Held Item Change (0x09)
                let mut held_item = Vec::new();
                protocol::write_varint_sync(&mut held_item, 0x09);
                protocol::write_i16_sync(&mut held_item, 0);
                let _ = tx.send(held_item).await;
            }
            0x02 => {
                let chat_json = protocol::read_string(&mut read_half).await?;
                println!("[CHAT] {}", chat_json);
            }
            0x03 => {
                let mut buf8 = [0u8; 8];
                read_half.read_exact(&mut buf8).await?;
                let _world_age = i64::from_be_bytes(buf8);
                read_half.read_exact(&mut buf8).await?;
                let time_of_day = i64::from_be_bytes(buf8);
                let time = if time_of_day < 0 { -time_of_day } else { time_of_day };
                let mut p = player.lock().unwrap();
                p.world_time = time;
                println!("[TIME] Server time updated: {}", time % 24000);
            }
            0x06 => {
                let mut payload = vec![0u8; payload_len];
                read_half.read_exact(&mut payload).await?;
                let mut offset = 0;
                if let (Ok(health_bits), Ok(food), Ok(sat_bits)) = (
                    protocol::read_i32_slice(&payload, &mut offset),
                    protocol::read_i16_slice(&payload, &mut offset),
                    protocol::read_i32_slice(&payload, &mut offset),
                ) {
                    let health = f32::from_bits(health_bits as u32);
                    let sat = f32::from_bits(sat_bits as u32);
                    println!("[HEALTH] Health: {:.1}/20, Food: {}, Saturation: {:.1}", health, food, sat);

                    let send_respawn = {
                        let mut p = player.lock().unwrap();
                        p.health = health;
                        p.food = food;
                        p.food_saturation = sat;

                        if health <= 0.0 && !p.is_dead {
                            p.is_dead = true;
                            println!("[PLAYER] Player died. Auto-requesting respawn...");
                            true
                        } else {
                            if health > 0.0 {
                                p.is_dead = false;
                            }
                            false
                        }
                    };

                    if send_respawn {
                        let mut respawn = Vec::new();
                        protocol::write_varint_sync(&mut respawn, 0x16);
                        protocol::write_varint_sync(&mut respawn, 0);
                        let _ = tx.send(respawn).await;
                    }
                }
            }
            0x07 => {
                let mut payload = vec![0u8; payload_len];
                read_half.read_exact(&mut payload).await?;
                let mut offset = 0;
                let dimension = protocol::read_i32_slice(&payload, &mut offset).unwrap_or(0);
                let difficulty = protocol::read_u8_slice(&payload, &mut offset).unwrap_or(0);
                let gamemode = protocol::read_u8_slice(&payload, &mut offset).unwrap_or(0);
                let level_type = protocol::read_string_slice(&payload, &mut offset).unwrap_or_default();
                println!("[RESPAWN] Respawned. Dimension: {}, Diff: {}, Mode: {}, Type: {}", dimension, difficulty, gamemode, level_type);

                let mut p = player.lock().unwrap();
                p.is_dead = false;
                p.health = 20.0;
                p.motion_x = 0.0;
                p.motion_y = 0.0;
                p.motion_z = 0.0;
                p.last_tick_time = Instant::now();
            }
            0x08 => {
                let mut buf8 = [0u8; 8];
                read_half.read_exact(&mut buf8).await?;
                let x = f64::from_be_bytes(buf8);
                read_half.read_exact(&mut buf8).await?;
                let server_y = f64::from_be_bytes(buf8);
                read_half.read_exact(&mut buf8).await?;
                let z = f64::from_be_bytes(buf8);

                let mut buf4 = [0u8; 4];
                read_half.read_exact(&mut buf4).await?;
                let yaw = f32::from_be_bytes(buf4);
                read_half.read_exact(&mut buf4).await?;
                let pitch = f32::from_be_bytes(buf4);

                let on_ground = read_half.read_u8().await? != 0;

                let bytes_read = 8 + 8 + 8 + 4 + 4 + 1;
                let remaining = payload_len.saturating_sub(bytes_read);
                if remaining > 0 {
                    let mut buf = vec![0u8; remaining];
                    read_half.read_exact(&mut buf).await?;
                }

                let feet_y = server_y - 1.62;

                {
                    let mut p = player.lock().unwrap();
                    p.prev_x = x;
                    p.prev_y = feet_y;
                    p.prev_z = z;
                    p.x = x;
                    p.y = feet_y;
                    p.z = z;
                    p.yaw = yaw;
                    p.pitch = pitch;
                    p.on_ground = on_ground;
                    p.motion_x = 0.0;
                    p.motion_y = 0.0;
                    p.motion_z = 0.0;
                    p.spawned = true;
                    p.is_dead = false;
                    p.last_tick_time = Instant::now();
                }

                println!(
                    "[SPAWN] Position: X: {:.2}, Y: {:.2} (Feet: {:.2}), Z: {:.2} (Yaw: {:.1}, Pitch: {:.1})",
                    x, server_y, feet_y, z, yaw, pitch
                );

                let mut resp = Vec::new();
                protocol::write_varint_sync(&mut resp, 0x06);
                resp.extend_from_slice(&x.to_be_bytes());
                resp.extend_from_slice(&feet_y.to_be_bytes());
                let stance = feet_y + 1.62;
                resp.extend_from_slice(&stance.to_be_bytes());
                resp.extend_from_slice(&z.to_be_bytes());
                resp.extend_from_slice(&yaw.to_be_bytes());
                resp.extend_from_slice(&pitch.to_be_bytes());
                resp.push(if on_ground { 1 } else { 0 });
                let _ = tx.send(resp).await;
            }
            0x0C => {
                let mut payload = vec![0u8; payload_len];
                read_half.read_exact(&mut payload).await?;

                let mut offset = 0;
                if let Ok(eid) = protocol::read_varint_slice(&payload, &mut offset) {
                    if let Ok(_uuid) = protocol::read_string_slice(&payload, &mut offset) {
                        if let Ok(name) = protocol::read_string_slice(&payload, &mut offset) {
                            if let Ok(prop_count) = protocol::read_varint_slice(&payload, &mut offset) {
                                let mut skin_url = None;
                                for _ in 0..prop_count {
                                    let prop_name = protocol::read_string_slice(&payload, &mut offset).unwrap_or_default();
                                    let prop_val = protocol::read_string_slice(&payload, &mut offset).unwrap_or_default();
                                    let _prop_sig = protocol::read_string_slice(&payload, &mut offset).unwrap_or_default();
                                    if prop_name == "textures" {
                                        use base64::Engine;
                                        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(prop_val.as_bytes()) {
                                            if let Ok(json_str) = String::from_utf8(decoded) {
                                                if let Some(pos) = json_str.find("\"url\"") {
                                                    let rest = &json_str[pos..];
                                                    if let Some(start_http) = rest.find("http") {
                                                        if let Some(end_quote) = rest[start_http..].find('"') {
                                                            skin_url = Some(rest[start_http..start_http + end_quote].to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if let (Ok(x_fixed), Ok(y_fixed), Ok(z_fixed)) = (
                                    protocol::read_i32_slice(&payload, &mut offset),
                                    protocol::read_i32_slice(&payload, &mut offset),
                                    protocol::read_i32_slice(&payload, &mut offset),
                                ) {
                                    let yaw_byte = protocol::read_i8_slice(&payload, &mut offset).unwrap_or(0);
                                    let pitch_byte = protocol::read_i8_slice(&payload, &mut offset).unwrap_or(0);
                                    let x = x_fixed as f64 / 32.0;
                                    let y = y_fixed as f64 / 32.0;
                                    let z = z_fixed as f64 / 32.0;
                                    let yaw = (yaw_byte as f32) * 360.0 / 256.0;
                                    let pitch = (pitch_byte as f32) * 360.0 / 256.0;

                                    println!("[SPAWN PLAYER] ID: {}, Name: {}, Pos: ({:.2}, {:.2}, {:.2})", eid, name, x, y, z);
                                    skin::fetch_skin_async(name.clone(), skin_url, name.clone(), pending_skins.clone());
                                    let mut ents = entities.lock().unwrap();
                                    ents.insert(
                                        eid,
                                        RemotePlayer {
                                            entity_id: eid,
                                            name,
                                            x,
                                            y,
                                            z,
                                            yaw,
                                            pitch,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
            0x13 => {
                let mut payload = vec![0u8; payload_len];
                read_half.read_exact(&mut payload).await?;

                let mut offset = 0;
                if let Ok(count) = protocol::read_u8_slice(&payload, &mut offset) {
                    let mut ents = entities.lock().unwrap();
                    for _ in 0..count {
                        if let Ok(eid) = protocol::read_i32_slice(&payload, &mut offset) {
                            ents.remove(&eid);
                        }
                    }
                }
            }
            0x15 => {
                let mut payload = vec![0u8; payload_len];
                read_half.read_exact(&mut payload).await?;

                let mut offset = 0;
                if let Ok(eid) = protocol::read_i32_slice(&payload, &mut offset) {
                    if let (Ok(dx), Ok(dy), Ok(dz)) = (
                        protocol::read_i8_slice(&payload, &mut offset),
                        protocol::read_i8_slice(&payload, &mut offset),
                        protocol::read_i8_slice(&payload, &mut offset),
                    ) {
                        let mut ents = entities.lock().unwrap();
                        if let Some(player) = ents.get_mut(&eid) {
                            player.x += dx as f64 / 32.0;
                            player.y += dy as f64 / 32.0;
                            player.z += dz as f64 / 32.0;
                        }
                    }
                }
            }
            0x16 => {
                let mut payload = vec![0u8; payload_len];
                read_half.read_exact(&mut payload).await?;

                let mut offset = 0;
                if let Ok(eid) = protocol::read_i32_slice(&payload, &mut offset) {
                    if let (Ok(yaw_b), Ok(pitch_b)) = (
                        protocol::read_i8_slice(&payload, &mut offset),
                        protocol::read_i8_slice(&payload, &mut offset),
                    ) {
                        let mut ents = entities.lock().unwrap();
                        if let Some(player) = ents.get_mut(&eid) {
                            player.yaw = (yaw_b as f32) * 360.0 / 256.0;
                            player.pitch = (pitch_b as f32) * 360.0 / 256.0;
                        }
                    }
                }
            }
            0x17 => {
                let mut payload = vec![0u8; payload_len];
                read_half.read_exact(&mut payload).await?;

                let mut offset = 0;
                if let Ok(eid) = protocol::read_i32_slice(&payload, &mut offset) {
                    if let (Ok(dx), Ok(dy), Ok(dz), Ok(yaw_b), Ok(pitch_b)) = (
                        protocol::read_i8_slice(&payload, &mut offset),
                        protocol::read_i8_slice(&payload, &mut offset),
                        protocol::read_i8_slice(&payload, &mut offset),
                        protocol::read_i8_slice(&payload, &mut offset),
                        protocol::read_i8_slice(&payload, &mut offset),
                    ) {
                        let mut ents = entities.lock().unwrap();
                        if let Some(player) = ents.get_mut(&eid) {
                            player.x += dx as f64 / 32.0;
                            player.y += dy as f64 / 32.0;
                            player.z += dz as f64 / 32.0;
                            player.yaw = (yaw_b as f32) * 360.0 / 256.0;
                            player.pitch = (pitch_b as f32) * 360.0 / 256.0;
                        }
                    }
                }
            }
            0x18 => {
                let mut payload = vec![0u8; payload_len];
                read_half.read_exact(&mut payload).await?;

                let mut offset = 0;
                if let Ok(eid) = protocol::read_i32_slice(&payload, &mut offset) {
                    if let (Ok(x_fixed), Ok(y_fixed), Ok(z_fixed), Ok(yaw_b), Ok(pitch_b)) = (
                        protocol::read_i32_slice(&payload, &mut offset),
                        protocol::read_i32_slice(&payload, &mut offset),
                        protocol::read_i32_slice(&payload, &mut offset),
                        protocol::read_i8_slice(&payload, &mut offset),
                        protocol::read_i8_slice(&payload, &mut offset),
                    ) {
                        let mut ents = entities.lock().unwrap();
                        if let Some(player) = ents.get_mut(&eid) {
                            player.x = x_fixed as f64 / 32.0;
                            player.y = y_fixed as f64 / 32.0;
                            player.z = z_fixed as f64 / 32.0;
                            player.yaw = (yaw_b as f32) * 360.0 / 256.0;
                            player.pitch = (pitch_b as f32) * 360.0 / 256.0;
                        }
                    }
                }
            }
            0x19 => {
                let mut payload = vec![0u8; payload_len];
                read_half.read_exact(&mut payload).await?;

                let mut offset = 0;
                if let Ok(eid) = protocol::read_i32_slice(&payload, &mut offset) {
                    if let Ok(head_yaw_b) = protocol::read_i8_slice(&payload, &mut offset) {
                        let mut ents = entities.lock().unwrap();
                        if let Some(player) = ents.get_mut(&eid) {
                            player.yaw = (head_yaw_b as f32) * 360.0 / 256.0;
                        }
                    }
                }
            }
            0x21 => {
                let chunk_x = read_half.read_i32().await?;
                let chunk_z = read_half.read_i32().await?;
                let ground_up_continuous = read_half.read_u8().await? != 0;
                let primary_bitmap = read_half.read_u16().await?;
                let add_bitmap = read_half.read_u16().await?;
                let compressed_size = read_half.read_i32().await?;

                let mut compressed_data = vec![0u8; compressed_size as usize];
                read_half.read_exact(&mut compressed_data).await?;

                if ground_up_continuous && primary_bitmap == 0 {
                    let mut w = world.lock().unwrap();
                    w.remove_chunk(chunk_x, chunk_z);
                } else {
                    let mut decoder = flate2::read::ZlibDecoder::new(&compressed_data[..]);
                    let mut uncompressed = Vec::new();

                    if decoder.read_to_end(&mut uncompressed).is_ok() {
                        match ChunkColumn::parse_column_data(
                            &uncompressed,
                            chunk_x,
                            chunk_z,
                            primary_bitmap,
                            add_bitmap,
                            true,
                            ground_up_continuous,
                        ) {
                            Ok((column, _)) => {
                                let mut w = world.lock().unwrap();
                                w.insert_chunk(column);
                            }
                            Err(e) => {
                                eprintln!("[CHUNK 0x21 ERROR] ({},{}): {}", chunk_x, chunk_z, e);
                            }
                        }
                    }
                }

                let bytes_read = 4 + 4 + 1 + 2 + 2 + 4 + compressed_size as usize;
                let remaining = payload_len.saturating_sub(bytes_read);
                if remaining > 0 {
                    let mut buf = vec![0u8; remaining];
                    read_half.read_exact(&mut buf).await?;
                }
            }
            0x22 => {
                let chunk_x = protocol::read_i32(&mut read_half).await?;
                let chunk_z = protocol::read_i32(&mut read_half).await?;
                let record_count = read_half.read_i16().await?;
                let data_size = protocol::read_i32(&mut read_half).await?;

                let mut record_data = vec![0u8; data_size as usize];
                read_half.read_exact(&mut record_data).await?;

                {
                    let mut w = world.lock().unwrap();
                    for i in 0..(record_count as usize) {
                        let offset = i * 4;
                        if offset + 4 <= record_data.len() {
                            let horizontal = record_data[offset];
                            let local_x = ((horizontal >> 4) & 0x0F) as i32;
                            let local_z = (horizontal & 0x0F) as i32;
                            let y = record_data[offset + 1] as i32;
                            let block_data =
                                ((record_data[offset + 2] as u16) << 8) | (record_data[offset + 3] as u16);
                            let block_id = block_data >> 4;
                            let block_meta = (block_data & 0x0F) as u8;

                            let wx = chunk_x * 16 + local_x;
                            let wz = chunk_z * 16 + local_z;
                            let existing = w.get_block(wx, y, wz);
                            w.set_block(
                                wx,
                                y,
                                wz,
                                Block {
                                    id: block_id,
                                    meta: block_meta,
                                    block_light: existing.block_light,
                                    sky_light: existing.sky_light,
                                },
                            );
                        }
                    }
                }

                let bytes_read = 4 + 4 + 2 + 4 + data_size as usize;
                let remaining = payload_len.saturating_sub(bytes_read);
                if remaining > 0 {
                    let mut buf = vec![0u8; remaining];
                    read_half.read_exact(&mut buf).await?;
                }
            }
            0x23 => {
                let bx = protocol::read_i32(&mut read_half).await?;
                let by = read_half.read_u8().await? as i32;
                let bz = protocol::read_i32(&mut read_half).await?;
                let block_id = protocol::read_varint(&mut read_half).await? as u16;
                let block_meta = read_half.read_u8().await?;

                {
                    let mut w = world.lock().unwrap();
                    let existing = w.get_block(bx, by, bz);
                    w.set_block(
                        bx,
                        by,
                        bz,
                        Block {
                            id: block_id,
                            meta: block_meta,
                            block_light: existing.block_light,
                            sky_light: existing.sky_light,
                        },
                    );
                }

                let id_len = if block_id == 0 {
                    1
                } else {
                    let mut tmp = Vec::new();
                    protocol::write_varint_sync(&mut tmp, block_id as i32);
                    tmp.len()
                };
                let bytes_read = 4 + 1 + 4 + id_len + 1;
                let remaining = payload_len.saturating_sub(bytes_read);
                if remaining > 0 {
                    let mut buf = vec![0u8; remaining];
                    read_half.read_exact(&mut buf).await?;
                }
            }
            0x26 => {
                let chunk_count = read_half.read_i16().await?;
                let data_length = read_half.read_i32().await?;
                let sky_light = read_half.read_u8().await? != 0;

                let mut compressed_data = vec![0u8; data_length as usize];
                read_half.read_exact(&mut compressed_data).await?;

                let mut chunk_metas = Vec::new();
                for _ in 0..chunk_count {
                    let x = read_half.read_i32().await?;
                    let z = read_half.read_i32().await?;
                    let primary_bitmap = read_half.read_u16().await?;
                    let add_bitmap = read_half.read_u16().await?;
                    chunk_metas.push(BulkChunkMeta {
                        x,
                        z,
                        primary_bitmap,
                        add_bitmap,
                    });
                }

                let mut decoder = flate2::read::ZlibDecoder::new(&compressed_data[..]);
                let mut uncompressed = Vec::new();

                if decoder.read_to_end(&mut uncompressed).is_ok() {
                    let mut offset = 0;
                    let mut w = world.lock().unwrap();

                    for meta in chunk_metas {
                        if offset >= uncompressed.len() {
                            break;
                        }
                        match ChunkColumn::parse_column_data(
                            &uncompressed[offset..],
                            meta.x,
                            meta.z,
                            meta.primary_bitmap,
                            meta.add_bitmap,
                            sky_light,
                            true,
                        ) {
                            Ok((column, bytes_consumed)) => {
                                offset += bytes_consumed;
                                w.insert_chunk(column);
                            }
                            Err(e) => {
                                eprintln!("[CHUNK 0x26 ERROR] ({},{}): {}", meta.x, meta.z, e);
                                break;
                            }
                        }
                    }
                }

                let bytes_read = 2 + 4 + 1 + data_length as usize + (chunk_count as usize * 12);
                let remaining = payload_len.saturating_sub(bytes_read);
                if remaining > 0 {
                    let mut buf = vec![0u8; remaining];
                    read_half.read_exact(&mut buf).await?;
                }
            }
            0x2F => {
                let mut payload = vec![0u8; payload_len];
                read_half.read_exact(&mut payload).await?;
                let mut offset = 0;
                if let (Ok(window_id), Ok(slot)) = (
                    protocol::read_u8_slice(&payload, &mut offset),
                    protocol::read_i16_slice(&payload, &mut offset),
                ) {
                    let item = parse_slot_data(&payload, &mut offset);
                    if window_id == 0 && slot >= 36 && slot <= 44 {
                        let hotbar_idx = (slot - 36) as usize;
                        let mut p = player.lock().unwrap();
                        p.hotbar[hotbar_idx] = item;
                    }
                }
            }
            0x30 => {
                let mut payload = vec![0u8; payload_len];
                read_half.read_exact(&mut payload).await?;
                let mut offset = 0;
                if let (Ok(window_id), Ok(count)) = (
                    protocol::read_u8_slice(&payload, &mut offset),
                    protocol::read_i16_slice(&payload, &mut offset),
                ) {
                    if window_id == 0 {
                        let mut p = player.lock().unwrap();
                        for slot_idx in 0..count {
                            let item = parse_slot_data(&payload, &mut offset);
                            if slot_idx >= 36 && slot_idx <= 44 {
                                p.hotbar[(slot_idx - 36) as usize] = item;
                            }
                        }
                    }
                }
            }
            0x40 => {
                let reason = protocol::read_string(&mut read_half).await?;
                println!("[DISCONNECT] Server kicked client: {}", reason);
                break;
            }
            _ => {
                let mut buf = vec![0u8; payload_len];
                read_half.read_exact(&mut buf).await?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_to_block_id() {
        assert_eq!(item_to_block_id(1), Some(1)); // Stone block
        assert_eq!(item_to_block_id(120), Some(120)); // End Portal Frame block
        assert_eq!(item_to_block_id(259), Some(51)); // Flint and Steel -> Fire
        assert_eq!(item_to_block_id(295), Some(59)); // Seeds -> Wheat
        assert_eq!(item_to_block_id(324), Some(64)); // Wood Door
        assert_eq!(item_to_block_id(326), Some(8)); // Water Bucket -> Water
        assert_eq!(item_to_block_id(327), Some(10)); // Lava Bucket -> Lava
        assert_eq!(item_to_block_id(330), Some(71)); // Iron Door
        assert_eq!(item_to_block_id(331), Some(55)); // Redstone Dust -> Wire
        assert_eq!(item_to_block_id(355), Some(26)); // Bed
        assert_eq!(item_to_block_id(356), Some(93)); // Repeater
        assert_eq!(item_to_block_id(381), None); // Eye of Ender (handled specially)
        assert_eq!(item_to_block_id(404), Some(149)); // Comparator
    }

    #[test]
    fn test_end_portal_activation() {
        let mut world = World::new();
        world.insert_chunk(ChunkColumn::new(0, 0));
        let mut mesh_manager = WorldMeshManager::new();

        // 3x3 portal at x: 4..=6, z: 4..=6, y: 64 (center is 5, 5)
        let frames = [
            (4, 3), (5, 3), (6, 3), // North
            (4, 7), (5, 7), (6, 7), // South
            (3, 4), (3, 5), (3, 6), // West
            (7, 4), (7, 5), (7, 6), // East
        ];

        // Place 11 frames with eyes, 1 without eye
        for (i, &(fx, fz)) in frames.iter().enumerate() {
            let meta = if i == 0 { 0 } else { 4 };
            world.set_block(fx, 64, fz, Block { id: 120, meta, block_light: 0, sky_light: 15 });
        }

        // Initially center is air (id: 0)
        assert_eq!(world.get_block(5, 64, 5).id, 0);

        // Place 12th eye on the first frame
        let (fx, fz) = frames[0];
        world.set_block(fx, 64, fz, Block { id: 120, meta: 4, block_light: 0, sky_light: 15 });
        check_and_activate_end_portal(&mut world, fx, 64, fz, &mut mesh_manager);

        // Now all 9 interior blocks must be activated End Portal (id: 119) with light level 15
        for ix in 4..=6 {
            for iz in 4..=6 {
                let b = world.get_block(ix, 64, iz);
                assert_eq!(b.id, 119, "Expected End Portal at {}, 64, {}", ix, iz);
                assert_eq!(b.block_light, 15);
            }
        }
    }
}
