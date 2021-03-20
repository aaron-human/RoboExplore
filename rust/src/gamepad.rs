
use crate::geo::vec2::Vec2;

/// All the virtual keys to care about.
/// These are the keys that the game cares about.
/// These are distinguished from real keys in that multiple real keys can map to any of these.
#[derive(Copy, Clone)]
pub enum Button {
	A = 0, // This will act like an index into a vector.
	B,
	X,
	Y,
	R,
	L,
	/// The number of tracked buttons.
	COUNT,
}

/// The minimum value that an analog input needs to be to register.
/// For some reason on Firefox + Ubuntu 16.04, the sticks can get stuck at about 0.04 when flicked. So the threshold is fairly high.
const ANALOG_THRESHOLD : f32 = 0.05;

/// Stores info about the current keyboard state.
pub struct Gamepad {
	/// The raw button values.
	button_values : Vec<bool>,
	/// The mapping from Button enum values (as indices) to the button's specific index in button_values.
	button_mapping : Vec<usize>,

	/// The raw directional values.
	direction_values : Vec<f32>,
	/// The index of the x-axis of the main analog stick (in direction_values).
	main_x_index : usize,
	/// The index of the y-axis of the main analog stick (in direction_values).
	main_y_index : usize,
	/// The index of the right trigger (in direction_values).
	r_trigger_index : usize,
	/// The index of the left trigger (in direction_values).
	l_trigger_index : usize,
}

impl Gamepad {
	pub fn new() -> Gamepad {
		let mut button_mapping = vec![0; Button::COUNT as usize];
		button_mapping[Button::A as usize] = 0;
		button_mapping[Button::B as usize] = 1;
		button_mapping[Button::X as usize] = 2;
		button_mapping[Button::Y as usize] = 3;
		button_mapping[Button::L as usize] = 4;
		button_mapping[Button::R as usize] = 5;
		Gamepad {
			button_values : Vec::new(),
			button_mapping,
			direction_values : Vec::new(),
			main_x_index    : 0, // 3 for the right stick.
			main_y_index    : 1, // 4 for the right stick.
			r_trigger_index : 5,
			l_trigger_index : 2,
		}
	}

	// TODO: Add a way to change and save bindings.

	/// Updates the current internal state.
	pub fn update(&mut self, button_source : Vec<f32>, analog_source : Vec<f32>) {
		let button_length = button_source.len();
		self.button_values.resize(button_length, false);
		for index in 0..button_length {
			self.button_values[index] = 0.5f32 < button_source[index];
		}

		let analog_length = analog_source.len();
		self.direction_values.resize(analog_length, 0.0);
		for index in 0..analog_length {
			let mut value = analog_source[index];
			if value.abs() < ANALOG_THRESHOLD {
				value = 0.0;
			}
			self.direction_values[index] = value;
		}
	}

	/// Gets whether the given button is down.
	pub fn is_down(&self, button : Button) -> bool {
		let index = self.button_mapping[button as usize];
		if index < self.button_values.len() {
			self.button_values[index]
		} else {
			false
		}
	}

	/// Gets the current position of the main analog stick.
	pub fn direction(&self) -> Vec2 {
		Vec2::new(
			*self.direction_values.get(self.main_x_index).unwrap_or(&0.0),
			-(*self.direction_values.get(self.main_y_index).unwrap_or(&0.0)), // Not using cartesian.
		)
	}

	/// Gets the left trigger's analog value.
	pub fn l_trigger(&self) -> f32 {
		*self.direction_values.get(self.l_trigger_index).unwrap_or(&0.0)
	}

	/// Gets the right trigger's analog value.
	pub fn r_trigger(&self) -> f32 {
		*self.direction_values.get(self.r_trigger_index).unwrap_or(&0.0)
	}
}
