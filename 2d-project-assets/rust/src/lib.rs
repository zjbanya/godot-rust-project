use godot::prelude::*;

mod gun;
mod mob;
mod player;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
