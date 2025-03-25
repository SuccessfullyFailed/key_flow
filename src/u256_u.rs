#[cfg(test)]
mod tests {
	use crate::U256;



	#[test]
	fn test_add() {
		assert_eq!(U256::new(1, 0) + U256::new(0, 1), U256::new(1, 1));
	}

	#[test]
	fn test_mul() {
		assert_eq!(U256::new(0, 2) * U256::new(0, 3), U256::new(0, 6));
	}

	#[test]
	fn test_bitwise_and() {
		assert_eq!(U256::new(0b1100, 0b1010) & U256::new(0b1010, 0b1100), U256::new(0b1000, 0b1000));
	}

	#[test]
	fn test_bitwise_or() {
		assert_eq!(U256::new(0b1100, 0b1010) | U256::new(0b1010, 0b1100), U256::new(0b1110, 0b1110));
	}

	#[test]
	fn test_bitwise_xor() {
		assert_eq!(U256::new(0b1100, 0b1010) ^ U256::new(0b1010, 0b1100), U256::new(0b0110, 0b0110));
	}

	#[test]
	fn test_shift_left() {
		assert_eq!(U256::new(0, 1) << 128, U256::new(1, 0));
	}

	#[test]
	fn test_shift_right() {
		assert_eq!(U256::new(1, 0) >> 128, U256::new(0, 1));
	}
}
