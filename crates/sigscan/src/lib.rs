cfg_if::cfg_if! {
	if #[cfg(windows)] {
		mod windows;
		pub use windows::Scanner;
	} else {
		mod linux;
		pub use linux::Scanner;
	}
}

pub type Signature = &'static [Option<u8>];
pub type SignatureAndOffset = (usize, Signature);

pub fn find(scanner: &Scanner, &(offset, signature): &SignatureAndOffset) -> Option<*mut u8> {
	scanner.find(signature).map(|address| unsafe {
		let to_read = address.add(offset) as *const *mut u8;
		to_read.read_unaligned()
	})
}

/// offset points at the call opcode
pub fn find_call(scanner: &Scanner, &(offset, signature): &SignatureAndOffset) -> Option<*mut u8> {
	if signature.get(offset) != Some(&Some(0xE8)) || signature.len().checked_sub(offset)? < 5 {
		return None;
	}
	// SAFETY: full signature match includes the opcode and all four of the other
	// bytes
	scanner.find(signature).map(|address| unsafe {
		let call = address.add(offset);
		let displacement = call.add(1).cast::<i32>().read_unaligned();
		call.wrapping_add(5).wrapping_offset(displacement as isize)
	})
}
