use wasm_bindgen::prelude::*;

use std::panic;

// The below modules were made public just so Rust would stop complaining about dead code.
// Conceptually much of the below is basically a library, but it's only used by the `game.ts` file (which is an example, so it doesn't use everything).
mod static_singletons;
pub mod geo;
mod externals;
mod color;
pub mod display_texture;
pub mod display_buffer;
pub mod tiled;
pub mod tiled_display;
pub mod tiled_geometry;
pub mod player;
mod camera;
pub mod mouse;
pub mod keyboard;
pub mod display_text;
mod game;

use console_error_panic_hook;

/// Sets up the whole game system.
/// Must be run before anything else!
#[wasm_bindgen]
pub fn setup(is_little_endian : bool) {
	panic::set_hook(Box::new(console_error_panic_hook::hook));
	static_singletons::set_browser_is_little_endian(is_little_endian);
	static_singletons::create_game();
}

/// Updates the game according to some number of elapsed seconds.
#[wasm_bindgen]
pub fn update(elapsed_seconds : f32) {
	static_singletons::get_game().update(elapsed_seconds);
}

/// Notifies the game that the view window has been resized.
#[wasm_bindgen]
pub fn on_resize(width : u32, height : u32) {
	static_singletons::get_game().on_resize(width, height);
}

/// Notifies the game when a key is pressed.
#[wasm_bindgen]
pub fn on_key_down(key : String) {
	static_singletons::get_game().on_key_down(key);
}

/// Notifies the game when a key is released.
#[wasm_bindgen]
pub fn on_key_up(key : String) {
	static_singletons::get_game().on_key_up(key);
}

/// Notifies the game when the mouse enters the canvas' space.
#[wasm_bindgen]
pub fn on_mouse_enter() {
	static_singletons::get_game().on_mouse_enter();
}

/// Notifies the game when the mouse moves while over the canvas' space.
#[wasm_bindgen]
pub fn on_mouse_update(x : u32, y : u32, buttons : u8) {
	static_singletons::get_game().on_mouse_update(x, y, buttons);
}

/// Notifies the game when the mouse leaves the canvas' space.
#[wasm_bindgen]
pub fn on_mouse_leave() {
	static_singletons::get_game().on_mouse_leave();
}

/// Notifies the game that the gamepad's state has changed.
#[wasm_bindgen]
pub fn on_gamepad_changed(valid : bool, buttons : Vec<f32>, raw_analog_sticks : Vec<f32>) {
	static_singletons::get_game().on_gamepad_changed(valid, buttons, raw_analog_sticks);
}
