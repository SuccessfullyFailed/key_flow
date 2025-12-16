use crate::{ InputBuilder, KeyPattern, key_hook, keys };
use mini_rand::Randomizable;
use std::time::Duration;



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



	/* PROPERTY GETTER METHODS */

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

	/// Whether or not this key is a modifier key, a key that usually doesn't do anything on it's own, but modifies other keys. Shift or Control for example.
	pub fn is_modifier_key(&self) -> bool {
		const MODIFIER_KEYS:&[Key] = &[keys::SHIFT, keys::LSHIFT, keys::RSHIFT, keys::CONTROL, keys::LCONTROL, keys::RCONTROL, keys::ALT, keys::RALT, keys::LALT, keys::LWIN, keys::RWIN];
		MODIFIER_KEYS.contains(self)
	}



	/* USAGE METHODS */

	/// Press the key.
	pub fn press(&self) {
		InputBuilder::new().with_press(self).execute();
	}

	/// Release the key.
	pub fn release(&self) {
		InputBuilder::new().with_release(self).execute();
	}

	/// Send the key.
	pub fn send<T>(&self, duration:T) where T:Randomizable<Duration> {
		InputBuilder::new().with_send(self, duration.randomizable_value().as_millis() as u64).execute();
	}

	/// Send the key in a separate thread.
	pub fn send_async<T>(&self, duration:T) where T:Randomizable<Duration> {
		InputBuilder::new().with_send(self, duration.randomizable_value().as_millis() as u64).execute_async();
	}
}
impl PartialEq for Key {
	fn eq(&self, other:&Self) -> bool {
		self.0 == other.0
	}
}