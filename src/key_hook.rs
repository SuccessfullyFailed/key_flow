use winapi::{shared::{minwindef::{LPARAM, LRESULT, WPARAM}, windef::HHOOK__}, um::winuser::{CallNextHookEx, KBDLLHOOKSTRUCT, LLKHF_INJECTED, LLMHF_INJECTED, MSLLHOOKSTRUCT, WM_KEYDOWN, WM_KEYUP}};
use std::{ptr, sync::{Mutex, MutexGuard}};
use crate::U256;



static mut LISTENER_THREAD_ID:Mutex<Option<u32>> = Mutex::new(None);
static mut PHYSICAL_KEY_STATES:U256 = U256::zero();



/* HOOK INSTALLATION METHODS */

/// Install the mouse and keyboard hook.
#[allow(static_mut_refs)]
pub fn install() {
	use winapi::um::{ processthreadsapi::GetCurrentThreadId, winuser::{ SetWindowsHookExW, WH_MOUSE_LL, WH_KEYBOARD_LL, GetMessageW } };
	use std::{ thread, ptr::null_mut };

	// Create activate listener thread if not exists.
	thread::spawn(|| unsafe {
		let mut listener_thread_id:MutexGuard<'_, Option<u32>> = LISTENER_THREAD_ID.lock().unwrap();
		if listener_thread_id.is_none() {
			*listener_thread_id = Some(GetCurrentThreadId());
	
			// Create new hooks
			let hook_mouse:*mut HHOOK__ = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_callback), null_mut(), 0);
			let hook_keyboard:*mut HHOOK__  = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_callback), null_mut(), 0);
			if hook_mouse.is_null() {
				panic!("Failed to install mouse hook");
			}
			if hook_keyboard.is_null() {
				panic!("Failed to install keyboard hook");
			}
			
			// Start the message listener.
			GetMessageW(null_mut(), null_mut(), 0, 0);
		}
	});
}



/* HOOK HANDLING METHODS */

/// The callback to catch the pressed keys.
unsafe extern "system" fn hook_callback(key_code:i32, w_param:WPARAM, l_param:LPARAM) -> LRESULT {
	let w_param:usize = w_param as usize;

	// Find key id and state change from arguments.
	if key_code >= 0 {
		if let Some((key, down)) = params_to_key_alteration(w_param as u32, l_param) {
			handle_key_alteration(key, down);
		}
	}

	// Move on to next callback.
	unsafe { CallNextHookEx(ptr::null_mut(), key_code, w_param, l_param) }
}

/// Figure out a pressed key-code and a boolean indicating the key being pressed or not from hook callback arguments.
fn params_to_key_alteration(w_param:u32, l_param:LPARAM) -> Option<(u32, bool)> {

	// Keyboard
	if w_param == WM_KEYDOWN || w_param == WM_KEYUP {
		let kbd:&KBDLLHOOKSTRUCT = unsafe { &*(l_param as *const KBDLLHOOKSTRUCT) };
		if kbd.flags & LLKHF_INJECTED == 0 {
			return Some((kbd.vkCode, w_param == WM_KEYDOWN));
		}
	}

	// Mouse 
	else if (0x201..0x20C).contains(&w_param) {
		let md:MSLLHOOKSTRUCT = unsafe { *(l_param as *const MSLLHOOKSTRUCT) };
		if md.flags & LLMHF_INJECTED == 0 {
			return match w_param {
				0x201 => Some((0x01, true)),
				0x202 => Some((0x01, false)),
				0x204 => Some((0x02, true)),
				0x205 => Some((0x02, false)),
				0x207 => Some((0x03, true)),
				0x208 => Some((0x03, false)),
				_ => None
			};
		}
	}

	// No key found.
	None
}

/// Handle a key being pressed or released.
fn handle_key_alteration(key_code:u32, down:bool) {
	unsafe {
		let modification_key:U256 = if key_code < 128 { U256::new(0, 1 << (key_code - 1)) } else { U256::new(1 << (key_code - 129), 1 << (key_code - 1)) };
		if down {
			PHYSICAL_KEY_STATES = PHYSICAL_KEY_STATES ^ modification_key;
		} else {
			PHYSICAL_KEY_STATES = PHYSICAL_KEY_STATES & !modification_key;
		}
	}
}