/// A place to quarantine all singleton handling.
///
/// Singletons are a necessary evil so JavaScript can easily call into Rust functions and interact with state.

use crate::game::Game;
use crate::tiled::TiledGenerator;

use std::ptr;

/// Whether the browser is little-endian
static mut BROWSER_IS_LITTLE_ENDIAN : bool = false;

/// Sets the "is browser little-endian" value.
pub fn set_browser_is_little_endian(value : bool) {
	unsafe {
		BROWSER_IS_LITTLE_ENDIAN = value;
	}
}

/// Check if the browser is little-endian.
pub fn is_browser_little_endian() -> bool {
	unsafe {
		return BROWSER_IS_LITTLE_ENDIAN;
	}
}

/// The game instance.
/// There's only allowed to be one to make JS calls into Rust/WASM easier.
/// TODO: There's probably a safer way to implement this? Though I don't use threading so far, so it's not required yet.
static mut GAME : *mut Game = ptr::null_mut();

/// Creates the only allowed Game instance.
pub fn create_game() {
	let instance = Box::new(Game::new());
	unsafe {
		GAME = Box::into_raw(instance);
	}
}

/// Gets a mutable reference to the current/only Game instance.
pub fn get_game() -> &'static mut Game {
	unsafe {
		return &mut * GAME;
	}
}

/// The object responsible for loading in Tiled information from its JSON exports.
static mut TILED_FILE_GENERATOR : *mut TiledGenerator = ptr::null_mut();

/// Gets the TiledGenerator instance.
/// Will create one if none exists yet.
pub fn get_tiled_generator() -> &'static mut TiledGenerator {
	unsafe {
		if TILED_FILE_GENERATOR.is_null() {
			TILED_FILE_GENERATOR = Box::into_raw(Box::new(TiledGenerator::new()));
		}
		return &mut *TILED_FILE_GENERATOR;
	}
}
