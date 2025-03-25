use std::ops::{ Add, BitAnd, BitOr, BitXor, Mul, Shl, Shr };



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct U256 {
	high:u128,
	low:u128
}

impl U256 {

	/* CONSTRUCTOR METHODS */

	/// Create a new u128 integer.
	pub const fn new(high:u128, low:u128) -> U256 {
		U256 { high, low }
	}

	/// Create a new empty u128 integer.
	pub const fn zero() -> U256 {
		U256::new(0, 0)
	}



	/* MATH METHODS */

	/// Add two U256s. Returns new number and boolean indicating overflow.
	pub fn overflowing_add(self, rhs:U256) -> (U256, bool) {
		let (low, carry) = self.low.overflowing_add(rhs.low);
		let (high, high_carry) = self.high.overflowing_add(rhs.high + (carry as u128));
		(U256::new(high, low), high_carry)
	}

	/// Multiply two U256s. Returns new number and boolean indicating overflow.
	pub fn overflowing_mul(self, rhs:U256) -> (U256, bool) {
		let (low, carry) = self.low.overflowing_mul(rhs.low);
		let high = self.high * rhs.low + self.low * rhs.high + carry as u128;
		(U256::new(high, low), high > 0)
	}
}
impl Default for U256 {
	fn default() -> U256 {
		U256::zero()
	}
}
impl Add for U256 {
	type Output = U256;

	fn add(self, addition:U256) -> U256 {
		self.overflowing_add(addition).0
	}
}
impl Mul for U256 {
	type Output = U256;

	fn mul(self, multiplication:U256) -> U256 {
		self.overflowing_mul(multiplication).0
	}
}
impl Shl<u64> for U256 {
	type Output = U256;

	fn shl(self, shift:u64) -> U256 {
		if shift >= 128 {
			U256::new(self.low << (shift - 128), 0)
		} else {
			U256::new((self.high << shift) | (self.low >> (128 - shift)), self.low << shift)
		}
	}
}
impl Shr<u64> for U256 {
	type Output = U256;

	fn shr(self, shift:u64) -> U256 {
		if shift >= 128 {
			U256::new(0, self.high >> (shift - 128))
		} else {
			U256::new(self.high >> shift, (self.low >> shift) | (self.high << (128 - shift)))
		}
	}
}
impl BitAnd for U256 {
	type Output = U256;

	fn bitand(self, compare:U256) -> U256 {
		U256::new(self.high & compare.high, self.low & compare.low)
	}
}
impl BitOr for U256 {
	type Output = U256;

	fn bitor(self, compare:U256) -> U256 {
		U256::new(self.high | compare.high, self.low | compare.low)
	}
}
impl BitXor for U256 {
	type Output = U256;
	
	fn bitxor(self, compare:U256) -> U256 {
		U256::new(self.high ^ compare.high, self.low ^ compare.low)
	}
}