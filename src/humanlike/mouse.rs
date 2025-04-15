use crate::{ key::KeyPressDuration, keys::LBUTTON };
use std::{thread::sleep, time::Duration};
use super::mouse_paths;
use crate::mouse;



/// Move the mouse a specific amount.
pub fn displace<T>(displacement:[i32; 2], displacement_randomness:[i32; 2], interval:T, duration:T, duration_randomness:T) where T:KeyPressDuration + 'static + Send + Sync {
	let sleep_duration:Duration = interval.as_duration();
	for displacement in mouse_paths::create_displacement_path(displacement, displacement_randomness, interval.as_millis(), duration.as_millis(), duration_randomness.as_millis()) {
		mouse::displace(displacement);
		sleep(sleep_duration);
	}
}

/// Move the mouse to a specific location.
pub fn move_to<T>(position:[i32; 2], position_randomness:[i32; 2], interval:T, duration:T, duration_randomness:T) where T:KeyPressDuration + 'static + Send + Sync + Clone {
	const CORRECTION_DURATION_MULTIPLIER:f32 = 0.1;
	const MAX_CORRECTION_ATTEMPTS:usize = 10;

	let interval:u64 = interval.as_millis();
	let duration:u64  = duration.as_millis();
	let duration_randomness:u64 = duration_randomness.as_millis();
	let correction_duration:u64 = (duration.clone().as_millis() as f32 * CORRECTION_DURATION_MULTIPLIER) as u64;

	// Initial movement.
	let mut current_position:[i32; 2] = get_pos();
	displace([position[0] - current_position[0], position[1] - current_position[1]], position_randomness, interval, duration, duration_randomness);

	// Correction movement.
	current_position = get_pos();
	let mut correction_attempts:usize = 0;
	while correction_attempts < MAX_CORRECTION_ATTEMPTS && current_position != position {
		displace([position[0] - current_position[0], position[1] - current_position[1]], [0, 0], interval, correction_duration, duration_randomness);
		current_position = get_pos();
		correction_attempts += 1;
	}
}

/// Get the current position of the mouse, relative to the screen.
pub fn get_pos() -> [i32; 2] {	
	mouse::get_pos()
}

/// Click at a specific location without moving the cursor.
pub fn click<T>(position:[i32; 2], position_randomness:[i32; 2], move_interval:T, move_duration:T, move_duration_randomness:T, press_duration:T, return_to_original_pos:bool) where T:KeyPressDuration + 'static + Send + Sync {
	let move_interval:u64 = move_interval.as_millis();
	let move_duration:u64  = move_duration.as_millis();
	let move_duration_randomness:u64 = move_duration_randomness.as_millis();

	let original_position:[i32; 2] = get_pos();
	move_to(position, position_randomness, move_interval, move_duration, move_duration_randomness);
	LBUTTON.send_await(press_duration);
	if return_to_original_pos {
		move_to(original_position, position_randomness, move_interval, move_duration, move_duration_randomness);
	}
}

/// Drag the mouse from one point to another.
pub fn drag<T>(start_position:[i32; 2], end_position:[i32; 2], position_randomness:[i32; 2], interval:T, move_duration:T, drag_duration:T, duration_randomness:T, return_to_original_pos:bool) where T:KeyPressDuration + 'static + Send + Sync {
	let interval:u64 = interval.as_millis();
	let move_duration:u64  = move_duration.as_millis();
	let drag_duration:u64 = drag_duration.as_millis();
	let duration_randomness:u64 = duration_randomness.as_millis();

	let original_position:[i32; 2] = get_pos();
	move_to(start_position, position_randomness, interval, move_duration, duration_randomness);
	LBUTTON.press();
	move_to(end_position, position_randomness, interval, drag_duration, duration_randomness);
	LBUTTON.release();
	if return_to_original_pos {
		move_to(original_position, position_randomness, interval, move_duration, duration_randomness);
	}
}