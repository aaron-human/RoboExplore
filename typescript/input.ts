namespace ExampleProject {
	/// The type of callback to use for keyboard input.
	export type KeyboardInputCallback = (key : string) => void;
	/// The type of callback to use for mouse movements.
	export type MouseUpdateCallback = (x : number, y : number, buttons : number) => void;
	/// The type of callback for mouse enter/leave events.
	export type MouseFocusCallback = () => void;
	/// The type of callback for mouse buttons are pressed/released.
	export type MouseButtonCallback = (button : number) => void;

	/// A way to describe a gamepad's state.
	export class GamepadState {
		/// If this actually exists. Gets set to true once know that an actual Gamepad object was used to construct this.
		public valid : boolean = false;
		/// The button states.
		public readonly buttons : number[] = [];
		/// The state of any analog sticks.
		/// This matches the values passed in by the DOM.
		public readonly analogSticks : number[] = [];

		/// Creates an instance from the given DOM object.
		public constructor(source : Gamepad) {
			if (!source) { return; } // The source may be 'null' if it got disconnected.
			this.valid = true;
			for (let button of source.buttons) {
				let state = 0.0; // Assume not pressed.
				if (undefined !== button["pressed"]) {
					state = (button.pressed) ? (1.0) : (0.0);
				}
				if (undefined !== button["value"]) {
					state = button.value;
				}
				if ("number" === typeof(button)) {
					// Some older browsers just store the button's state directly as a number.
					state = button;
				}
				this.buttons.push(state);
			}
			// The analog sticks data is packed as a weird pile of floating point values.
			// ... Why? Well, whatever, that's easy to pass to WASM/Rust.
			for (let index = 0;index < source.axes.length;index += 1) {
				this.analogSticks.push(source.axes[index]);
			}
		}

		/// Check if this has the same value as some other GamepadState.
		public equals(other : GamepadState) : boolean {
			if (this.buttons.length !== other.buttons.length) { return false; }
			for (let index = 0;index < this.buttons.length;index += 1) {
				if (this.buttons[index] !== other.buttons[index]) { return false; }
			}

			if (this.analogSticks.length !== other.analogSticks.length) { return false; }
			for (let index = 0;index < this.analogSticks.length;index += 1) {
				if (this.analogSticks[index] !== other.analogSticks[index]) { return false; }
			}

			return true;
		}
	}

	/**
	 * A class for handling keyboard, mouse, and gamepad input.
	 */
	export class Input {
		/// The key-down callback.
		private _keyDownCallback : KeyboardInputCallback = null;
		/// The key-up callback.
		private _keyUpCallback : KeyboardInputCallback = null;
		/// In order to ignore repeated key presses. Store the last key event state sent.
		private _keyState : Map<string, boolean> = new Map();
		/// The "mouse enters the canvas" callback.
		private _mouseEnterCallback : MouseFocusCallback = null;
		/// The "mouse moves over the canvas" and "button clicked/released" callback.
		private _mouseUpdateCallback : MouseUpdateCallback = null;
		/// The "mouse leaves the canvas" callback.
		private _mouseLeaveCallback : MouseFocusCallback = null;

		/// The index of gamepad that just got added.
		/// An invalid index means none was added.
		private _gamepadAddedIndex : number = -1;
		/// The most recent gamepad states. The indices are the Gamepad.index values that the DOM provides.
		private _gamepads : GamepadState[] = [];

		/// Creates an instance.
		constructor(private readonly _canvas : HTMLCanvasElement) {
			document.addEventListener("keydown", this._onKeyDown.bind(this));
			document.addEventListener("keyup", this._onKeyUp.bind(this));
			_canvas.addEventListener("mouseenter", this._onMouseEnter.bind(this));
			_canvas.addEventListener("mousemove", this._onMouseUpdate.bind(this));
			_canvas.addEventListener("mouseleave", this._onMouseLeave.bind(this));
			_canvas.addEventListener("mousedown", this._onMouseUpdate.bind(this));
			_canvas.addEventListener("mouseup", this._onMouseUpdate.bind(this));
			// Prevent right clicking menu from showing
			_canvas.addEventListener("contextmenu", (event : MouseEvent) => { event.preventDefault(); });

			// Detect when gamepads are connected/started up.
			window.addEventListener("gamepadconnected", this._onGamepadConnected.bind(this));
			window.addEventListener("gamepaddisconnected", this._onGamepadDisconnected.bind(this));
		}

		/// Sets up everything. Including linking the given callbacks.
		public setup(keyDownCallback : KeyboardInputCallback, keyUpCallback : KeyboardInputCallback, mouseEnterCallback : MouseFocusCallback, mouseUpdateCallback : MouseUpdateCallback, mouseLeaveCallback : MouseFocusCallback) {
			this._keyDownCallback = keyDownCallback;
			this._keyUpCallback = keyUpCallback;
			this._mouseEnterCallback = mouseEnterCallback;
			this._mouseUpdateCallback = mouseUpdateCallback;
			this._mouseLeaveCallback = mouseLeaveCallback;
		}

		/// Handles the key being pressed.
		private _onKeyDown(event : KeyboardEvent) {
			const key = event.key;
			if (this._keyState.get(key)) {
				return;
			}
			this._keyState.set(key, true);
			if (this._keyDownCallback) {
				this._keyDownCallback(key);
			}
		}

		/// Handles the key being released.
		private _onKeyUp(event : KeyboardEvent) {
			const key = event.key;
			this._keyState.set(key, false);
			if (this._keyUpCallback) {
				this._keyUpCallback(key);
			}
		}

		/// Handles the mouse entering the canvas area.
		private _onMouseEnter(event : MouseEvent) {
			if (this._mouseEnterCallback) {
				this._mouseEnterCallback();
			}
		}

		/// Handles the mouse moving over the canvas.
		private _onMouseUpdate(event : MouseEvent) {
			let bounds = this._canvas.getBoundingClientRect();
			if (this._mouseUpdateCallback) {
				this._mouseUpdateCallback(event.clientX - bounds.x, event.clientY - bounds.y, event.buttons);
			}
		}

		/// Handles the mouse leaving the canvas area.
		private _onMouseLeave(event : MouseEvent) {
			// Then handle the actual leave.
			if (this._mouseLeaveCallback) {
				this._mouseLeaveCallback();
			}
		}

		/// Handles a gamepad connecting.
		private _onGamepadConnected(event : GamepadEvent) {
			this._gamepadAddedIndex = event.gamepad.index;
		}

		/// Handles a gamepad disconnecting.
		private _onGamepadDisconnected(event : GamepadEvent) {
			// Do I need to do anything here?
		}

		/// Gets the most recent GamepadState, if it has changed.
		public getChangedGamepadState() : GamepadState {
			let gamepads : Gamepad[] = [];
			if (undefined !== navigator["getGamepads"]) {
				gamepads = navigator.getGamepads();
			}
			if (undefined !== navigator["webkitGetGamepads"]) {
				// Other browsers use this instead...
				gamepads = navigator["webkitGetGamepads"]();
			}
			// Then compare all the gamepads to their last known states and take the most recent one.
			for (let index = 0;index < gamepads.length;index += 1) {
				// Store the data in a way that's easy to check.
				const current = new GamepadState(gamepads[index]);
				// Compare it to the known state (if there was one).
				// If find **any** change, then this is the right gamepad.
				if (!this._gamepads[index] || !this._gamepads[index].equals(current)) {
					this._gamepads[index] = current;
					console.log(`Gamepad changed to #${index}: ${gamepads[index].id}`);
					return current;
				}
			}
			// If none changed but one was just added, then try that one.
			const lastResort = this._gamepads[this._gamepadAddedIndex];
			if (lastResort) {
				this._gamepadAddedIndex = -1; // Just used up this update.
				return lastResort;
			}
			return null;
		}
	}
}
