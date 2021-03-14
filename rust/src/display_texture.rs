use crate::externals::*;

pub struct DisplayTexture {
	id : DrawTextureID, // The reference to the external JS buffer.
}


impl DisplayTexture {
	pub fn new() -> DisplayTexture {
		DisplayTexture {
			id : createDrawTexture(),
		}
	}

	/// Loads in the texture information from the given URL.
	pub fn load_from_url(&mut self, url : &str) {
		assert!(setDrawTextureFromURL(self.id, url), "Couldn't start loading url {:?} into draw texture {}", url, self.id)
	}

	/// Gets the raw low-level ID. Don't use this unless you're calling from `DisplayBuffer.set_texture()`.
	pub fn get_id(&self) -> DrawTextureID {
		self.id
	}
}


impl Drop for DisplayTexture {
	/// Remove the texture.
	fn drop(&mut self) {
		assert!(deleteDrawTexture(self.id), "Couldn't delete draw texture {}", self.id);
	}
}
