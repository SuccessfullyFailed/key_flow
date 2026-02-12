use cachew::cache;
use winapi::um::winuser::{ GetSystemMetrics, INPUT, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT, MapVirtualKeyW, SM_CXSCREEN, SM_CYSCREEN, SendInput };
use crate::{ Key, KeyPattern, key_hook::handle_virtual_key_alteration, sleep };
use std::{ mem, ptr, thread, time::Duration };



#[derive(Clone)]
enum InputBuilderInput {
	KeyInput((INPUT, u8, bool)),
	MouseInput(INPUT),
	Delay(u64)
}
impl InputBuilderInput {
	fn delay(&self) -> u64 {
		match self {
			InputBuilderInput::Delay(delay) => *delay,
			_ => 0
		}
	}
	fn raw_input(&self) -> Option<&INPUT> {
		match self {
			InputBuilderInput::KeyInput((input, _, _)) => Some(input),
			InputBuilderInput::MouseInput(input) => Some(input),
			InputBuilderInput::Delay(_) => None
		}
	}
}


#[derive(Clone)]
pub struct InputBuilder {
	inputs:Vec<InputBuilderInput>
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

	/// Return self with a delay. All added actions after this one will happen after the delay.
	pub fn with_delay(mut self, duration_millis:u64) -> Self {
		self.add_delay(duration_millis);
		self
	}

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

	/// Return self with additional key-send inputs to type a specific str.
	pub fn with_send_str(mut self, string:&str, press_duration:u64) -> Self {
		self.add_send_str(string, press_duration);
		self
	}
	
	/// Return self with an additional mouse displacement.
	pub fn with_mouse_displacement(mut self, offset:[i32; 2]) -> Self {
		self.add_mouse_displacement(offset);
		self
	}

	/// Return self with an additional mouse movement.
	pub fn with_mouse_move(mut self, target_position:[i32; 2]) -> Self {
		self.add_mouse_move(target_position);
		self
	}



	/* INPUT ADDITION METHODS */

	/// Add a delay. All added actions after this one will happen after the delay.
	pub fn add_delay(&mut self, duration_millis:u64) {
		self.inputs.push(InputBuilderInput::Delay(duration_millis));
	}

	/// Add a key-press input.
	pub fn add_press(&mut self, key:&dyn KeyOrKeyPattern) {
		self.add_raw_key_inputs(key, true);
	}

	/// Add a key-release input.
	pub fn add_release(&mut self, key:&dyn KeyOrKeyPattern) {
		self.add_raw_key_inputs(key, false);
	}

	/// Add a key-click input. Presses it and immediately releases it.
	pub fn add_click(&mut self, key:&dyn KeyOrKeyPattern) {
		self.add_press(key);
		self.add_release(key);
	}

	/// Add a key-send input. Presses it and releases it later.
	pub fn add_send(&mut self, key:&dyn KeyOrKeyPattern, duration_millis:u64) {
		self.add_raw_key_inputs(key, true);
		self.add_delay(duration_millis);
		self.add_raw_key_inputs(key, false);
	}

	/// Add key-send inputs to type a specific str.
	pub fn add_send_str(&mut self, string:&str, press_duration:u64) {
		let key_patterns:Vec<KeyPattern> = KeyPattern::from_str(string);
		for key_pattern in key_patterns {
			self.add_send(&key_pattern, press_duration);
		}
	}
	
	/// Add a mouse displacement input.
	pub fn add_mouse_displacement(&mut self, offset:[i32; 2]) {
		self.add_raw_mouse_input(MOUSEEVENTF_MOVE, offset[0], offset[1])
	}

	/// Add a mouse move input.
	pub fn add_mouse_move(&mut self, target_position:[i32; 2]) {
		const POSITION_MULTIPLIER:i32 = 65535;

		let screen_size:&[i32; 2] = cache!([i32; 2], [GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)]);
		let normalized_position:[i32; 2] = [target_position[0] * POSITION_MULTIPLIER / screen_size[0], target_position[1] * POSITION_MULTIPLIER / screen_size[1]];
		self.add_raw_mouse_input(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK, normalized_position[0], normalized_position[1]);
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
			let next_delay_index:usize = self.inputs.iter().skip(cursor).position(|input| input.delay() != 0).unwrap_or(self.inputs.len() - 1);
			let execution_group:Vec<&InputBuilderInput> = self.inputs.iter().skip(cursor).take(next_delay_index + 1).collect();
			let delay_after_execution:u64 = execution_group.last().map(|last| last.delay()).unwrap_or_default();
			let inputs_to_execute:Vec<INPUT> = execution_group.iter().map(|input| input.raw_input()).flatten().cloned().collect();
			Self::execute_inputs(inputs_to_execute);

			// Update virtual key-states.
			for input in execution_group {
				match input {
					InputBuilderInput::KeyInput((_, key, down)) => handle_virtual_key_alteration(*key, *down),
					_ => {}
				}
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

	/// Add an input from core key-data.
	#[allow(invalid_value)]
	fn add_raw_key_inputs(&mut self, key:&dyn KeyOrKeyPattern, keys_down:bool) {
		const MOUSE_DOWN_EVENTS:[u32; 5] = [MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XDOWN];
		const MOUSE_UP_EVENTS:[u32; 5] = [MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_XUP, MOUSEEVENTF_XUP];

		self.inputs.extend(
			key.as_pattern().keys().into_iter().map(|key| {
				let key_code:u8 = key.key_code();
				InputBuilderInput::KeyInput((
					unsafe {
						if key_code < 6 {
							let mut input_record:INPUT = INPUT { type_: INPUT_MOUSE, u: mem::MaybeUninit::uninit().assume_init() };
							let event_list:&[u32; 5] = if keys_down { &MOUSE_DOWN_EVENTS } else { &MOUSE_UP_EVENTS };
							let flags:u32 = event_list[key_code as usize - 1];
							let input:MOUSEINPUT = MOUSEINPUT { dx: 0, dy: 0, mouseData: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 };
							ptr::write(&mut input_record.u as *mut _ as *mut MOUSEINPUT, input);
							input_record
						} else {
							let mut input_record:INPUT = INPUT { type_: 1, u: mem::MaybeUninit::uninit().assume_init() };
							let flags:u32 = if keys_down { 0 } else { KEYEVENTF_KEYUP };
							let input:KEYBDINPUT = KEYBDINPUT { wVk: key_code as u16, wScan: MapVirtualKeyW(key_code as u32, 0) as u16, dwFlags: flags, time: 0, dwExtraInfo: 0 };
							ptr::write(&mut input_record.u as *mut _ as *mut KEYBDINPUT, input);
							input_record
						}
					},
					key.key_code(),
					keys_down
				))
			}).collect::<Vec<InputBuilderInput>>()
		);
	}

	/// Add an input from core mouse-data.
	#[allow(invalid_value)]
	fn add_raw_mouse_input(&mut self, flags:u32, x:i32, y:i32) {
		self.inputs.push(
			InputBuilderInput::MouseInput(
				unsafe {
					let mut input_record:INPUT = INPUT { type_: INPUT_MOUSE, u: mem::MaybeUninit::uninit().assume_init() };
					let input:MOUSEINPUT = MOUSEINPUT { dx: x, dy: y, mouseData: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 };
					ptr::write(&mut input_record.u as *mut _ as *mut MOUSEINPUT, input);
					input_record
				}
			)
		);
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