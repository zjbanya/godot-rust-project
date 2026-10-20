use godot::prelude::*;

mod bullet;
mod gun;
mod mob;
mod player;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
