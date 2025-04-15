#[cfg(test)]
mod test {
	use crate::humanlike::mouse_paths::{ create_displacement_path, CURSOR_RECORDINGS_DIR_ENV_VAR };


	
	#[test]
	fn test_mouse_path_creation() {
		use std::env;

		// Set fake recordings dir.
		let original_arg_value = env::var(CURSOR_RECORDINGS_DIR_ENV_VAR);
		unsafe { env::set_var(CURSOR_RECORDINGS_DIR_ENV_VAR, "fake_dir/that_triggers/default_linear_curve"); }

		// Create path.
		let path:Vec<[i32; 2]> = create_displacement_path([100, 100], [0, 0], 100, 1000, 0);
		assert_eq!(&path, &[[15, 0], [15, 0], [15, 0], [15, 0], [15, 0], [15, 0], [10, 0]]);

		// Restore env var value.
		unsafe { env::set_var(CURSOR_RECORDINGS_DIR_ENV_VAR, match original_arg_value { Ok(val) => val, Err(_) => String::new() }); }
	}
}