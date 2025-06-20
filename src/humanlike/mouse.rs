use crate::{ humanlike::mouse_paths, keys::LBUTTON, mouse, RandomizableCoordinate, RandomizableDuration };
use std::{thread::sleep, time::Duration};



/// Move the mouse a specific amount.
pub fn displace<T, U, V>(displacement:T, interval:U, duration:V) where T:RandomizableCoordinate, U:RandomizableDuration, V:RandomizableDuration {
	let sleep_duration:Duration = interval.as_duration();
	for displacement in mouse_paths::create_displacement_path(displacement, interval, duration) {
		mouse::displace(displacement);
		sleep(sleep_duration);
	}
}

/// Move the mouse to a specific location.
pub fn move_to<T, U, V>(position:T, interval:U, duration:V) where T:RandomizableCoordinate, U:RandomizableDuration, V:RandomizableDuration {
	const CORRECTION_DURATION_MULTIPLIER:f32 = 0.1;
	const MAX_CORRECTION_ATTEMPTS:usize = 10;

	// Get raw argument values.
	let position:[i32; 2] = position.get_value();
	let interval:u64 = interval.as_millis();
	let duration:u64 = duration.as_millis();
	let correction_duration:u64 = (duration.clone().as_millis() as f32 * CORRECTION_DURATION_MULTIPLIER) as u64;

	// Initial movement.
	let mut current_position:[i32; 2] = get_pos();
	displace((position[0] - current_position[0], position[1] - current_position[1]), interval, duration);

	// Correction movement.
	current_position = get_pos();
	let mut correction_attempts:usize = 0;
	while correction_attempts < MAX_CORRECTION_ATTEMPTS && current_position != position {
		displace((position[0] - current_position[0], position[1] - current_position[1]), interval, correction_duration);
		current_position = get_pos();
		correction_attempts += 1;
	}
}

/// Get the current position of the mouse, relative to the screen.
pub fn get_pos() -> [i32; 2] {	
	mouse::get_pos()
}

/// Click at a specific location without moving the cursor.
pub fn click<T, U, V, W>(position:T, move_interval:U, move_duration:V, press_duration:W, return_to_original_pos:bool) where T:RandomizableCoordinate, U:RandomizableDuration, V:RandomizableDuration, W:RandomizableDuration + 'static {
	let move_interval:u64 = move_interval.as_millis();
	let move_duration:u64  = move_duration.as_millis();

	let original_position:[i32; 2] = get_pos();
	let original_position:(i32, i32) = (original_position[0], original_position[1]);
	move_to(position, move_interval, move_duration);
	LBUTTON.send_await(press_duration);
	if return_to_original_pos {
		move_to(original_position, move_interval, move_duration);
	}
}

/// Drag the mouse from one point to another.
pub fn drag<T, U, V, W, X>(start_position:T, end_position:U, interval:V, move_duration:W, drag_duration:X, return_to_original_pos:bool) where T:RandomizableCoordinate, U:RandomizableCoordinate, V:RandomizableDuration, W:RandomizableDuration, X:RandomizableDuration {
	let interval:u64 = interval.as_millis();
	let move_duration:u64  = move_duration.as_millis();
	let drag_duration:u64 = drag_duration.as_millis();

	let original_position:[i32; 2] = get_pos();
	move_to(start_position, interval, move_duration);
	LBUTTON.press();
	move_to(end_position, interval, drag_duration);
	LBUTTON.release();
	if return_to_original_pos {
		move_to(original_position, interval, move_duration);
	}
}