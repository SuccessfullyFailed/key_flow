# key_flow

`key_flow` is a Rust crate that enables the creation of hotkeys, virtual key presses, and mouse movements using the Windows API. It allows users to automate input interactions, making it useful for scripting, automation, and accessibility purposes.

⚠ **Note:** This crate is **Windows-only**, as it relies on Windows API functions.

## Features

- Create global hotkeys to trigger actions
- Simulate key presses and releases
- Move and displace the mouse cursor
- Check the state of specific keys

## Installation

Add `key_flow` to your `Cargo.toml`:

```toml
[dependencies]
key_flow = { git="https://github.com/SuccessfullyFailed/key_flow" }
```

## Usage

Here’s an example demonstrating how to set up a hotkey, send virtual key inputs, and move the mouse:

```rust
use key_flow::{Hotkey, keys, mouse, key_hook};

fn main() {
	// Install the key hook to enable hotkey functionality.
	key_hook::install();

	// Set a hotkey to print a message when the spacebar is pressed.
	Hotkey::new(&[keys::SPACE]).on_press(|| {
		println!("SPACE PRESSED");
		println!("Lbutton state: {}", keys::LBUTTON.down());
	});

	// Simulate key presses and mouse movement.
	keys::CONTROL.press();
	keys::LBUTTON.send_await(100);
	keys::CONTROL.release();

	// Move the mouse to an absolute position.
	mouse::move_to([0, 0]);

	// Displace the mouse by an offset.
	mouse::displace([100, 50]);
}
```

## Functions & Features

### Hotkeys
- `Hotkey::new(&[keys::KEY]).on_press(|| { /* action */ }).on_release(|| { /* action */ }).blocking();` → Binds a function to a hotkey that will be stopped from iterating to the next processes.
- `key_hook::install();` → Enables global hotkey detection.

### Virtual Key Presses
- `keys::KEY.press();` → Presses a key.
- `keys::KEY.release();` → Releases a key.
- `keys::KEY.send_await(duration);` → Sends a key press for a duration (takes duration and integer as milliseconds).
- `keys::KEY.down();` → Checks if a key is currently held down.

### Mouse Manipulation
- `mouse::move_to([x, y]);` → Moves the mouse cursor to an absolute position.
- `mouse::displace([dx, dy]);` → Moves the mouse cursor relative to its current position.

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.