use crate::geo::vec3::*;
use crate::camera::*;
use crate::geo::consts::*;

/// The mouse button. Values map to the values JS/DOM uses.
#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
	LEFT = 1,
	RIGHT = 2,
	MIDDLE = 4,
}

/// The mouse object.
pub struct Mouse {
	position : Vec3, // The z-position is currently junk.
	on_screen : bool, // Whether the mouse is on screen.
	button_state : u8, // The exact current state of the left, middle, and right buttons.
	// TODO? Make it record where the mouse is when clicked and released? Debouncing?
	changed : bool, // Whether it has changed since last checked.
}

impl Mouse {
	/// Creates a new instance.
	pub fn new() -> Mouse {
		Mouse {
			position: Vec3::zero(),
			changed: false,
			on_screen: false,
			button_state: 0,
		}
	}

	/// Returns if the mouse state has changed since the last time this was called.
	pub fn has_changed_since(&mut self) -> bool {
		let changed = self.changed;
		self.changed = false;
		changed
	}

	/// Returns the position of hte mouse.
	pub fn position(&self) -> Vec3 {
		self.position.clone()
	}

	/// Whether the mouse is currently on screen.
	pub fn is_on_screen(&self) -> bool {
		self.on_screen
	}

	/// Checks if a given button is currently down.
	pub fn is_button_down(&self, button : MouseButton) -> bool {
		(button as u8) == self.button_state & (button as u8)
	}

	/// Notifies when the mouse enters.
	pub fn on_enter(&mut self) {
		self.on_screen = true;
	}

	/// Store where the mouse moved to.
	pub fn on_mouse_update(&mut self, camera: &Camera, x : u32, y : u32, mut buttons : u8) {
		let new_position = camera.to_game_space(&Vec3::new(x as f32, y as f32, 0.0));
		if EPSILON < (&self.position - &new_position).length() {
			self.on_move(new_position.x, new_position.y);
		}
		buttons = buttons & 0x07; // Drop all but the first 3 buttons.
		if self.button_state != buttons {
			for button in [MouseButton::LEFT, MouseButton::RIGHT, MouseButton::MIDDLE].iter() {
				if 0 == self.button_state & (*button as u8) && 0 != buttons & (*button as u8) {
					self.on_up(*button);
				}
				if 0 != self.button_state & (*button as u8) && 0 == buttons & (*button as u8) {
					self.on_down(*button);
				}
			}
			self.button_state = buttons;
			self.changed = true;
		}
		self.on_screen = true;
	}

	/// Notifies when the mouse leaves.
	pub fn on_leave(&mut self) {
		self.on_screen = false;
	}

	fn on_move(&mut self, x : f32, y : f32) {
		//
		self.position.x = x;
		self.position.y = y;
		self.changed = true;
	}

	/// Notifies when a mouse button goes down.
	fn on_down(&mut self, _button : MouseButton) {
		// TODO: Junk this if not used...
	}

	/// Notifies when a mouse button is released.
	fn on_up(&mut self, _button : MouseButton) {
		// TODO: Junk this if not used...
	}
}
