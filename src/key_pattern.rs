use std::ops::{ Add, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Shl, Shr };
use crate::Key;



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