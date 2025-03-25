use winapi::{ shared::windef::POINT, um::winuser::{ mouse_event, GetCursorPos, SetCursorPos } };
use crate::{ key::KeyPressDuration, keys::LBUTTON };



/// Move the mouse a specific amount.
pub fn displace(offset:[i32; 2]) {
	unsafe { mouse_event(0x01, offset[0] as u32, offset[1] as u32, 0, 0); }
}

/// Move the mouse to a specific location.
pub fn move_to(position:[i32; 2]) {	
	unsafe { SetCursorPos(position[0], position[1]); };
}

/// Get the current position of the mouse, relative to the screen.
pub fn get_pos() -> [i32; 2] {	
	let mut cursor_pos:POINT = POINT { x: 0, y: 0 };
	unsafe { GetCursorPos(&mut cursor_pos); }
	[cursor_pos.x, cursor_pos.y]
}

/// Click at a specific location without moving the cursor.
pub fn click<T>(position:[i32; 2], duration:T) where T:KeyPressDuration + 'static + Send + Sync {
	let original_pos:[i32; 2] = get_pos();
	move_to(position);
	LBUTTON.send_await(duration);
	move_to(original_pos);
}