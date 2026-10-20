use godot::prelude::*;

mod bullet;
mod gun;
mod mob;
mod player;
mod survivors_game;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
