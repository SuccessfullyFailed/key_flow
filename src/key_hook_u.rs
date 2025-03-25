#[cfg(test)]
mod tests {
	use crate::key_hook;

	#[test]
	fn test_install() {
		key_hook::install();
	}
}
