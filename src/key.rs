use winapi::um::winuser::{ INPUT, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT, MapVirtualKeyW, SendInput };
use crate::{ KeyPattern, key_hook::{ self, handle_virtual_key_alteration }, keys, sleep };
use std::{ mem, ptr, thread, time::Duration };
use mini_rand::Randomizable;



#[derive(Clone, Copy, Debug)]
pub struct Key(u8);
impl Key {

	/* CONSTRUCTOR METHODS */

	/// Create a new key.
	pub const fn new(code:u8) -> Key {
		Key(code)
	}

	/// Try to get a list of keys from a str.
	pub fn from_str(source:&str) -> Vec<Vec<Key>> {
		source.chars().map(|character| Key::from_char(character)).collect()
	}

	/// Try to get a list of keys from a character.
	pub fn from_char(character:char) -> Vec<Key> {

		let character_index:u8 = character as u8;

		// Lowercase chars.
		const LOWERCASE_START:u8 = 'a' as u8;
		const LOWERCASE_END:u8 = 'z' as u8;
		if character_index >= LOWERCASE_START && character_index <= LOWERCASE_END {
			return vec![Key(keys::A.0 + (character_index - LOWERCASE_START))];
		}

		// Uppercase chars.
		const UPPERCASE_START:u8 = 'A' as u8;
		const UPPERCASE_END:u8 = 'Z' as u8;
		if character_index >= UPPERCASE_START && character_index <= UPPERCASE_END {
			return vec![keys::SHIFT, Key(keys::A.0 + (character_index - UPPERCASE_START))];
		}

		// Digits.
		const DIGIT_START:u8 = '0' as u8;
		const DIGIT_END:u8 = '9' as u8;
		if character_index >= DIGIT_START && character_index <= DIGIT_END {
			return vec![Key(keys::KEY_0.0 + (character_index - DIGIT_START))];
		}

		// Shifted digits.
		const SHIFTED_DIGITS:&[char] = &['!', '@', '#', '$', '%', '^', '&', '*', '(', ')'];
		if let Some(digit_index) = SHIFTED_DIGITS.iter().position(|c| *c == character) {
			return vec![keys::SHIFT, Key(keys::KEY_0.0 + digit_index as u8)];
		}

		// Punctuation.
		match character {
			' '  => vec![keys::SPACE],
			','  => vec![keys::COMMA],
			'<'  => vec![keys::SHIFT, keys::COMMA],
			';'  => vec![keys::COLON],
			':'  => vec![keys::SHIFT, keys::COLON],
			'\'' => vec![keys::QUOTE],
			'"'  => vec![keys::SHIFT, keys::QUOTE],
			'`'  => vec![keys::TILDA],
			'~'  => vec![keys::SHIFT, keys::TILDA],
			'['  => vec![keys::BLOCK_OPEN],
			'{'  => vec![keys::SHIFT, keys::BLOCK_OPEN],
			']'  => vec![keys::BLOCK_CLOSE],
			'}'  => vec![keys::SHIFT, keys::BLOCK_CLOSE],

			'-' => vec![keys::SUBTRACT],
			'_' => vec![keys::SHIFT, keys::SUBTRACT],
			'=' => vec![keys::ADD],
			'+' => vec![keys::SHIFT, keys::ADD],

			'\t' => vec![keys::TAB],
			'\n' => vec![keys::ENTER],
			'\r' => vec![keys::ENTER],

			_ => Vec::new()
		}
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
			self.create_mouse_event(match self.0 { 1 => MOUSEEVENTF_LEFTUP, 2 => MOUSEEVENTF_RIGHTUP, 4 => MOUSEEVENTF_MIDDLEUP, 5 => MOUSEEVENTF_XUP, 6 => MOUSEEVENTF_XUP, _ => 0 });
		} else {
			self.create_keyboard_event(KEYEVENTF_KEYUP);
		}
		handle_virtual_key_alteration(self.0, false);
	}

	/// Send the key.
	pub fn send<T>(&self, duration:T) where T:Randomizable<Duration> {
		let duration:Duration = duration.randomizable_value();
		if duration.is_zero() {
			self.press();
			self.release();
		} else {
			let key:Key = self.clone();
			thread::spawn(move || {
				key.press();
				sleep(duration);
				key.release();
			});
		}
	}

	/// Send the key and wait until the key is released.
	pub fn send_await<T>(&self, duration:T) where T:Randomizable<Duration> {
		let duration:Duration = duration.randomizable_value();
		if duration.is_zero() {
			self.press();
			self.release();
		} else {
			self.press();
			sleep(duration.randomizable_value());
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