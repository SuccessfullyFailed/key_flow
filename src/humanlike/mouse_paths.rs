use super::mouse_recorder::MouseRecording;
use rand::{rngs::ThreadRng, seq::IndexedRandom, Rng};
use std::error::Error;
use file_ref::FileRef;
use cachew::cache;



pub(super) const CURSOR_RECORDINGS_DIR_ENV_VAR:&str = "KEY_FLOW_HUMAN_LIKE_MOUSE_PATHS_DIR";
pub(super) const RECORD_CURSOR_ARG:&str = "RECORD_HUMANLIKE_MOUSE";
pub(super) const RECORD_CURSOR_ARG_ACCEPTANCE_VALUE:&str = "1";
const PLACEHOLDER_PATH:&[[usize; 2]]  = &[[0, 0], [50, 50], [100, 100]];



/// Create a path for a mouse displacement using a random progression path curve. The returned values are relative displacement, not absolute coordinates.
pub(crate) fn create_displacement_path(displacement:[i32; 2], displacement_randomness:[i32; 2], interval_ms:u64, duration_ms:u64, duration_randomness_ms:u64) -> Vec<[i32; 2]> {

	// Randomize arguments.
	let rng:&mut ThreadRng = cache!(ThreadRng, rand::rng());
	let displacement_random_amount:[i32; 2] = displacement_randomness.map(|value| if value == 0 { 0 } else { rng.random_range(-value..value)});
	let displacement:[i32; 2] = [displacement[0] + displacement_random_amount[0], displacement[1] + displacement_random_amount[1]];
	let duration_ms:u64 = (duration_ms as i64 + if duration_randomness_ms == 0 { 0 } else { rng.random_range(0..2 * duration_randomness_ms as i64) - duration_randomness_ms as i64 / 2 }) as u64;
	let displacement_f32:[f32; 2] = [displacement[0] as f32, displacement[1] as f32];
	
	// Pick and parse random base path.
	let base_path:&MouseProgressionPath = random_progression_path();
	let base_path_len:usize = base_path.path.len();
	let max_left_index:usize = base_path_len - 2;

	// Create mouse movement curve.
	let cursor_incrementations:f32 = duration_ms as f32 / interval_ms as f32;
	let cursor_incrementation:f32 = base_path_len as f32 / cursor_incrementations;
	let cursor_max:f32 = base_path_len as f32 - 1.0;
	let mut cursor:f32 = cursor_incrementation;
	let mut current_coord:[i32; 2] = [0, 0];
	let mut path:Vec<[i32; 2]> = Vec::with_capacity(cursor_incrementations.ceil() as usize);
	while cursor < cursor_max {
		let left_index:usize = (cursor as usize).min(max_left_index);
		let left:[f32; 2] = base_path.path[left_index];
		let right:[f32; 2] = base_path.path[left_index + 1];
		let factor:f32 = cursor % 1.0;

		// Create randomized displacement based on progress through the curve.
		let curve_coord_factor:[f32; 2] = [left[0] + (right[0] - left[0]) * factor, left[1] + (right[1] - left[1]) * factor];
		let target_coord:[i32; 2] = [(curve_coord_factor[0] * displacement_f32[0]) as i32, (curve_coord_factor[1] * displacement_f32[1]) as i32];
		let step_displacement:[i32; 2] = [target_coord[0] - current_coord[0], target_coord[1] - current_coord[1]];
		path.push(step_displacement);
		current_coord = target_coord;

		cursor += cursor_incrementation;
	}
	path.push([displacement[0] - current_coord[0], displacement[1] - current_coord[1]]);

	// Return curve.
	path
}

/// Get a random mouse progression path.
fn random_progression_path() -> &'static MouseProgressionPath {
	let available_paths:&mut Vec<MouseProgressionPath> = cache!(Vec<MouseProgressionPath>, match load_progression_paths() {
		Ok(paths) => paths,
		Err(error) => {
			eprintln!("Error loading KeyFlow mouse progression paths: {error}");
			vec![MouseProgressionPath::new(PLACEHOLDER_PATH.to_vec())]
		}
	});
	available_paths.choose(cache!(ThreadRng, rand::rng())).unwrap()
}

