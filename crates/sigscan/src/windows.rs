use std::{
	mem,
	ops::{Deref, DerefMut},
};
use windows::{
	Win32::{
		Foundation::{FreeLibrary, HMODULE},
		System::{
			LibraryLoader::GetModuleHandleExW,
			ProcessStatus::{GetModuleInformation, MODULEINFO},
			Threading::GetCurrentProcess,
		},
	},
	core::{PCWSTR, Result as WindowsResult},
};

pub struct Scanner {
	module: ModuleHandle,
	pub data_begin: *mut u8,
	pub data_end: *mut u8,
}

impl Scanner {
	pub fn for_module(name: &str) -> WindowsResult<Self> {
		let mut module = ModuleHandle::default();
		let data_begin: *mut u8;
		let data_end: *mut u8;

		// Convert to UTF-16 and ensure null termination
		let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();

		unsafe {
			GetModuleHandleExW(0, PCWSTR(name_wide.as_ptr()), &mut *module)?;

			let mut module_info = mem::MaybeUninit::<MODULEINFO>::zeroed();
			GetModuleInformation(
				GetCurrentProcess(),
				*module,
				module_info.as_mut_ptr(),
				mem::size_of::<MODULEINFO>() as u32,
			)?;

			let module_info = module_info.assume_init();
			data_begin = module_info.lpBaseOfDll as *mut u8;
			data_end = data_begin.add(module_info.SizeOfImage as usize).sub(1);
		}

		Ok(Scanner {
			module,
			data_begin,
			data_end,
		})
	}

	pub fn find(&self, signature: &[Option<u8>]) -> Option<*mut u8> {
		let mut data_current = self.data_begin;
		let data_end = self.data_end;
		let mut signature_offset = 0;
		let mut result: Option<*mut u8> = None;

		unsafe {
			while data_current <= data_end {
				if signature[signature_offset].is_none()
					|| signature[signature_offset] == Some(*data_current)
				{
					if signature.len() <= signature_offset + 1 {
						if result.is_some() {
							// Found two matches.
							return None;
						}
						result = Some(data_current.offset(-(signature_offset as isize)));
						data_current = data_current.offset(-(signature_offset as isize));
						signature_offset = 0;
					} else {
						signature_offset += 1;
					}
				} else {
					data_current = data_current.offset(-(signature_offset as isize));
					signature_offset = 0;
				}

				data_current = data_current.offset(1);
			}
		}

		result
	}

	pub fn finish(self) -> WindowsResult<()> {
		self.module.finish()
	}
}

#[repr(transparent)]
#[derive(Default)]
struct ModuleHandle(HMODULE);

impl ModuleHandle {
	fn finish(self) -> WindowsResult<()> {
		let module = self.0;
		std::mem::forget(self);
		unsafe { FreeLibrary(module) }
	}
}

impl Deref for ModuleHandle {
	type Target = HMODULE;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for ModuleHandle {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl AsRef<HMODULE> for ModuleHandle {
	fn as_ref(&self) -> &HMODULE {
		&self.0
	}
}

impl AsMut<HMODULE> for ModuleHandle {
	fn as_mut(&mut self) -> &mut HMODULE {
		&mut self.0
	}
}

impl Drop for ModuleHandle {
	fn drop(&mut self) {
		if let Err(error) = unsafe { FreeLibrary(self.0) } {
			if cfg!(debug_assertions) {
				panic!("Failed to free module handle: {error}");
			} else {
				eprintln!("Failed to free module handle: {error}");
			}
		}
	}
}
