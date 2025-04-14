use std::{ error::Error, thread::sleep, time::Duration };
use minifb::{ Window, WindowOptions, MouseMode };
use super::mouse::MouseProgressionPath;
use rand::Rng;



pub struct MouseRecording(Vec<MouseProgressionPath>);
impl MouseRecording {

	/// Create a mouse recording by playing a target practice game.
	pub fn create(window_size:[usize; 2], target_paths_count:usize) -> Result<MouseRecording, Box<dyn Error>> {
		use rand::prelude::ThreadRng;

		// Window settings.
		const INTERVAL:Duration = Duration::from_millis(10);
		const PADDING:f32 = 0.1;
		let inner_size:[usize; 4] = [
			(window_size[0] as f32 * PADDING) as usize,
			(window_size[1] as f32 * PADDING) as usize,
			(window_size[0] as f32 * (1.0 - 2.0 * PADDING)) as usize,
			(window_size[1] as f32 * (1.0 - 2.0 * PADDING)) as usize
		];

		// Target settings.
		const TARGET_SIZE:usize = 10;
		const TARGET_COLOR:u32 = 0xFFFF0000;
		const BACKGROUND_COLOR:u32 = 0xFF202020;
		const TRACKING_PATH_MIN_DISTANCE:usize = 10;
		const TRACKING_PATH_INITIAL_VEC_SIZE:usize = 1000;
		const TRACKING_PATH_MIN_ENTRIES:usize = 20;
		const CURSOR_STILL_MAX_MOVEMENT:usize = 1;
		const CURSOR_STILL_TARGET_COUNT:usize = 20;

		// Create "target practive" window.
		let mut window:Window = Window::new("KeyFlow mouse recorder", window_size[0], window_size[1], WindowOptions::default())?;
		let mut source:[usize; 2] = [0; 2];
		let mut target:[usize; 2] = [0; 2];
		let mut last_cursor_location:[usize; 2] = [0; 2];
		let mut cursor_still_count:usize = 0;
		let mut current_path:Vec<[usize; 2]> = Vec::new();
		let mut paths:Vec<MouseProgressionPath> = Vec::new();

		// Keep updating window.
		let mut rng:ThreadRng = rand::rng();
		while window.is_open() && paths.len() < target_paths_count {

			// Track mouse.
			let cursor_position:(f32, f32) = window.get_mouse_pos(MouseMode::Clamp).unwrap();
			let cursor_position:[usize; 2] = [cursor_position.0 as usize, cursor_position.1 as usize];
			let is_on_target:bool = cursor_position[0] >= target[0] && cursor_position[0] < target[0] + TARGET_SIZE && cursor_position[1] >= target[1] && cursor_position[1] < target[1] + TARGET_SIZE;
			current_path.push(cursor_position);
			let mut move_target:bool = false;
			if is_on_target {

				// Check if cursor has been still for long enough.
				let cursor_offset:[usize; 2] = [cursor_position[0].max(last_cursor_location[0]) - cursor_position[0].min(last_cursor_location[0]), cursor_position[1].max(last_cursor_location[1]) - cursor_position[1].min(last_cursor_location[1])];
				if cursor_offset[0].max(cursor_offset[1]) <= CURSOR_STILL_MAX_MOVEMENT {
					cursor_still_count += 1;
					if cursor_still_count == CURSOR_STILL_TARGET_COUNT {
						
						// If path valid, convert coordinates to progression factors and store to results.
						move_target = true;
						let progression_path:MouseProgressionPath = MouseProgressionPath::new(current_path);
						if progression_path.path.len() > TRACKING_PATH_MIN_ENTRIES && source != [0; 2] {
							paths.push(progression_path);
						}
						current_path = Vec::with_capacity(TRACKING_PATH_INITIAL_VEC_SIZE);
						cursor_still_count = 0;
					}
				} else {
					cursor_still_count = 0;
				}
			}
			last_cursor_location = cursor_position;

			// Update target position.
			if source == target || move_target {
				source = cursor_position;
				while source[0].max(target[0]) - source[0].min(target[0]) < TRACKING_PATH_MIN_DISTANCE && source[1].max(target[1]) - source[1].min(target[1]) < TRACKING_PATH_MIN_DISTANCE {
					target = [rng.random_range(inner_size[0]..inner_size[0] + inner_size[2] - TARGET_SIZE), rng.random_range(inner_size[1]..inner_size[1] + inner_size[3] - TARGET_SIZE)];
				}
				let mut buffer:Vec<u32> = vec![BACKGROUND_COLOR; window_size[0] * window_size[1]];
				for y in target[1]..target[1] + TARGET_SIZE {
					let y_index:usize = y * window_size[0];
					for x in target[0]..target[0] + TARGET_SIZE {
						buffer[y_index + x] = TARGET_COLOR;
					}
				}
				window.update_with_buffer(&buffer, window_size[0], window_size[1])?;
			} else {
				window.update();
			}
			sleep(INTERVAL);
		}

		// Return recording.
		Ok(MouseRecording(paths))
	}

