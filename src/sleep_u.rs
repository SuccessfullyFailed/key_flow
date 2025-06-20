#[cfg(test)]
mod spin_sleep_test {
	use std::time::Instant;
	use crate::sleep;

	#[test]
	fn test_sleep_accuracy() {
		let sleep_times:&[u64] = &[2, 4, 8, 16, 32, 64, 512];
		for sleep_time in sleep_times {
			let test_start = Instant::now();
			sleep(*sleep_time);
			assert_eq!(test_start.elapsed().as_millis() as u64, *sleep_time);
		}

	}
}