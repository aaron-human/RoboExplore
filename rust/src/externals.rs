/// A module for all calls to JavaScript code.

use wasm_bindgen::prelude::*;

pub type ColorMagnitude = u8;
pub type DrawBufferID = i32;
pub type DrawCoord = f32;
pub type DrawIndex = u16;
pub type DrawTextID = i32;

#[wasm_bindgen]
extern {
	/// Logs a message to the console.
	#[wasm_bindgen(js_namespace=console, js_name=log)]
	fn _log(message : &str);

	#[wasm_bindgen(js_namespace=GAME, js_name=exportExample)]
	pub fn customCall(number : i32);

	#[wasm_bindgen(js_namespace=GAME, js_name=createDrawBuffer)]
	pub fn createDrawBuffer(type_ : i32) -> DrawBufferID;

	#[wasm_bindgen(js_namespace=GAME, js_name=deleteDrawBuffer)]
	pub fn deleteDrawBuffer(id : DrawBufferID);

	#[wasm_bindgen(js_namespace=GAME, js_name=setDisplayBuffer)]
	fn _setDisplayBuffer(id : DrawBufferID, vertices : Vec<DrawCoord>, colors : Vec<ColorMagnitude>, indices : Vec<DrawIndex>) -> bool;

	#[wasm_bindgen(js_namespace=GAME, js_name=setDisplayBufferTransform)]
	fn _setDisplayBufferTransform(id : DrawBufferID, matrix : Vec<DrawCoord>) -> bool;

	#[wasm_bindgen(js_namespace=GAME, js_name=setDisplayTransform)]
	pub fn setDisplayTransform(matrix : Vec<DrawCoord>);

	#[wasm_bindgen(js_namespace=GAME, js_name=setDisplayBufferVisibility)]
	pub fn setDisplayBufferVisibility(id : DrawBufferID, visibility : bool);

	#[wasm_bindgen(js_namespace=GAME, js_name="text.addTextPoint")]
	pub fn createTextPoint(x : i32, y : i32, horizontal : f32, vertical : f32, width : &str, height : &str, color : &str, alignment : &str, text : &str) -> DrawTextID;

	#[wasm_bindgen(js_namespace=GAME, js_name="text.positionTextPoint")]
	pub fn setTextPointPosition(id : DrawTextID, x : i32, y : i32, horizontal : f32, vertical : f32, width : &str, height : &str);

	#[wasm_bindgen(js_namespace=GAME, js_name="text.addTextArea")]
	pub fn createTextArea(top : f32, left : f32, bottom : f32, right : f32, color : &str, alignment : &str, text : &str) -> DrawTextID;

	#[wasm_bindgen(js_namespace=GAME, js_name="text.positionTextArea")]
	pub fn setTextAreaPosition(id : DrawTextID, top : f32, left : f32, bottom : f32, right : f32);

	#[wasm_bindgen(js_namespace=GAME, js_name="text.setText")]
	pub fn setTextValues(id : DrawTextID, color : &str, alignment : &str, text : &str);

	#[wasm_bindgen(js_namespace=GAME, js_name="text.setTextVisibility")]
	pub fn setDisplayTextVisibility(id : DrawTextID, visible : bool);
}

#[allow(non_snake_case)] // To keep with TypeScript's naming conventions, don't mess with this.
pub fn setDisplayBuffer(id : DrawBufferID, vertices : &Vec<DrawCoord>, colors : &Vec<ColorMagnitude>, indices : &Vec<DrawIndex>) {
	if !_setDisplayBuffer(id, vertices.clone(), colors.clone(), indices.clone()) {
		panic!("No such display buffer {}", id);
	}
}

#[allow(non_snake_case)] // To keep with TypeScript's naming conventions, don't mess with this.
pub fn setDisplayBufferTransform(id : DrawBufferID, matrix : Vec<DrawCoord>) {
	if !_setDisplayBufferTransform(id, matrix) {
		panic!("No such display buffer {}", id);
	}
}

pub fn log(message : &str) {
	if !cfg!(test) {
		_log(message);
	}
}
