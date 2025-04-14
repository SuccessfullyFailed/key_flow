use std::error::Error;

use file_ref::FileRef;



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
			path: mouse_path[start_index..end_index].into_iter().map(|position| [(position[0] as f32 - start_f32[0]) / (end_f32[0] - start_f32[0]), (position[1] as f32 - start_f32[1]) / (end_f32[1] - start_f32[1])]).collect()
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
	pub fn to_bytes(&self) -> Vec<u8> {
		vec![
			self.start.iter().map(|value| (*value as u64).to_be_bytes()).flatten().collect::<Vec<u8>>(),
			self.end.iter().map(|value| (*value as u64).to_be_bytes()).flatten().collect::<Vec<u8>>(),
			self.path.iter().map(|coordinate| coordinate.iter().map(|value| value.to_be_bytes()).flatten().collect::<Vec<u8>>()).flatten().collect::<Vec<u8>>()
		].into_iter().flatten().collect()
	}

	/// Create from bytes.
	pub fn from_bytes(bytes:&[u8]) -> Result<MouseProgressionPath, Box<dyn Error>> {
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
	pub fn from_file(path:&str) -> Result<MouseProgressionPath, Box<dyn Error>> {
		MouseProgressionPath::from_bytes(&FileRef::new(path).read_bytes()?)
	}

	/// Store to a file.
	pub fn to_file(&self, path:&str) -> Result<(), Box<dyn Error>> {
		FileRef::new(path).write_bytes(&self.to_bytes())
	}
}