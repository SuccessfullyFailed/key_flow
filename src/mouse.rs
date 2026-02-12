use winapi::{ shared::windef::POINT, um::winuser::GetCursorPos };
use crate::{ InputBuilder, keys::LBUTTON };
use mini_rand::Randomizable;
use std::time::Duration;



/// Move the mouse a specific amount.
pub fn displace(offset:[i32; 2]) {
	InputBuilder::new().with_mouse_displacement(offset).execute();
}

/// Move the mouse to a specific location.
pub fn move_to(position:[i32; 2]) {	
	InputBuilder::new().with_mouse_move(position).execute();
}

/// Get the current position of the mouse, relative to the screen.
pub fn get_pos() -> [i32; 2] {	
	let mut cursor_pos:POINT = POINT { x: 0, y: 0 };
	unsafe { GetCursorPos(&mut cursor_pos); }
	[cursor_pos.x, cursor_pos.y]
}

/// Click at a specific location without moving the cursor.
pub fn click<T>(position:[i32; 2], press_duration:T) where T:Randomizable<Duration> {
	let original_pos:[i32; 2] = get_pos();
	let press_duration_millis:u64 = press_duration.randomizable_value().as_millis() as u64;
	InputBuilder::new().with_mouse_move(position).with_send(&LBUTTON, press_duration_millis).with_mouse_move(original_pos).execute();
}

/// Drag the mouse from one point to another.
pub fn drag<T>(start:[i32; 2], end:[i32; 2], press_duration:T) where T:Randomizable<Duration> {

	let mut input_builder:InputBuilder = InputBuilder::new();
	input_builder.add_mouse_move(start);
	input_builder.add_press(&LBUTTON);

	let press_duration_millis:u64 = press_duration.randomizable_value().as_millis() as u64;
	let offset_per_milli:[f32; 2] = [(end[0] - start[0]) as f32 / press_duration_millis.max(1) as f32, (end[1] - start[1]) as f32 / press_duration_millis.max(1) as f32];
	let mut cursor:[f32; 2] = start.map(|value| value as f32);
	for index in 0..press_duration_millis {
		if index > 0 {
			input_builder.add_delay(1);
		}
		input_builder.add_mouse_move(cursor.map(|value| value as i32));
		cursor[0] += offset_per_milli[0];
		cursor[1] += offset_per_milli[1];
	}

	input_builder.add_release(&LBUTTON);
	input_builder.add_mouse_move(end);
	input_builder.execute();
}