use crate::RandomizableDuration;



/// Sleep for the given duration. Will use the more accurate custom method if "sleep" feature is enabled. Otherwise will fallback to std::thread::sleep method.
pub fn sleep<T>(duration:T) where T:RandomizableDuration {
	#[cfg(feature="sleep")]
	custom_sleep::sleep(duration);
	#[cfg(not(feature="sleep"))]
	std::thread::sleep(duration.as_duration());
}

#[cfg(feature="sleep")]
mod custom_sleep {
	use windows_sys::Win32::{ Foundation::FALSE, Media::{ timeGetDevCaps, TIMECAPS, TIMERR_NOERROR }, System::Threading::{ CreateWaitableTimerExW, SetWaitableTimer, WaitForSingleObject, CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, INFINITE, TIMER_ALL_ACCESS } };
	use std::{ error::Error, ffi::c_void, mem, ptr::null, sync::{ Mutex, MutexGuard }, time::{ Duration, Instant } };
	use crate::RandomizableDuration;

	/// A more accurate version of std::thread::sleep.
	pub fn sleep<T>(duration:T) where T:RandomizableDuration {

		// Get sleeping arguments.
		let duration:Duration = duration.as_duration();
		let native_accuracy:u64 = windows_native_sleep_accuracy_nanoseconds();
		let deadline:Instant = Instant::now() + duration;
		let accuracy:Duration = Duration::from_nanos(native_accuracy);

		// If duration is more than windows native accuracy, use windows native sleep or rust built-in sleep function.
		if duration > accuracy {
			if sleep_windows_native(&duration).is_err() {
				std::thread::sleep(duration);
			}
		}
		
		// For the remaining time, repeat a very small sleep until deadline is reached.
		while Instant::now() < deadline {
			std::hint::spin_loop()
		}
		
	}

	/// Use a windows native timer to await the given duration.
	fn sleep_windows_native(duration:&Duration) -> Result<(), Box<dyn Error>> {

		// Execute action on thead-bound windows timer.
		WINDOWS_TIMER.with(|timer| unsafe {

			// Validate timer.
			if timer.is_null() {
				return Err("Could not initialize windows native timer.".into());
			}

			// Convert duration to the 100-nanoseconds interval count windows is expecting.
			let time:u64 = duration.as_nanos() as u64 / 100;

			// Set timer.
			let time_i64:i64 = time as i64;
			if SetWaitableTimer(*timer, &time_i64 as *const i64, 0, None, null(), FALSE) == 0 {
				return Err("Could not set windows native timer.".into());
			}

			// Block execution until timer is finished.
			if WaitForSingleObject(*timer, INFINITE) == windows_sys::Win32::Foundation::WAIT_FAILED {
				return Err("Could not wait for timer".into());
			}

			// Return success.
			Ok(())
		})
	}

	/// Get windows' native sleep accuracy.
	fn windows_native_sleep_accuracy_nanoseconds() -> u64 {

		// If windows native timer works, use default accuracy for that.
		if WINDOWS_TIMER.with(|timer| timer.is_null()) {
			// 500..1000 nanoseconds should fix most oversleeps.
			750_000
		}
		
		// If windows native timer not working, calculate other time period.
		else {
			static MIN_TIME_PERIOD:Mutex<Option<u64>> = Mutex::new(None);

			// Query system for timing.
			let mut time_period:MutexGuard<'_, Option<u64>> = MIN_TIME_PERIOD.lock().unwrap();
			if time_period.is_none() {
				let tc_size:u32 = mem::size_of::<TIMECAPS>() as u32;
				let mut tc:TIMECAPS = TIMECAPS { wPeriodMin:0, wPeriodMax:0 };
				let native_min_time_period:u64 = if unsafe { timeGetDevCaps(&mut tc as *mut TIMECAPS, tc_size) } == TIMERR_NOERROR { tc.wPeriodMin as u64 } else { 1 };
				*time_period = Some(native_min_time_period * 1_000_000);
			}
			time_period.unwrap()
		}
	}



	// Create thread-relative timer. Allows own timer for each thread.
	thread_local! {
		static WINDOWS_TIMER:*mut c_void = unsafe { CreateWaitableTimerExW(null(), null(), CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, TIMER_ALL_ACCESS) };
	}
}