namespace ExampleProject {
	/// Displays overlay text in a given element.
	export class TextDisplay {
		/// All the text elements ordered according to ID.
		/// Elements are never removed, just replaced with null.
		private _elements : HTMLDivElement[] = [];

		/// Creates an instance using the given element to store text in.
		constructor(private readonly _container : HTMLElement) {
			//
		}

		/// Creates and stores a text box element and returns its id.
		/// The element isn't in the document, it will have to be added later.
		private _makeTextBox() : number {
			const element = document.createElement("div");
			element.classList.add("dynamic_text");
			const id = this._elements.length;
			this._elements.push(element);
			return id;
		}

		/// Gets the element given its ID.
		private _getTextBox(id : number) : HTMLDivElement {
			const element = this._elements[id];
			if (!element) {
				throw `Invalid element ID ${id}`;
			}
			return element;
		}

		/**
		 * Adds a text box using a specific point. Returns the handle to the element.
		 * @param x The x position in pixels. Zero is the center of the screen. The left half is negative.
		 * @param y The y position in pixels. Zero is the center of the screen. The bottom half is negative.
		 * @param hoirzontal The horizontal position of the (x, y) coordinate relative to the text. 0.0 puts the point on the left. 1.0 puts it on the right.
		 * @param vertical The vertical position of the (x, y) coordinate relative to the text. 0.0 puts the point at the top. 1.0 puts it at the bottom.
		 * @param width The width of the text box in CSS units (I'd recommend "em").
		 * @param height The height of the text box in CSS units (I'd recommend "ex").
		 * @param color A CSS color value.
		 * @param alignment The CSS "text-align" value to use.
		 * @param text The (HTML) text to fill it with.
		 * @return A unique ID representing this text box.
		 */
		public addTextPoint(x : number, y : number, horizontal : number, vertical : number, width : string, height : string, color : string, alignment : string, text : string) : number {
			const id = this._makeTextBox();

			this.positionTextPoint(id, x, y, horizontal, vertical, width, height);
			this.setText(id, color, alignment, text);

			this._container.appendChild(this._getTextBox(id));
			return id;
		}

		/**
		 * Sets the position of a text box using a specific point.
		 * @param id The text box's ID.
		 * @param x The x position in pixels. Zero is the center of the screen. The left half is negative.
		 * @param y The y position in pixels. Zero is the center of the screen. The bottom half is negative.
		 * @param hoirzontal The horizontal position of the (x, y) coordinate relative to the text. 0.0 puts the point on the left. 1.0 puts it on the right.
		 * @param vertical The vertical position of the (x, y) coordinate relative to the text. 0.0 puts the point at the top. 1.0 puts it at the bottom.
		 * @param width The width of the text box in CSS units (I'd recommend "em").
		 * @param height The height of the text box in CSS units (I'd recommend "ex").
		 */
		public positionTextPoint(id : number, x : number, y : number, horizontal : number, vertical : number, width : string, height : string) {
			const element = this._getTextBox(id);
			element.style.top  = `calc(50% - ${y}px - ${vertical} * ${height} )`;
			element.style.left = `calc(50% + ${x}px - ${horizontal} * ${width} )`;
			element.style.width  = width;
			element.style.height = height;
		}

		/**
		 * Adds a text box using view size percentages. Returns the handle to the element.
		 * @param top The position of the top edge of the text box. Always a percentage (0.0 = top of screen, 1.0 = bottom of screen).
		 * @param left The position of the left edge of the text box. Always a percentage (0.0 = left side of screen, 1.0 = right side of screen).
		 * @param bottom The position of the bottom edge of the text box. Always a percentage (0.0 = top of screen, 1.0 = bottom of screen).
		 * @param right The position of the right edge of the text box. Always a percentage (0.0 = left side of screen, 1.0 = right side of screen).
		 * @param color A CSS color value.
		 * @param alignment The CSS "text-align" value to use.
		 * @param text The (HTML) text to fill it with.
		 * @return A unique ID representing this text box.
		 */
		public addTextArea(top : number, left : number, bottom : number, right : number, color : string, alignment : string, text : string) : number {
			const id = this._makeTextBox();

			this.positionTextArea(id, top, left, bottom, right);
			this.setText(id, color, alignment, text);

			this._container.appendChild(this._getTextBox(id));
			return id;
		}

		/**
		 * Positions the given text box using view size percentages.
		 * @param id The text box's ID.
		 * @param top The position of the top edge of the text box. Always a percentage (0.0 = top of screen, 1.0 = bottom of screen).
		 * @param left The position of the left edge of the text box. Always a percentage (0.0 = left side of screen, 1.0 = right side of screen).
		 * @param bottom The position of the bottom edge of the text box. Always a percentage (0.0 = top of screen, 1.0 = bottom of screen).
		 * @param right The position of the right edge of the text box. Always a percentage (0.0 = left side of screen, 1.0 = right side of screen).
		 */
		public positionTextArea(id : number, top : number, left : number, bottom : number, right : number) {
			const element = this._getTextBox(id);
			element.style.top  = `${100.0 * top}%`;
			element.style.left = `${100.0 * left}%`;
			element.style.width  = `${100.0 * (right - left)}%`;
			element.style.height = `${100.0 * (bottom - top)}%`;
		}

		/**
		 * Sets the contents and alignment of a text box.
		 * @param id The text box's ID.
		 * @param color A CSS color value.
		 * @param alignment The CSS "text-align" value to use.
		 * @param text The (HTML) text to fill it with.
		 */
		public setText(id : number, color : string, alignment : string, text : string) {
			const element = this._getTextBox(id);
			element.style.textAlign = alignment;
			element.style.color = color;
			element.innerHTML = text;
		}

		/// Sets whether a bit of text is visible.
		public setTextVisibility(id : number, visible : boolean) {
			const element = this._getTextBox(id);
			element.style.display = (visible) ? "block" : "none";
		}
	}
}
