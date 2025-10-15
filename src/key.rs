use winapi::um::winuser::{ MapVirtualKeyW, SendInput, INPUT, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_XDOWN, MOUSEINPUT };
use crate::{ sleep, key_hook::{ self, handle_virtual_key_alteration }, KeyPattern, RandomizableDuration };
use std::{ mem, ptr, thread };



#[derive(Clone, Copy, Debug)]
pub struct Key(u8);
impl Key {

	/* CONSTRUCTOR METHODS */

	/// Create a new key.
	pub const fn new(code:u8) -> Key {
		Key(code)
	}



	/* USAGE METHODS */

	/// Get the Key-code of the key.
	pub fn key_code(&self) -> u8 {
		self.0
	}

	/// Return the key as a pattern.
	pub fn as_pattern(&self) -> KeyPattern {
		if self.0 == 0 {
			KeyPattern::new(0, 0)
		} else if self.0 < 129 {
			KeyPattern::new(0, 1 << (self.0 - 1))
		} else {
			KeyPattern::new(1 << (self.0 - 129), 0)
		}
	}

	/// Check if the key is down.
	pub fn down(&self) -> bool {
		key_hook::get_key_state(self.0)
	}

	/// Press the key.
	pub fn press(&self) {
		if self.0 < 6 {
			self.create_mouse_event(match self.0 { 1 => MOUSEEVENTF_LEFTDOWN, 2 => MOUSEEVENTF_RIGHTDOWN, 4 => MOUSEEVENTF_MIDDLEDOWN, 5 => MOUSEEVENTF_XDOWN, 6 => MOUSEEVENTF_XDOWN, _ => 0 });
		} else {
			self.create_keyboard_event(0);
		}
		handle_virtual_key_alteration(self.0, true);
	}

	/// Release the key.
	pub fn release(&self) {
		if self.0 < 6 {
			self.create_mouse_event(match self.0 { 1 => MOUSEEVENTF_LEFTDOWN, 2 => MOUSEEVENTF_RIGHTDOWN, 4 => MOUSEEVENTF_MIDDLEDOWN, 5 => MOUSEEVENTF_XDOWN, 6 => MOUSEEVENTF_XDOWN, _ => 0 });
		} else {
			self.create_keyboard_event(KEYEVENTF_KEYUP);
		}
		handle_virtual_key_alteration(self.0, false);
	}

	/// Send the key.
	pub fn send<T>(&self, duration:T) where T:RandomizableDuration + Send + 'static {
		if duration.is_empty() {
			self.press();
			self.release();
		} else {
			let key:Key = self.clone();
			thread::spawn(move || {
				key.press();
				sleep(duration.as_duration());
				key.release();
			});
		}
	}

	/// Send the key and wait until the key is released.
	pub fn send_await<T>(&self, duration:T) where T:RandomizableDuration + 'static {
		if duration.is_empty() {
			self.press();
			self.release();
		} else {
			self.press();
			sleep(duration.as_duration());
			self.release();
		}
	}

	/// Toggle the key.
	pub fn toggle(&self) {
		if self.down() {
			self.release();
		} else {
			self.down();
		}
	}
	


	/* WINDOWS EVENT METHODS */

	/// Create a windows keyboard event.
	#[allow(invalid_value)]
	fn create_keyboard_event(&self, flags:u32) {
		unsafe {
			let input:KEYBDINPUT = KEYBDINPUT { wVk: self.0 as u16, wScan: MapVirtualKeyW(self.0 as u32, 0) as u16, dwFlags: flags, time: 0, dwExtraInfo: 0 };
			let mut input_record:INPUT = INPUT { type_: 1, u: mem::MaybeUninit::uninit().assume_init() };
			ptr::write(&mut input_record.u as *mut _ as *mut KEYBDINPUT, input);
			SendInput(1, &mut input_record, mem::size_of::<INPUT>() as i32);
		}
	}

	/// Create a windows mouse event.
	#[allow(invalid_value)]
	fn create_mouse_event(&self, flags:u32) {
		unsafe {
			let input:MOUSEINPUT = MOUSEINPUT { dx: 0, dy: 0, mouseData: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 };
			let mut input_record = INPUT { type_: INPUT_MOUSE, u: mem::MaybeUninit::uninit().assume_init() };
			ptr::write(&mut input_record.u as *mut _ as *mut MOUSEINPUT, input);
			SendInput(1, &mut input_record, mem::size_of::<INPUT>() as i32);
		}
	}
}
impl PartialEq for Key {
	fn eq(&self, other:&Self) -> bool {
		self.0 == other.0
	}
}