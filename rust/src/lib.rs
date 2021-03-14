use wasm_bindgen::prelude::*;

// The below modules were made public just so Rust would stop complaining about dead code.
// Conceptually much of the below is basically a library, but it's only used by the `game.ts` file (which is an example, so it doesn't use everything).
pub mod geo;
mod externals;
mod color;
pub mod display_buffer;
mod camera;
pub mod mouse;
pub mod keyboard;
pub mod display_text;
mod bullet;
mod game;
use crate::game::*;

use std::ptr;

/// The game instance.
/// There's only allowed to be one to make JS calls into Rust/WASM easier.
/// TODO: There's probably a safer way to implement this? Though I don't use threading so far, so it's not required yet.
static mut GAME : *mut Game = ptr::null_mut();

/// Sets up the whole game system.
/// Must be run before anything else!
#[wasm_bindgen]
pub fn setup() {
	let instance = Box::new(Game::new());
	unsafe {
		GAME = Box::into_raw(instance);
	}
}

/// Updates the game according to some number of elapsed seconds.
#[wasm_bindgen]
pub fn update(elapsed_seconds : f32) {
	let &mut instance;
	unsafe {
		instance = &mut *GAME;
	}
	instance.update(elapsed_seconds);
}

/// Notifies the game that the view window has been resized.
#[wasm_bindgen]
pub fn on_resize(width : u32, height : u32) {
	let &mut instance;
	unsafe {
		instance = &mut *GAME;
	}
	instance.on_resize(width, height);
}

/// Notifies the game when a key is pressed.
#[wasm_bindgen]
pub fn on_key_down(key : String) {
	let &mut instance;
	unsafe {
		instance = &mut *GAME;
	}
	instance.on_key_down(key);
}

/// Notifies the game when a key is released.
#[wasm_bindgen]
pub fn on_key_up(key : String) {
	let &mut instance;
	unsafe {
		instance = &mut *GAME;
	}
	instance.on_key_up(key);
}

/// Notifies the game when the mouse enters the canvas' space.
#[wasm_bindgen]
pub fn on_mouse_enter() {
	let &mut instance;
	unsafe {
		instance = &mut *GAME;
	}
	instance.on_mouse_enter();
}

/// Notifies the game when the mouse moves while over the canvas' space.
#[wasm_bindgen]
pub fn on_mouse_update(x : u32, y : u32, buttons : u8) {
	let &mut instance;
	unsafe {
		instance = &mut *GAME;
	}
	instance.on_mouse_update(x, y, buttons);
}

/// Notifies the game when the mouse leaves the canvas' space.
#[wasm_bindgen]
pub fn on_mouse_leave() {
	let &mut instance;
	unsafe {
		instance = &mut *GAME;
	}
	instance.on_mouse_leave();
}
