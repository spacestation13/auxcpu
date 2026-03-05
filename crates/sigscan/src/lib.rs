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
		/* eprintln!(
			"to_tread: {:?}",
			to_read.byte_sub(scanner.data_begin as usize)
		); */
		to_read.read_unaligned()
	})
}

pub fn find_call(scanner: &Scanner, &(_offset, signature): &SignatureAndOffset) -> Option<*mut u8> {
	scanner.find(signature).map(|address| unsafe {
		let offset = (address.offset(1) as *const isize).read_unaligned();
		address.offset(5).offset(offset)
		/* eprintln!(
			"offset: {:?}, call_addr: {:?}",
			offset,
			call_addr.byte_sub(scanner.data_begin as usize)
		); */
	})
}
