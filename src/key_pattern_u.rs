#[cfg(test)]
mod tests {
	use crate::{ Key, KeyPattern };



	#[test]
	fn test_key_pattern_add() {
		assert_eq!(KeyPattern::new(1, 0) + KeyPattern::new(0, 1), KeyPattern::new(1, 1));
	}

	#[test]
	fn test_key_pattern_mul() {
		assert_eq!(KeyPattern::new(0, 2) * KeyPattern::new(0, 3), KeyPattern::new(0, 6));
	}

	#[test]
	fn test_key_pattern_bitwise_and() {
		assert_eq!(KeyPattern::new(0b1100, 0b1010) & KeyPattern::new(0b1010, 0b1100), KeyPattern::new(0b1000, 0b1000));
		assert_eq!(KeyPattern::new(0b1100, 0b1010) & Key::new(0b0010), KeyPattern::new(0b0000, 0b0010));

		let mut value:KeyPattern = KeyPattern::new(0b1100, 0b1010);
		value &= KeyPattern::new(0b1010, 0b1100);
		assert_eq!(value, KeyPattern::new(0b1000, 0b1000));

		let mut value:KeyPattern = KeyPattern::new(0b1100, 0b1010);
		value &= Key::new(0b0010);
		assert_eq!(value, KeyPattern::new(0b0000, 0b0010));
	}

	#[test]
	fn test_key_pattern_bitwise_or() {
		assert_eq!(KeyPattern::new(0b1100, 0b1010) | KeyPattern::new(0b1010, 0b1100), KeyPattern::new(0b1110, 0b1110));
		assert_eq!(KeyPattern::new(0b1100, 0b1010) | Key::new(0b0001), KeyPattern::new(0b1100, 0b1011));
		assert_eq!(KeyPattern::new(0b1100, 0b1010) | Key::new(0b0010), KeyPattern::new(0b1100, 0b1010));

		let mut value:KeyPattern = KeyPattern::new(0b1100, 0b1010);
		value |= KeyPattern::new(0b1010, 0b1100);
		assert_eq!(value, KeyPattern::new(0b1110, 0b1110));

		let mut value:KeyPattern = KeyPattern::new(0b1100, 0b1010);
		value |= Key::new(0b0001);
		assert_eq!(value, KeyPattern::new(0b1100, 0b1011));
	}

	#[test]
	fn test_key_pattern_bitwise_xor() {
		assert_eq!(KeyPattern::new(0b1100, 0b1010) ^ KeyPattern::new(0b1010, 0b1100), KeyPattern::new(0b0110, 0b0110));
		assert_eq!(KeyPattern::new(0b1100, 0b1010) ^ Key::new(0b0001), KeyPattern::new(0b1100, 0b1011));

		let mut value:KeyPattern = KeyPattern::new(0b1100, 0b1010);
		value ^= KeyPattern::new(0b1010, 0b1100);
		assert_eq!(value, KeyPattern::new(0b0110, 0b0110));

		let mut value:KeyPattern = KeyPattern::new(0b1100, 0b1010);
		value ^= Key::new(0b0001);
		assert_eq!(value, KeyPattern::new(0b1100, 0b1011));
	}

	#[test]
	fn test_key_pattern_shift_left() {
		assert_eq!(KeyPattern::new(0, 1) << 128, KeyPattern::new(1, 0));
	}

	#[test]
	fn test_key_pattern_shift_right() {
		assert_eq!(KeyPattern::new(1, 0) >> 128, KeyPattern::new(0, 1));
	}

	#[test]
	fn test_all_keys_to_pattern() {
		for key_code in 0..0xFF {
			println!("Converting key with code {} to pattern.", key_code);
			Key::new(key_code).as_pattern();
		}
	}

	#[test]
	fn test_pattern_to_key_list() {
		let key_codes:[u8; 6] = [3, 6, 9, 22, 127, 255];
		let pattern:KeyPattern = key_codes.iter().map(|code| Key::new(*code).as_pattern()).reduce(|a, b| a | b).unwrap();
		assert_eq!(pattern.as_keys(), key_codes.iter().map(|code| Key::new(*code)).collect::<Vec<Key>>());
	}
}
