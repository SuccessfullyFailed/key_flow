#[cfg(test)]
mod tests {
	use crate::KeyPattern;



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
	}

	#[test]
	fn test_key_pattern_bitwise_or() {
		assert_eq!(KeyPattern::new(0b1100, 0b1010) | KeyPattern::new(0b1010, 0b1100), KeyPattern::new(0b1110, 0b1110));
	}

	#[test]
	fn test_key_pattern_bitwise_xor() {
		assert_eq!(KeyPattern::new(0b1100, 0b1010) ^ KeyPattern::new(0b1010, 0b1100), KeyPattern::new(0b0110, 0b0110));
	}

	#[test]
	fn test_key_pattern_shift_left() {
		assert_eq!(KeyPattern::new(0, 1) << 128, KeyPattern::new(1, 0));
	}

	#[test]
	fn test_key_pattern_shift_right() {
		assert_eq!(KeyPattern::new(1, 0) >> 128, KeyPattern::new(0, 1));
	}
}
