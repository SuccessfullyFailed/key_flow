use std::sync::{ Mutex, MutexGuard };
use crate::{ Key, U256 };



pub(crate) static REGISTERED_HOTKEYS:Mutex<Vec<Hotkey>> = Mutex::new(Vec::new());



pub struct Hotkey {
	id:u64,
	key_pattern:U256,
	on_press:Option<Box<dyn Fn() + Send + Sync>>,
	on_release:Option<Box<dyn Fn() + Send + Sync>>,
	blocking:bool,
	state:bool,
	enabled:bool,
	registered:bool
}
impl Hotkey {

	/* CONSTRUCTOR METHODS */

	/// Create a new hotkey.
	pub fn new(keys:&[Key]) -> Hotkey {
		static mut ID_GENERATOR:u64 = 0;
		Hotkey {
			id: unsafe { ID_GENERATOR += 1; ID_GENERATOR },
			key_pattern: keys.iter().map(|key| key.as_pattern()).reduce(|a, b| a ^ b).unwrap_or_default(),
			on_press: None,
			on_release: None,
			blocking: false,
			state: false,
			enabled: true,
			registered: false
		}
	}

	/// Set the press handler of the hotkey.
	pub fn on_press<T>(mut self, handler:T) -> Self where T:Fn() + 'static + Send + Sync {
		self.on_press = Some(Box::new(handler));
		self
	}

	/// Set the release handler of the hotkey.
	pub fn on_release<T>(mut self, handler:T) -> Self where T:Fn() + 'static + Send + Sync {
		self.on_release = Some(Box::new(handler));
		self
	}

	/// Set the hotkeys blocking state. This will stop other processes from receiving the pressed hotkey.
	pub fn blocking(mut self) -> Self {
		self.blocking = true;
		self
	}



	/* REGISTERED STATIC METHODS */

	/// Register the hotkey to the static list.
	pub fn register(mut self) -> HotkeyHandle {
		self.registered = true;
		let handle:HotkeyHandle = HotkeyHandle(self.id);

		let mut hotkey_list:MutexGuard<'_, Vec<Hotkey>> = REGISTERED_HOTKEYS.lock().unwrap();
		if let Some(existing_index) = hotkey_list.iter().position(|hotkey| hotkey == hotkey) {
			hotkey_list[existing_index] = self;
		} else {
			hotkey_list.push(self);
		}

		handle
	}



	/* PROPERTY GETTER METHPDS */

	/// Wether or not the hotkey is enabled.
	pub fn enabled(&self) -> bool {
		self.enabled
	}



	/* USAGE METHODS */

	/// Enable the hotkey.
	pub fn enable(&mut self) {
		self.enabled = true;
	}

	/// Disable the hotkey.
	pub fn disable(&mut self) {
		self.enabled = false;
	}

	/// Toggle the hotkey.
	pub fn toggle(&mut self) {
		self.enabled = !self.enabled;
	}

	/// Update the current state. Returns true if hotkey blocks.
	pub(crate) fn update_state(&mut self, active_pattern:&U256) -> bool {
		if !self.enabled { return false; }
		let new_state:bool = self.key_pattern & *active_pattern == self.key_pattern;
		if new_state != self.state {
			if let Some(handler) = if new_state { &self.on_press } else { &self.on_release } {
				handler();
			}
		}
		self.state = new_state;
		self.state && self.blocking
	}
}
impl PartialEq for Hotkey {
	fn eq(&self, other:&Self) -> bool {
		self.id == other.id
	}
}
impl Drop for Hotkey {
	fn drop(&mut self) {
		if !self.registered {
			eprintln!("Dropped unregistered hotkey. Are you sure youe xecuteds the 'register' hotkey on all created hotkeys?");
		}
	}
}



pub struct HotkeyHandle(u64);
impl HotkeyHandle {

	/// Execute something on the hotkey the handle is assigned to.
	fn execute<T>(&self, execution:T) where T:Fn(&mut Hotkey) {
		if let Some(hotkey) = REGISTERED_HOTKEYS.lock().unwrap().iter_mut().find(|hotkey| hotkey.id == self.0) {
			execution(hotkey);
		}
	}

	/// Enable the hotkey.
	pub fn enable(&mut self) {
		self.execute(|hotkey| hotkey.enabled = true);
	}

	/// Disable the hotkey.
	pub fn disable(&mut self) {
		self.execute(|hotkey| hotkey.enabled = false);
	}

	/// Toggle the hotkey.
	pub fn toggle(&mut self) {
		self.execute(|hotkey| hotkey.enabled = !hotkey.enabled);
	}
}