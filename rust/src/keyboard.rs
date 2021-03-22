use std::collections::{HashMap, HashSet};

/// All the virtual keys to care about.
/// These are the keys that the game cares about.
/// These are distinguished from real keys in that multiple real keys can map to any of these.
#[derive(Copy, Clone)]
pub enum Key {
	NULL = 0, // A junk key that tracked real keys are bound to when they're sent into "unbind()".
	UP,
	LEFT,
	DOWN,
	RIGHT,
	SPACE,
	DEBUG,
	COUNT, // Not a key. Just here to count how many exist.
}

/// Stores info about the current keyboard state.
pub struct Keyboard {
	key_mapping : HashMap<String, usize>, // Maps from keyboard event `key` strings to the index in `key_state` (if the key is tracked).
	key_state : Vec<bool>, // The state of all tracked (real) keys.
	bindings : Vec<HashSet<usize>>, // The (outer) Vec has one entry for each Key. The inner HashSet stores the key_state indices that that virtual key maps to.
	reverse_bindings : Vec<Key> // The reverse of `bindings`: Every real key index has an entry here to indicate which key it's already bound to. This is to make unbinding faster.
}

impl Keyboard {
	/// Creates an instance. Assumes all keys are not being pressed.
	pub fn new() -> Keyboard {
		let mut bindings = Vec::new();
		for _virtual_key in 0..(Key::COUNT as usize) {
			bindings.push(HashSet::new());
		}
		let mut instance = Keyboard {
			key_mapping: HashMap::new(),
			key_state: Vec::new(),
			bindings,
			reverse_bindings: Vec::new(),
		};
		// Setup some default key bindings.
		instance.bind(String::from("ArrowUp"),    Key::UP);
		instance.bind(String::from("ArrowLeft"),  Key::LEFT);
		instance.bind(String::from("ArrowDown"),  Key::DOWN);
		instance.bind(String::from("ArrowRight"), Key::RIGHT);
		// Some older browsers don't have "Arrow" in the key name.
		instance.bind(String::from("Up"),    Key::UP);
		instance.bind(String::from("Left"),  Key::LEFT);
		instance.bind(String::from("Down"),  Key::DOWN);
		instance.bind(String::from("Right"), Key::RIGHT);
		// Allow WASD too.
		instance.bind(String::from("w"), Key::UP);
		instance.bind(String::from("a"), Key::LEFT);
		instance.bind(String::from("s"), Key::DOWN);
		instance.bind(String::from("d"), Key::RIGHT);

		instance.bind(String::from(" "), Key::SPACE);

		instance.bind(String::from("~"), Key::DEBUG);
		instance
	}

	/// Binds a real key to a virtual one.
	pub fn bind(&mut self, real : String, virtual_ : Key) {
		// First setup a place for the real key to store its state.
		let real_index = match self.key_mapping.get(&real) {
			Option::Some(index) => {
				self.bindings[self.reverse_bindings[*index] as usize].remove(index);
				*index
			},
			Option::None => {
				let index = self.key_state.len();
				self.key_state.push(false);
				self.reverse_bindings.push(virtual_);
				self.key_mapping.insert(real, index);
				index
			},
		};
		// The bind it to the virtual key.
		self.bindings[virtual_ as usize].insert(real_index);
	}

	/// Fakes unbinding the given key by binding it to the Key::NULL value.
	pub fn unbind(&mut self, real : String) {
		self.bind(real, Key::NULL);
	}

	/// Checks if the given virtual key is pressed.
	pub fn is_down(&self, key : Key) -> bool {
		for real_index in &self.bindings[key as usize] {
			if self.key_state[*real_index] { return true; }
		}
		return false;
	}

	// Signals that the given (real) key has been pressed.
	pub fn on_down(&mut self, real : String) {
		if let Option::Some(real_index) = self.key_mapping.get(&real) {
			self.key_state[*real_index] = true;
		}
	}

	// Signals that the given (real) key has been released.
	pub fn on_up(&mut self, real : String) {
		if let Option::Some(real_index) = self.key_mapping.get(&real) {
			self.key_state[*real_index] = false;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn default_bindings() {
		let mut keyboard = Keyboard::new();
		assert_eq!(keyboard.is_down(Key::UP),    false);
		assert_eq!(keyboard.is_down(Key::LEFT),  false);
		assert_eq!(keyboard.is_down(Key::DOWN),  false);
		assert_eq!(keyboard.is_down(Key::RIGHT), false);

		assert_eq!(keyboard.is_down(Key::SPACE), false);
		keyboard.on_down(" ".to_string());
		assert_eq!(keyboard.is_down(Key::SPACE), true);
		keyboard.on_up(" ".to_string());
		assert_eq!(keyboard.is_down(Key::SPACE), false);
	}

	#[test]
	fn rebinding() {
		let mut keyboard = Keyboard::new();
		keyboard.unbind("Up".to_string());
		keyboard.bind("q".to_string(), Key::UP);
		assert_eq!(keyboard.is_down(Key::UP), false);

		keyboard.on_down("Up".to_string());
		assert_eq!(keyboard.is_down(Key::UP), false);
		keyboard.on_down("q".to_string());
		assert_eq!(keyboard.is_down(Key::UP), true);
		keyboard.on_up("Up".to_string());
		assert_eq!(keyboard.is_down(Key::UP), true);
		keyboard.on_up("q".to_string());
		assert_eq!(keyboard.is_down(Key::UP), false);

		keyboard.bind("q".to_string(), Key::DOWN);

		assert_eq!(keyboard.is_down(Key::UP),   false);
		assert_eq!(keyboard.is_down(Key::DOWN), false);
		keyboard.on_down("q".to_string());
		assert_eq!(keyboard.is_down(Key::UP),   false);
		assert_eq!(keyboard.is_down(Key::DOWN), true);
		keyboard.on_up("q".to_string());
		assert_eq!(keyboard.is_down(Key::UP),   false);
		assert_eq!(keyboard.is_down(Key::DOWN), false);
	}
}
