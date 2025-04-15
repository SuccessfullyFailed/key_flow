#[cfg(test)]
mod test {
	use crate::humanlike::mouse::{ create_displacement_path, CURSOR_RECORDINGS_DIR_ENV_VAR };
	use std::time::Duration;



	#[test]
	fn test_mouse_path_creation() {
		use std::env;

		// Set fake recordings dir.
		let original_arg_value = env::var(CURSOR_RECORDINGS_DIR_ENV_VAR);
		unsafe { env::set_var(CURSOR_RECORDINGS_DIR_ENV_VAR, "fake_dir/that_triggers/default_linear_curve"); }

		// Create path.
		let path:Vec<[i32; 2]> = create_displacement_path([100, 100], [0, 0], Duration::from_millis(100), Duration::from_millis(1000), Duration::from_millis(0));
		assert_eq!(&path, &[[15, 0], [15, 0], [15, 0], [15, 0], [15, 0], [15, 0], [10, 0]]);

		// Restore env var value.
		unsafe { env::set_var(CURSOR_RECORDINGS_DIR_ENV_VAR, match original_arg_value { Ok(val) => val, Err(_) => String::new() }); }
	}
}