/// Load all progression paths available in dedicated dir.
fn load_progression_paths() -> Result<Vec<MouseProgressionPath>, Box<dyn Error>> {
	const DEFAULT_CURSOR_RECORDS_DIR:FileRef = FileRef::new_const("./target/key_flow/humanlike/mouse_paths");

	// Read files.
	let exported_paths_dir:FileRef = std::env::var(CURSOR_RECORDINGS_DIR_ENV_VAR).map(|path| FileRef::new(&path)).unwrap_or(DEFAULT_CURSOR_RECORDS_DIR);
	let mut exported_paths_files:Vec<FileRef> = exported_paths_dir.list_files();

	// If dir doesn't exist or has no files, ask user to create a recording.
	if !exported_paths_dir.exists() || exported_paths_files.is_empty() {
		eprintln!("KeyFlow humanlike could not find mouse path records. Please run a keyflow mouse movement function with environment variable {} set to {}. This will start a target practice minigame that records your mouse movement to replicate personalized mouse movement.", RECORD_CURSOR_ARG, RECORD_CURSOR_ARG_ACCEPTANCE_VALUE);
		if std::env::var(RECORD_CURSOR_ARG).map(|value| value == RECORD_CURSOR_ARG_ACCEPTANCE_VALUE).unwrap_or(false) {
			let recording:MouseRecording = MouseRecording::create([800, 600], 100)?;
			recording.show_graph([800, 600])?;
			recording.save_to(&exported_paths_dir)?;
			exported_paths_files = exported_paths_dir.list_files();
		}
	}
	
	// Create paths.
	let mut paths:Vec<MouseProgressionPath> = exported_paths_files.into_iter().map(|file| MouseProgressionPath::from_file(file.path())).flatten().collect();
	if paths.is_empty() {
		eprintln!("Could not find any mouse path records. Using default placeholder paths for now.");
		paths = vec![MouseProgressionPath::new(PLACEHOLDER_PATH.to_vec())];
	}
	Ok(paths)
}



#[derive(Clone, PartialEq, Debug)]
pub struct MouseProgressionPath {
	pub(crate) start:[usize; 2],
	pub(crate) end:[usize; 2],
	pub(crate) path:Vec<[f32; 2]>
}
impl MouseProgressionPath {

	/* CONSTRUCTOR METHODS */

	/// Create a new path from raw mouse positions.
	pub fn new(mouse_path:Vec<[usize; 2]>) -> MouseProgressionPath {
		const MINIMUM_MOVEMENT:usize = 2;
		const HAS_MINIMUM_MOVEMENT:fn(&[usize; 2], &[usize; 2]) -> bool = |left, right| ((left[0].max(right[0]) - left[0].min(right[0])) + (left[1].max(right[1]) - left[1].min(right[1]))) >= MINIMUM_MOVEMENT;

		let start_index:usize = mouse_path.iter().skip(1).enumerate().position(|(previous_index, current)| HAS_MINIMUM_MOVEMENT(&mouse_path[previous_index], current)).unwrap_or_default();
		let end_index:usize = mouse_path.len() - 1 - mouse_path.iter().rev().skip(1).enumerate().position(|(previous_index, current)| HAS_MINIMUM_MOVEMENT(&mouse_path[previous_index], current)).unwrap_or_default();
		let end_index:usize = end_index.max(start_index);
		let start_f32:[f32; 2] = [mouse_path[start_index][0] as f32, mouse_path[start_index][1] as f32];
		let end_f32:[f32; 2] = [mouse_path[end_index][0] as f32, mouse_path[end_index][1] as f32];
		MouseProgressionPath {
			start: mouse_path[start_index],
			end: mouse_path[end_index],
			path: mouse_path[start_index..end_index + 1].into_iter().map(|position| [(position[0] as f32 - start_f32[0]) / (end_f32[0] - start_f32[0]), (position[1] as f32 - start_f32[1]) / (end_f32[1] - start_f32[1])]).collect()
		}
	}



	/* USAGE METHODS */

