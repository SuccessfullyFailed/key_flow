use circular_buffer::CircularBuffer;
use crate::{ Key, KeyPattern };



// As the mutations of hotkeys are done using 'mutation_lock', the list of hotkeys is only mutated by the key_hook thread. This means no Mutex is required for this list.
pub(crate) static mut REGISTERED_HOTKEYS:Vec<Hotkey> = Vec::new();



#[derive(Clone, Copy)]
enum ModificationRequest { Enable, Disable, Toggle, None }
impl Default for ModificationRequest {
	fn default() -> Self {
		ModificationRequest::None
	}
}
const MODIFICATIONS_QUEUE_SIZE:usize = 64;



pub struct Hotkey {
	id:u64,
	key_pattern:KeyPattern,
	on_press:Option<Box<dyn Fn() + Send + Sync>>,
	on_release:Option<Box<dyn Fn() + Send + Sync>>,
	blocking:bool,
	state:bool,
	enabled:bool,
	registered:bool,

	modifications_queue:CircularBuffer<ModificationRequest, MODIFICATIONS_QUEUE_SIZE> // Only one thread reads and writes this, so no async handling required
}
impl Hotkey {

	/* CONSTRUCTOR METHODS */

	/// Create a new hotkey.
	pub fn new(keys:&[Key]) -> Hotkey {
		static mut ID_GENERATOR:u64 = 0;
		Hotkey {
			id: unsafe { ID_GENERATOR += 1; ID_GENERATOR },
			key_pattern: keys.iter().map(|key| key.pattern()).reduce(|a, b| a ^ b).unwrap_or_default(),
			on_press: None,
			on_release: None,
			blocking: false,
			state: false,
			enabled: true,
			registered: false,

			modifications_queue: CircularBuffer::new()
		}
	}

	/// Return self with a handler that triggers when all keys are pressed.
	pub fn on_press<T>(mut self, handler:T) -> Self where T:Fn() + 'static + Send + Sync {
		self.on_press = Some(Box::new(handler));
		self
	}

	/// Return self with a handler that triggers when any of the keys are released after they were all held.
	pub fn on_release<T>(mut self, handler:T) -> Self where T:Fn() + 'static + Send + Sync {
		self.on_release = Some(Box::new(handler));
		self
	}

	/// Return self with blocking set to true. This will stop other processes from receiving the pressed hotkey.
	pub fn blocking(mut self) -> Self {
		self.blocking = true;
		self
	}

	/// Return self, but disabled.
	pub fn disabled(mut self) -> Self {
		self.enabled = false;
		self
	}



	/* REGISTERED STATIC METHODS */

	/// Register the hotkey to the static list. 
	#[allow(static_mut_refs)]
	pub fn register(mut self) -> HotkeyHandle {
		// As this consumes self, mutation lock is not required.

		// Set self as registered and create a handle.
		self.registered = true;
		let handle:HotkeyHandle = HotkeyHandle(self.id);

		// Push the hotkey to the registered hotkeys list.
		unsafe {
			if let Some(existing_index) = REGISTERED_HOTKEYS.iter().position(|existing_hotkey| existing_hotkey == &self) {
				REGISTERED_HOTKEYS[existing_index] = self;
			} else {
				REGISTERED_HOTKEYS.push(self);
			}
		}
		
		handle
	}



	/* PROPERTY GETTER METHODS */

	/// Wether or not the hotkey is enabled.
	pub fn enabled(&self) -> bool {
		self.enabled
	}



	/* USAGE METHODS */

	/// Create a request to enable the hotkey. Will be applied on the next update.
	pub fn enable(&mut self) {
		self.modifications_queue.push(ModificationRequest::Enable);
	}

	/// Create a request to disable the hotkey. Will be applied on the next update.
	pub fn disable(&mut self) {
		self.modifications_queue.push(ModificationRequest::Disable);
	}

	/// Create a request to toggle the hotkey. Will be applied on the next update.
	pub fn toggle(&mut self) {
		self.modifications_queue.push(ModificationRequest::Toggle);
	}

	/// Update the current state. Returns true if hotkey blocks.
	pub(crate) fn update_state(&mut self, active_pattern:&KeyPattern) -> bool {

		// Handle requested modifications.
		for modification in self.modifications_queue.take_all() {
			match modification {
				ModificationRequest::Enable => self.enabled = true,
				ModificationRequest::Disable => self.enabled = false,
				ModificationRequest::Toggle => self.enabled = !self.enabled,
				ModificationRequest::None => {}
			}
		}

		// Update state.
		if !self.enabled { return false; }
		let new_state:bool = self.key_pattern & *active_pattern == self.key_pattern;
		let mut executed_any:bool = false;
		if new_state != self.state {
			if let Some(handler) = if new_state { &self.on_press } else { &self.on_release } {
				handler();
				executed_any = true;
			}
		}
		self.state = new_state;

		// Return blocking state.
		executed_any && self.blocking
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
			eprintln!("Dropped unregistered hotkey. Are you sure you executed the 'register' hotkey on all created hotkeys?");
		}
	}
}



pub struct HotkeyHandle(u64);
impl HotkeyHandle {

	/// Execute something on the hotkey the handle is assigned to.
	#[allow(static_mut_refs)]
	fn execute<T>(&self, execution:T) where T:Fn(&mut Hotkey) {
		if let Some(hotkey) = unsafe { REGISTERED_HOTKEYS.iter_mut().find(|hotkey| hotkey.id == self.0) } {
			execution(hotkey);
		}
	}

	/// Enable the hotkey.
	pub fn enable(&mut self) {
		self.execute(|hotkey| hotkey.enable());
	}

	/// Disable the hotkey.
	pub fn disable(&mut self) {
		self.execute(|hotkey| hotkey.disable());
	}

	/// Toggle the hotkey.
	pub fn toggle(&mut self) {
		self.execute(|hotkey| hotkey.toggle());
	}
}