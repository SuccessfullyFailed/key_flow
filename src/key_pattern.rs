use mini_rand::Randomizable;
use winapi::um::winuser::{ INPUT, KEYBDINPUT, KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT, MapVirtualKeyW, SendInput };
use std::{ mem, ops::{ Add, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Shl, Shr }, ptr, thread, time::Duration };
use crate::{ Key, key_hook::{PHYSICAL_KEY_STATES, handle_virtual_key_alteration}, sleep };



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyPattern {
	high:u128,
	low:u128
}
impl KeyPattern {
	pub const ZERO:KeyPattern = KeyPattern { high: 0, low: 0 };



	/* CONSTRUCTOR METHODS */

	/// Create a new u128 integer.
	pub const fn new(high:u128, low:u128) -> KeyPattern {
		KeyPattern { high, low }
	}

	/// Create a new empty u128 integer.
	pub const fn zero() -> KeyPattern {
		KeyPattern { high: 0, low: 0 }
	}

	/// Try to get a list of keys from a str.
	pub fn from_str(source:&str) -> Vec<KeyPattern> {
		source.chars().map(|character| KeyPattern::from_char(character)).collect()
	}

	/// Create a key-pattern from a character.
	pub fn from_char(character:char) -> KeyPattern {
		Key::from_char(character).into_iter().map(|key| key.as_pattern()).reduce(|a, b| a | b).unwrap_or_default()
	}



	/* MATH METHODS */

	/// Add two U256s. Returns new number and boolean indicating overflow.
	pub fn overflowing_add(self, rhs:KeyPattern) -> (KeyPattern, bool) {
		let (low, carry) = self.low.overflowing_add(rhs.low);
		let (high, high_carry) = self.high.overflowing_add(rhs.high + (carry as u128));
		(KeyPattern { high, low }, high_carry)
	}

	/// Multiply two U256s. Returns new number and boolean indicating overflow.
	pub fn overflowing_mul(self, rhs:KeyPattern) -> (KeyPattern, bool) {
		let (low, carry) = self.low.overflowing_mul(rhs.low);
		let high = self.high * rhs.low + self.low * rhs.high + carry as u128;
		(KeyPattern { high, low }, high > 0)
	}



	/* USAGE METHODS */

	/// Convert the pattern to a list of keys.
	pub fn as_keys(&self) -> Vec<Key> {
		let mut keys:Vec<Key> = Vec::new();
		for index in 0..128 {
			if self.low >> index & 1 == 1 {
				keys.push(Key::new(index + 1));
			}
		}
		for index in 0..128 {
			if self.high >> index & 1 == 1 {
				keys.push(Key::new(index + 129));
			}
		}
		keys
	}

	/// Returns self, filtered by the keys that are pressed.
	pub fn pressed_pattern(&self) -> KeyPattern {
		*self & unsafe { PHYSICAL_KEY_STATES }
	}

	/// Whether or not the pattern is completely pressed.
	pub fn all_pressed(&self) -> bool {
		self.pressed_pattern() == *self
	}

	/// Press all keys in the pattern.
	pub fn press(&self) {
		self.create_keyboard_event(true);
	}

	/// Release all keys in the pattern.
	pub fn release(&self) {
		self.create_keyboard_event(false);
	}

