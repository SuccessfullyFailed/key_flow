#[cfg(test)]
mod tests {
	use super::super::mouse_paths::MouseProgressionPath;
	use mini_rand::RandomNumber;
	use file_ref::FileRef;
	use std::ops::Range;

	

	/// Crate a random mouse progression path.
	fn random_mouse_progression_path() -> MouseProgressionPath {
		const RAND_POSITION_RANGE:Range<usize> = 0..500;

		MouseProgressionPath::new((0..500).map(|_| [usize::random_range(RAND_POSITION_RANGE), usize::random_range(RAND_POSITION_RANGE)]).collect())
	}

	#[test]
	fn test_mouse_path_byte_conversion() {
		let original_path:MouseProgressionPath = random_mouse_progression_path();
		let bytes:Vec<u8> = original_path.to_bytes();
		let path_from_bytes:MouseProgressionPath = MouseProgressionPath::from_bytes(&bytes).unwrap();
		
		assert_eq!(original_path, path_from_bytes);
	}

	#[test]
	fn test_mouse_path_file_conversion() {
		const FILE_PATH:&str = "target/MouseProgressionPath_unittest.dat";

		let original_path:MouseProgressionPath = random_mouse_progression_path();
		original_path.save_to_file(FILE_PATH).unwrap();
		let path_from_file:MouseProgressionPath = MouseProgressionPath::from_file(FILE_PATH).unwrap();
		FileRef::new(FILE_PATH).delete().unwrap();
		
		assert_eq!(original_path, path_from_file);
	}
}