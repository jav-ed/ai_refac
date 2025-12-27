use std::path::Path;
use super::utils::create_file;

pub fn generate(root: &Path) -> std::io::Result<()> {
    let base = root.join("rust");
    println!("Generating Complex Rust project (Game Engine Domain)...");

    create_file(&base, "Cargo.toml", r#"[package]
name = "complex_game"
version = "0.1.0"
edition = "2021"

[dependencies]
"#)?;

    // 1. Lib root (File 1)
    create_file(&base, "src/lib.rs", r#"
pub mod engine;
pub mod gameplay;
pub mod utils;
"#)?;

    // 2. Utils (File 2)
    create_file(&base, "src/utils.rs", r#"
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}
"#)?;

    // 3. Engine (File 3, 4)
    create_file(&base, "src/engine/mod.rs", r#"
pub mod physics;
pub mod renderer;

pub struct GameState {
    pub is_running: bool,
    pub frame_count: u64,
}
"#)?;

    create_file(&base, "src/engine/renderer.rs", r#"
use crate::utils::Vector2;

pub struct Texture {
    pub id: u32,
    pub width: u32,
    pub height: u32,
}

pub fn draw_sprite(texture: &Texture, position: Vector2) {
    println!("Drawing texture {} at ({}, {})", texture.id, position.x, position.y);
}
"#)?;
    create_file(&base, "src/engine/physics.rs", "pub fn update() {}")?;

    // 4. Gameplay (File 5, 6)
    create_file(&base, "src/gameplay/mod.rs", "pub mod player;")?;

    create_file(&base, "src/gameplay/player.rs", r#"
use crate::utils::Vector2;
use crate::engine::renderer::Texture;

pub struct Player {
    pub name: String,
    pub health: i32,
    pub position: Vector2,
    pub skin: Texture,
}

impl Player {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            health: 100,
            position: Vector2::zero(),
            skin: Texture { id: 1, width: 64, height: 64 }
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        if self.health < 0 { self.health = 0; }
    }
}
"#)?;

    // 5. Main (File 7)
    create_file(&base, "src/main.rs", r#"
use complex_game::gameplay::player::Player;
use complex_game::engine::{renderer, GameState};

fn main() {
    let mut game = GameState { is_running: true, frame_count: 0 };
    println!("Game starting: running={}", game.is_running);

    let mut hero = Player::new("Hero");
    hero.take_damage(10);

    renderer::draw_sprite(&hero.skin, hero.position);
    
    game.frame_count += 1;
    println!("Player {} HP: {}", hero.name, hero.health);
}
"#)?;

    Ok(())
}
