use crate::{ sleep, humanlike::mouse_paths, keys::LBUTTON, mouse };
use mini_rand::Randomizable;
use std::time::Duration;



/// Move the mouse a specific amount.
pub fn displace<T, U, V>(displacement:T, interval:U, duration:V) where T:Randomizable<[i32; 2]>, U:Randomizable<Duration>, V:Randomizable<Duration> {
	let interval:Duration = interval.randomizable_value();
	for displacement in mouse_paths::create_displacement_path(displacement, interval, duration) {
		mouse::displace(displacement);
		sleep(interval);
	}
}

/// Move the mouse to a specific location.
pub fn move_to<T, U, V>(position:T, interval:U, duration:V) where T:Randomizable<[i32; 2]>, U:Randomizable<Duration>, V:Randomizable<Duration> {
	const CORRECTION_DURATION_MULTIPLIER:f32 = 0.1;
	const MAX_CORRECTION_ATTEMPTS:usize = 10;

	// Get raw argument values.
	let position:[i32; 2] = position.randomizable_value();
	let interval:Duration = interval.randomizable_value();
	let duration:Duration = duration.randomizable_value();
	let correction_duration:Duration = Duration::from_millis((duration.clone().as_millis() as f32 * CORRECTION_DURATION_MULTIPLIER) as u64);

	// Initial movement.
	let mut current_position:[i32; 2] = get_pos();
	displace([position[0] - current_position[0], position[1] - current_position[1]], interval, duration);

	// Correction movement.
	current_position = get_pos();
	let mut correction_attempts:usize = 0;
	while correction_attempts < MAX_CORRECTION_ATTEMPTS && current_position != position {
		displace([position[0] - current_position[0], position[1] - current_position[1]], interval, correction_duration);
		current_position = get_pos();
		correction_attempts += 1;
	}
}

/// Get the current position of the mouse, relative to the screen.
pub fn get_pos() -> [i32; 2] {	
	mouse::get_pos()
}

/// Click at a specific location without moving the cursor.
pub fn click<T, U, V, W>(position:T, move_interval:U, move_duration:V, press_duration:W, return_to_original_pos:bool) where T:Randomizable<[i32; 2]>, U:Randomizable<Duration>, V:Randomizable<Duration>, W:Randomizable<Duration> {
	let move_interval:Duration = move_interval.randomizable_value();
	let move_duration:Duration  = move_duration.randomizable_value();

	let original_position:[i32; 2] = get_pos();
	move_to(position, move_interval, move_duration);
	LBUTTON.send_await(press_duration);
	if return_to_original_pos {
		move_to(original_position, move_interval, move_duration);
	}
}

/// Drag the mouse from one point to another.
pub fn drag<T, U, V, W, X>(start_position:T, end_position:U, interval:V, move_duration:W, drag_duration:X, return_to_original_pos:bool) where T:Randomizable<[i32; 2]>, U:Randomizable<[i32; 2]>, V:Randomizable<Duration>, W:Randomizable<Duration>, X:Randomizable<Duration> {
	let interval:Duration = interval.randomizable_value();
	let move_duration:Duration  = move_duration.randomizable_value();
	let drag_duration:Duration = drag_duration.randomizable_value();

	let original_position:[i32; 2] = get_pos();
	move_to(start_position, interval, move_duration);
	LBUTTON.press();
	move_to(end_position, interval, drag_duration);
	LBUTTON.release();
	if return_to_original_pos {
		move_to(original_position, interval, move_duration);
	}
}