	/// Draw self on the buffer of a graph.
	pub(crate) fn draw_on_graph_buffer(&self, buffer:&mut [u32], color_x:u32, color_y:u32, window_size:&[usize; 2], inner_bounds:&[usize; 4]) {

		// Function to get the coordinates of all pixels in a line.
		const COORDS_IN_LINE:fn([usize; 2], [usize; 2]) -> Vec<[usize; 2]> = |start, end| {
			let start:[isize; 2] = [start[0] as isize, start[1] as isize];
			let end:[isize; 2] = [end[0] as isize, end[1] as isize];
			let difference:[isize; 2] = [(end[0] - start[0]).abs(), -(end[1] - start[1]).abs()];
			let mirror:[isize; 2] = [if start[0] < end[0] { 1 } else { -1 }, if start[1] < end[1] { 1 } else { -1 }];

			let mut coords:Vec<[isize; 2]> = vec![start];
			let mut cursor:[isize; 2] = start;
			let mut cursor_off_from_ideal:isize = difference[0] + difference[1];
			while cursor != end {
				let double_off_from_ideal:isize = 2 * cursor_off_from_ideal;
				if double_off_from_ideal >= difference[1] { cursor_off_from_ideal += difference[1]; cursor[0] += mirror[0]; }
				if double_off_from_ideal <= difference[0] { cursor_off_from_ideal += difference[0]; cursor[1] += mirror[1]; }
				coords.push(cursor);
			}
			coords.into_iter().map(|coord| [coord[0] as usize, coord[1] as usize]).collect()
		};

		// Draw onto buffer.
		let path_len_f32:f32 = self.path.len() as f32;
		let mut last_position:[[usize; 2]; 2] = [[inner_bounds[0], inner_bounds[1] + inner_bounds[3]]; 2];
		for (index, position) in self.path.iter().enumerate() {
			let x:usize = (index as f32 / path_len_f32 * inner_bounds[2] as f32) as usize;
			for axis in 0..2 {
				let y:usize = (inner_bounds[1] as f32 + inner_bounds[3] as f32 * position[axis]).max(0.0).min(window_size[1] as f32 - 1.0) as usize;
				for coord in COORDS_IN_LINE(last_position[axis], [x, y]) {
					buffer[coord[1] * window_size[0] + coord[0]] = if axis == 0 { color_x } else { color_y };
				}
				last_position[axis] = [x, y];
			}
		}
	}



	/* STORING AND LOADING METHODS */

	/// Convert to bytes.
	pub(super) fn to_bytes(&self) -> Vec<u8> {
		vec![
			self.start.iter().map(|value| (*value as u64).to_be_bytes()).flatten().collect::<Vec<u8>>(),
			self.end.iter().map(|value| (*value as u64).to_be_bytes()).flatten().collect::<Vec<u8>>(),
			self.path.iter().map(|coordinate| coordinate.iter().map(|value| value.to_be_bytes()).flatten().collect::<Vec<u8>>()).flatten().collect::<Vec<u8>>()
		].into_iter().flatten().collect()
	}

	/// Create from bytes.
	pub(super) fn from_bytes(bytes:&[u8]) -> Result<MouseProgressionPath, Box<dyn Error>> {
		const MIN_BYTES:usize = 8 * 4;

		let bytes_len:usize = bytes.len();
		if bytes_len < MIN_BYTES {
			return Err(format!("Could not create MouseProgressionPath from {} bytes. Bytes are required to be at least {} bytes.", bytes_len, MIN_BYTES).into());
		}

		let start:[usize; 2] = [u64::from_be_bytes(bytes[0..8].try_into().unwrap()) as usize, u64::from_be_bytes(bytes[8..16].try_into().unwrap()) as usize];
		let end:[usize; 2] = [u64::from_be_bytes(bytes[16..24].try_into().unwrap()) as usize, u64::from_be_bytes(bytes[24..32].try_into().unwrap()) as usize];
		let mut coords:Vec<[f32; 2]> = Vec::with_capacity((bytes_len - MIN_BYTES) / 8);
		for set_bytes in bytes[MIN_BYTES..].chunks(8) {
			coords.push([
				f32::from_be_bytes(set_bytes[..4].try_into()?),
				f32::from_be_bytes(set_bytes[4..].try_into()?)
			]);
		}

		Ok(MouseProgressionPath {
			start,
			end,
			path: coords
		})
	}

	/// Load from a file.
	pub(super) fn from_file(path:&str) -> Result<MouseProgressionPath, Box<dyn Error>> {
		MouseProgressionPath::from_bytes(&FileRef::new(path).read_bytes()?)
	}

	/// Store to a file.
	pub(super) fn save_to_file(&self, path:&str) -> Result<(), Box<dyn Error>> {
		FileRef::new(path).write_bytes(&self.to_bytes())
	}
}