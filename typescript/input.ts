namespace ExampleProject {
	/// The type of callback to use for keyboard input.
	export type KeyboardInputCallback = (key : string) => void;
	/// The type of callback to use for mouse movements.
	export type MouseUpdateCallback = (x : number, y : number, buttons : number) => void;
	/// The type of callback for mouse enter/leave events.
	export type MouseFocusCallback = () => void;
	/// The type of callback for mouse buttons are pressed/released.
	export type MouseButtonCallback = (button : number) => void;

	/**
	 * A class for handling keyboard and mouse intput
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
	}
}
