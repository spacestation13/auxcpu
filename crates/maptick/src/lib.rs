#![allow(static_mut_refs)]
use auxcpu_impl::convert_signature;
use auxcpu_sigscan::{Scanner, SignatureAndOffset};
use cfg_if::cfg_if;
use retour::RawDetour;
use std::{cell::RefCell, time::Instant};

static mut SEND_MAPS_ORIGINAL: Option<extern "cdecl" fn()> = None;
static mut LAST_MAPTICK_DS: f32 = 0.0;

thread_local!(static SEND_MAPS_DETOUR: RefCell<Option<RawDetour>> = const { RefCell::new(None) });

cfg_if! {
	if #[cfg(windows)] {
		const BYONDCORE: &str = "byondcore.dll";
		const SEND_MAPS_SIGNATURE: SignatureAndOffset = (1, convert_signature!("E8 ?? ?? ?? ?? E8 ?? ?? ?? ?? 2B C3"));
	} else {
		const BYONDCORE: &str = "libbyond.so";
		const SEND_MAPS_SIGNATURE: SignatureAndOffset = (0, convert_signature!("55 89 E5 57 56 53 81 EC ?? ?? ?? ?? 65 A1"));
	}
}

extern "cdecl" fn send_maps_hook() {
	unsafe {
		let start = Instant::now();
		SEND_MAPS_ORIGINAL.unwrap_unchecked()();
		LAST_MAPTICK_DS = (start.elapsed().as_secs_f64() * 10.0) as f32; // convert to deciseconds
	}
}

pub fn init() -> Result<(), String> {
	if unsafe { SEND_MAPS_ORIGINAL.is_some() } {
		return Err("send_maps is already hooked!".to_owned());
	}
	let scanner = Scanner::for_module(BYONDCORE)
		.map_err(|error| format!("Failed to scan {BYONDCORE}: {error}"))?;
	/* eprintln!(
		"byondcore: {:?} -> {:?}",
		scanner.data_begin, scanner.data_end
	); */
	cfg_if! {
		if #[cfg(windows)] {
			let send_maps_ptr = auxcpu_sigscan::find_call(&scanner, &SEND_MAPS_SIGNATURE).ok_or("Failed to find send_maps")?;
		} else {
			let send_maps_ptr = auxcpu_sigscan::find(&scanner, &SEND_MAPS_SIGNATURE).ok_or("Failed to find send_maps")?;
		}
	};
	/* eprintln!("send_maps_ptr: {:?}", unsafe {
		send_maps_ptr.byte_sub(scanner.data_begin as usize)
	}); */
	unsafe {
		let send_maps_hook = RawDetour::new(send_maps_ptr as _, send_maps_hook as _)
			.map_err(|error| format!("Failed to setup send_maps hook: {error}"))?;
		send_maps_hook
			.enable()
			.map_err(|error| format!("Failed to enable send_maps hook: {error}"))?;
		SEND_MAPS_ORIGINAL = Some(std::mem::transmute::<&(), extern "cdecl" fn()>(
			send_maps_hook.trampoline(),
		));
		SEND_MAPS_DETOUR.with_borrow_mut(|detour| *detour = Some(send_maps_hook));
	};
	Ok(())
}

pub fn shutdown() {
	unsafe {
		SEND_MAPS_DETOUR.with_borrow_mut(|detour| {
			if let Some(detour) = detour.take() {
				detour
					.disable()
					.expect("failed to disable send_maps detour");
			}
		});
		SEND_MAPS_ORIGINAL = None;
		LAST_MAPTICK_DS = 0.0; // reset to default
	}
}

#[inline(always)]
pub fn last_maptick() -> f32 {
	unsafe { LAST_MAPTICK_DS }
}