	/// Send the key.
	pub fn send<T>(&self, duration:T) where T:Randomizable<Duration> {
		let duration:Duration = duration.randomizable_value();
		if duration.is_zero() {
			self.press();
			self.release();
		} else {
			let pattern:KeyPattern = self.clone();
			thread::spawn(move || {
				pattern.press();
				sleep(duration);
				pattern.release();
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

	/// Create a windows keyboard event.
	#[allow(invalid_value)]
	fn create_keyboard_event(&self, keys_down:bool) {
		const MOUSE_DOWN_EVENTS:[u32; 5] = [MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XDOWN];
		const MOUSE_UP_EVENTS:[u32; 5] = [MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_XUP, MOUSEEVENTF_XUP];
		unsafe {

			// Create list of keys.
			let mut keys:Vec<Key> = self.as_keys();
			if !keys_down {
				keys.reverse();
			}

			// Create inputs.
			let mut input_records:Vec<INPUT> = keys.iter().map(|_| INPUT { type_: 1, u: mem::MaybeUninit::uninit().assume_init() }).collect();
			for (index, key) in keys.iter().enumerate() {
				if key.key_code() < 5  {
					let flags:u32 = (if keys_down { MOUSE_DOWN_EVENTS } else { MOUSE_UP_EVENTS })[key.key_code() as usize];
					let input:MOUSEINPUT = MOUSEINPUT { dx: 0, dy: 0, mouseData: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 };
					ptr::write(&mut input_records[index].u as *mut _ as *mut MOUSEINPUT, input);
				} else {
					let flags:u32 = if keys_down { 0 } else { KEYEVENTF_KEYUP };
					let input:KEYBDINPUT = KEYBDINPUT { wVk: key.key_code() as u16, wScan: MapVirtualKeyW(key.key_code() as u32, 0) as u16, dwFlags: flags, time: 0, dwExtraInfo: 0 };
					ptr::write(&mut input_records[index].u as *mut _ as *mut KEYBDINPUT, input);
				}
			}

			// Send all inputs at once.
			SendInput(input_records.len() as u32, input_records.as_mut_ptr(), mem::size_of::<INPUT>() as i32);

			// Update virtual keystate.
			handle_virtual_key_alteration(self.clone(), keys_down);
		}
	}
}
impl Default for KeyPattern {
	fn default() -> KeyPattern {
		KeyPattern::zero()
	}
}
impl Add for KeyPattern {
	type Output = KeyPattern;

	fn add(self, addition:KeyPattern) -> KeyPattern {
		self.overflowing_add(addition).0
	}
}
impl Mul for KeyPattern {
	type Output = KeyPattern;

	fn mul(self, multiplication:KeyPattern) -> KeyPattern {
		self.overflowing_mul(multiplication).0
	}
}
impl Shl<u64> for KeyPattern {
	type Output = KeyPattern;

	fn shl(self, shift:u64) -> KeyPattern {
		if shift >= 128 {
			KeyPattern{ high: self.low << (shift - 128), low: 0 }
		} else {
			KeyPattern{ high: (self.high << shift) | (self.low >> (128 - shift)), low: self.low << shift }
		}
	}
}
impl Shr<u64> for KeyPattern {
	type Output = KeyPattern;

	fn shr(self, shift:u64) -> KeyPattern {
		if shift >= 128 {
			KeyPattern { high: 0, low: self.high >> (shift - 128) }
		} else {
			KeyPattern { high: self.high >> shift, low: (self.low >> shift) | (self.high << (128 - shift)) }
		}
	}
}
impl BitAnd<KeyPattern> for KeyPattern {
	type Output = KeyPattern;

	fn bitand(self, compare:KeyPattern) -> KeyPattern {
		KeyPattern { high: self.high & compare.high, low: self.low & compare.low }
	}
}
impl BitAnd<Key> for KeyPattern {
	type Output = KeyPattern;

	fn bitand(self, compare:Key) -> KeyPattern {
		self & compare.as_pattern()
	}
}
impl BitAndAssign<KeyPattern> for KeyPattern {
	fn bitand_assign(&mut self, compare:KeyPattern) {
		self.high &= compare.high;
		self.low &= compare.low;
	}
}
impl BitAndAssign<Key> for KeyPattern {
	fn bitand_assign(&mut self, compare:Key) {
		let compare:KeyPattern = compare.as_pattern();
		self.high &= compare.high;
		self.low &= compare.low;
	}
}
impl BitOr<KeyPattern> for KeyPattern {
	type Output = KeyPattern;

	fn bitor(self, compare:KeyPattern) -> KeyPattern {
		KeyPattern { high: self.high | compare.high, low: self.low | compare.low }
	}
}
impl BitOr<Key> for KeyPattern {
	type Output = KeyPattern;

	fn bitor(self, compare:Key) -> KeyPattern {
		self | compare.as_pattern()
	}
}
impl BitOrAssign<KeyPattern> for KeyPattern {
	fn bitor_assign(&mut self, compare:KeyPattern) {
		self.high |= compare.high;
		self.low |= compare.low;
	}
}
impl BitOrAssign<Key> for KeyPattern {
	fn bitor_assign(&mut self, compare:Key) {
		let compare:KeyPattern = compare.as_pattern();
		self.high |= compare.high;
		self.low |= compare.low;
	}
}
impl BitXor<KeyPattern> for KeyPattern {
	type Output = KeyPattern;
	
	fn bitxor(self, compare:KeyPattern) -> KeyPattern {
		KeyPattern { high: self.high ^ compare.high, low: self.low ^ compare.low }
	}
}
impl BitXor<Key> for KeyPattern {
	type Output = KeyPattern;
	
	fn bitxor(self, compare:Key) -> KeyPattern {
		self ^ compare.as_pattern()
	}
}
impl BitXorAssign<KeyPattern> for KeyPattern {
	fn bitxor_assign(&mut self, compare:KeyPattern) {
		self.high ^= compare.high;
		self.low ^= compare.low;
	}
}
impl BitXorAssign<Key> for KeyPattern {
	fn bitxor_assign(&mut self, compare:Key) {
		let compare:KeyPattern = compare.as_pattern();
		self.high ^= compare.high;
		self.low ^= compare.low;
	}
}
impl Not for KeyPattern {
	type Output = KeyPattern;

	fn not(self) -> Self::Output {
		KeyPattern { high: !self.high, low: !self.low }
	}
}