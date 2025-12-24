use winapi::um::winuser::{ INPUT, KEYBDINPUT, KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT, MapVirtualKeyW, SendInput };
use crate::{ Key, KeyPattern, key_hook::handle_virtual_key_alteration, sleep };
use std::{ mem, ptr, thread, time::Duration };



#[derive(Clone)]
pub struct InputBuilder {
	inputs:Vec<(INPUT, u64, u8, bool)>
}
impl InputBuilder {

	/* CONSTRUCTOR METHODS */

	/// Create a new, empty input builder.
	pub fn new() -> InputBuilder {
		InputBuilder {
			inputs: Vec::new()
		}
	}



	/* INPUT BUILDER METHODS */

	/// Return self with an additional key-press input.
	pub fn with_press(mut self, key:&dyn KeyOrKeyPattern) -> Self {
		self.add_press(key);
		self
	}

	/// Return self with an additional key-release input.
	pub fn with_release(mut self, key:&dyn KeyOrKeyPattern) -> Self {
		self.add_release(key);
		self
	}

	/// Return self with an additional key-click input.
	pub fn with_click(mut self, key:&dyn KeyOrKeyPattern) -> Self {
		self.add_click(key);
		self
	}

	/// Return self with an additional key-send input.
	pub fn with_send(mut self, key:&dyn KeyOrKeyPattern, press_duration:u64) -> Self {
		self.add_send(key, press_duration);
		self
	}

	/// Return self with addition key-send inputs to type a specific str.
	pub fn with_send_str(mut self, string:&str, press_duration:u64) -> Self {
		self.add_send_str(string, press_duration);
		self
	}



	/* INPUT ADDITION METHODS */

	/// Add a key-press input.
	pub fn add_press(&mut self, key:&dyn KeyOrKeyPattern) {
		self.add_raw_inputs(key, true, 0);
	}

	/// Add a key-release input.
	pub fn add_release(&mut self, key:&dyn KeyOrKeyPattern) {
		self.add_raw_inputs(key, false, 0);
	}

	/// Add a key-click input. Presses it and immediately releases it.
	pub fn add_click(&mut self, key:&dyn KeyOrKeyPattern) {
		self.add_press(key);
		self.add_release(key);
	}

	/// Add a key-send input. Presses it and releases it later.
	pub fn add_send(&mut self, key:&dyn KeyOrKeyPattern, duration_millis:u64) {
		self.add_raw_inputs(key, true, duration_millis);
		self.add_raw_inputs(key, false, duration_millis);
	}

	/// Add key-send inputs to type a specific str.
	pub fn add_send_str(&mut self, string:&str, press_duration:u64) {
		let key_patterns:Vec<KeyPattern> = KeyPattern::from_str(string);
		for key_pattern in key_patterns {
			self.add_send(&key_pattern, press_duration);
		}
	}



	/* USAGE METHODS */

	/// Send the inputs in a separate thread.
	pub fn execute_async(&self) {
		let clone:InputBuilder = self.clone();
		thread::spawn(move || clone.execute());
	}

	/// Send the inputs and wait for all of them to finish.
	pub fn execute(&self) {
		let mut cursor:usize = 0;
		while cursor < self.inputs.len() {

			// Group all inputs that are scheduled to execute at the same time.
			let next_delay_index:usize = self.inputs.iter().skip(cursor).position(|(_, delay, _, _)| *delay != 0).unwrap_or(self.inputs.len() - 1);
			let execution_group:Vec<&(INPUT, u64, u8, bool)> = self.inputs.iter().skip(cursor).take(next_delay_index + 1).collect();
			let delay_after_execution:u64 = execution_group.last().map(|last| last.1).unwrap_or_default();
			let inputs_to_execute:Vec<INPUT> = execution_group.iter().map(|(input, _, _, _)| input.clone()).collect();
			Self::execute_inputs(inputs_to_execute);

			// Update virtual key-states.
			for (_, _, key, down) in execution_group {
				handle_virtual_key_alteration(*key, *down);
			}

			// Sleep any possible delays.
			if delay_after_execution > 0 {
				sleep(Duration::from_millis(delay_after_execution));
			}

			// Move cursor to next execution group.
			cursor += next_delay_index + 1;
		}
	}

	/// Execute the given inputs.
	fn execute_inputs(mut inputs:Vec<INPUT>) {
		unsafe { SendInput(inputs.len() as u32, inputs.as_mut_ptr(), mem::size_of::<INPUT>() as i32) };
	}



	/* RAW INPUT METHODS */

	/// Add an input from core data.
	fn add_raw_inputs(&mut self, key:&dyn KeyOrKeyPattern, keys_down:bool, delay:u64) {
		self.inputs.extend(
			key.as_pattern().keys().into_iter().map(|key| (
				Self::create_raw_input(key.key_code(), keys_down),
				delay,
				key.key_code(),
				keys_down
			)).collect::<Vec<(INPUT, u64, u8, bool)>>()
		);
	}

	/// Create a raw input from core data.
	fn create_raw_input(key_code:u8, key_down:bool) -> INPUT {
		if key_code < 5 {
			Self::create_raw_input_mouse(key_code, key_down)
		} else {
			Self::create_raw_input_keyboard(key_code, key_down)
		}
	}

	/// Create a raw keyboard input from core data.
	#[allow(invalid_value)]
	fn create_raw_input_keyboard(key_code:u8, key_down:bool) -> INPUT {
		unsafe {
			let mut input_record:INPUT = INPUT { type_: 1, u: mem::MaybeUninit::uninit().assume_init() };
			let flags:u32 = if key_down { 0 } else { KEYEVENTF_KEYUP };
			let input:KEYBDINPUT = KEYBDINPUT { wVk: key_code as u16, wScan: MapVirtualKeyW(key_code as u32, 0) as u16, dwFlags: flags, time: 0, dwExtraInfo: 0 };
			ptr::write(&mut input_record.u as *mut _ as *mut KEYBDINPUT, input);
			input_record
		}
	}

	/// Create a raw mouse input from core data.
	#[allow(invalid_value)]
	fn create_raw_input_mouse(key_code:u8, key_down:bool) -> INPUT {
		const MOUSE_DOWN_EVENTS:[u32; 5] = [MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XDOWN];
		const MOUSE_UP_EVENTS:[u32; 5] = [MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_XUP, MOUSEEVENTF_XUP];
		let event_list:&[u32; 5] = if key_down { &MOUSE_DOWN_EVENTS } else { &MOUSE_UP_EVENTS };

		unsafe {
			let mut input_record:INPUT = INPUT { type_: 1, u: mem::MaybeUninit::uninit().assume_init() };
			let flags:u32 = event_list[key_code as usize];
			let input:MOUSEINPUT = MOUSEINPUT { dx: 0, dy: 0, mouseData: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 };
			ptr::write(&mut input_record.u as *mut _ as *mut MOUSEINPUT, input);
			input_record
		}
	}
}



pub trait KeyOrKeyPattern {
	fn as_pattern(&self) -> KeyPattern;
}
impl KeyOrKeyPattern for KeyPattern {
	fn as_pattern(&self) -> KeyPattern {
		self.clone()
	}
}
impl KeyOrKeyPattern for Key {
	fn as_pattern(&self) -> KeyPattern {
		Key::pattern(self)
	}
}