	/// Show a graph displaying the paths.
	pub fn show_graph(&self, window_size:[usize; 2]) -> Result<(), Box<dyn Error>> {

		// Window settings.
		const INTERVAL:Duration = Duration::from_millis(800);
		const GRAPH_COLOR_X:u32 = 0xFF660000;
		const GRAPH_COLOR_Y:u32 = 0xFF000066;
		const GRAPH_COLOR_ACTIVE_X:u32 = 0xFFFF0000;
		const GRAPH_COLOR_ACTIVE_Y:u32 = 0xFF0000FF;
		const AXIS_COLOR:u32 = 0xFF88FF00;
		const GRAPH_BACKGROUND_COLOR:u32 = 0xFF202020;
		const INNER_PADDING:f32 = 0.25;
		let inner_bounds:[usize; 4] = [
			0,
			(window_size[1] as f32 * INNER_PADDING) as usize,
			window_size[0],
			(window_size[1] as f32 * (1.0 - 2.0 * INNER_PADDING)) as usize
		];

		// Create buffer.
		let mut buffer:Vec<u32> = vec![GRAPH_BACKGROUND_COLOR; window_size[0] * window_size[1]];
		for x in inner_bounds[0]..inner_bounds[0] + inner_bounds[2] {
			for y in [inner_bounds[1], inner_bounds[1] + inner_bounds[3]] {
				buffer[y * window_size[0] + x] = AXIS_COLOR;
			}
		}
		for y in inner_bounds[1]..inner_bounds[1] + inner_bounds[3] {
			for x in [inner_bounds[0], inner_bounds[0] + inner_bounds[2]] {
				buffer[y * window_size[0] + x] = AXIS_COLOR;
			}
		}
		for path in &self.0 {
			path.draw_on_graph_buffer(&mut buffer, GRAPH_COLOR_X, GRAPH_COLOR_Y, &window_size, &inner_bounds);
		}

		// Create and show window.
		let mut window:Window = Window::new("KeyFlow mouse recorder", window_size[0], window_size[1], WindowOptions::default())?;
		let mut active_path_index:usize = 0;
		while window.is_active() {
			self.0[active_path_index].draw_on_graph_buffer(&mut buffer, GRAPH_COLOR_ACTIVE_X, GRAPH_COLOR_ACTIVE_Y, &window_size, &inner_bounds);
			window.update_with_buffer(&buffer, window_size[0], window_size[1])?;
			sleep(INTERVAL);
			self.0[active_path_index].draw_on_graph_buffer(&mut buffer, GRAPH_COLOR_X, GRAPH_COLOR_Y, &window_size, &inner_bounds);
			active_path_index = (active_path_index + 1) % self.0.len();
		}

		// Return success.
		Ok(())
	}
}



#[cfg(test)]
#[test]
fn test() {
	if std::env::var("RECORD_HUMANLIKE_MOUSE").map(|value| value == "1").unwrap_or(false) {
		let result:MouseRecording = MouseRecording::create([800, 600], 10).unwrap();
		result.show_graph([1200, 800]).unwrap();
	}
}