use std::{ ops::Range, time::Duration };
use rand::{ rngs::ThreadRng, Rng };
use cachew::cache;



pub trait RandomizableDuration {
	fn as_duration(&self) -> Duration;
	fn as_millis(&self) -> u64;
	fn is_empty(&self) -> bool;
}
impl RandomizableDuration for Duration {
	fn as_duration(&self) -> Duration {
		self.clone()
	}
	fn as_millis(&self) -> u64 {
		Duration::as_millis(&self) as u64
	}
	fn is_empty(&self) -> bool {
		self.as_millis() == 0
	}
}
impl RandomizableDuration for u64 {
	fn as_duration(&self) -> Duration {
		Duration::from_millis(*self)
	}
	fn as_millis(&self) -> u64 {
		*self
	}
	fn is_empty(&self) -> bool {
		*self == 0
	}
}
impl<T> RandomizableDuration for Range<T> where T:RandomizableDuration + PartialEq {
	fn as_duration(&self) -> Duration {
		Duration::from_millis(self.as_millis())
	}
	fn as_millis(&self) -> u64 {
		let rng:&mut ThreadRng = cache!(ThreadRng, rand::rng());
		rng.random_range(self.start.as_millis()..self.end.as_millis())
	}
	fn is_empty(&self) -> bool {
		self.start == self.end
	}
}



pub trait RandomizablePosition {
	fn get_value(&self) -> i32;
}
impl RandomizablePosition for i32 {
	fn get_value(&self) -> i32 {
		*self
	}
}
impl RandomizablePosition for usize {
	fn get_value(&self) -> i32 {
		*self as i32
	}
}
impl<T> RandomizablePosition for Range<T> where T:RandomizablePosition {
	fn get_value(&self) -> i32 {
		let rng:&mut ThreadRng = cache!(ThreadRng, rand::rng());
		rng.random_range(self.start.get_value()..self.end.get_value())
	}
}



pub trait RandomizableCoordinate {
	fn get_value(&self) -> [i32; 2];
}
impl<T> RandomizableCoordinate for [T; 2] where T:RandomizablePosition {
	fn get_value(&self) -> [i32; 2] {
		[self[0].get_value(), self[1].get_value()]
	}
}
impl<T, U> RandomizableCoordinate for (T, U) where T:RandomizablePosition, U:RandomizablePosition {
	fn get_value(&self) -> [i32; 2] {
		[self.0.get_value(), self.1.get_value()]
	}
}