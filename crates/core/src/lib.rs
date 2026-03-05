use auxcpu_impl::convert_signature;
use auxcpu_sigscan::{Scanner, SignatureAndOffset, find};
use cfg_if::cfg_if;

static mut CPU_VALUE_TABLE: *mut [f32; 16] = std::ptr::null_mut();
static mut CPU_INDEX: *mut u8 = std::ptr::null_mut();

/// Returns the current CPU index.
pub fn current_index() -> usize {
	unsafe { (CPU_INDEX.read().wrapping_sub(1) & 0xF) as usize }
}

/// Returns the CPU value of the current index.
pub fn read_cpu() -> f32 {
	unsafe { *(*CPU_VALUE_TABLE).get_unchecked(current_index()) }
}

/// Reads the CPU value at the given index.
/// Index must be between 0 and 15.
pub fn read_cpu_at_index(index: usize) -> Result<f32, String> {
	unsafe { *CPU_VALUE_TABLE }
		.get(index)
		.copied()
		.ok_or_else(|| format!("CPU index must be 0-15 (got {})", index))
}

/* don't use this for now
/// Clears the CPU table, setting all values to 0, and setting the index to 0.
pub fn clear_cpu_table() {
	unsafe {
		CPU_VALUE_TABLE.write([0.0; 16]);
		CPU_INDEX.write(0);
	}
}
*/

/// Returns the raw CPU table.
/// If CPU_VALUE_TABLE, it will just return `[0.0; 16]`
pub fn cpu_table() -> [f32; 16] {
	unsafe {
		if CPU_VALUE_TABLE.is_null() {
			[0.0; 16]
		} else {
			*CPU_VALUE_TABLE
		}
	}
}

cfg_if! {
	if #[cfg(windows)] {
		const BYONDCORE: &str = "byondcore.dll";
		const CPU_VALUE_TABLE_SIGNATURE: SignatureAndOffset = (5, convert_signature!("F3 0F 11 04 85 ?? ?? ?? ?? 33 C0"));
		const CPU_INDEX_SIGNATURE: SignatureAndOffset = (2, convert_signature!("88 0D ?? ?? ?? ?? F2 0F 5E C8 66 0F 5A C1"));
	} else {
		const BYONDCORE: &str = "libbyond.so";
		const CPU_VALUE_TABLE_SIGNATURE: SignatureAndOffset = (3, convert_signature!("D8 24 8D"));
		const CPU_INDEX_SIGNATURE: SignatureAndOffset = (1, convert_signature!("A2 ?? ?? ?? ?? D9 1C 24"));
	}
}

pub fn find_signatures() -> Result<(), String> {
	let scanner = Scanner::for_module(BYONDCORE)
		.map_err(|error| format!("Failed to scan {BYONDCORE}: {error}"))?;
	let cpu_value_table_ptr =
		find(&scanner, &CPU_VALUE_TABLE_SIGNATURE).ok_or("Failed to find CPU_VALUE_TABLE")?;
	let cpu_index_ptr = find(&scanner, &CPU_INDEX_SIGNATURE).ok_or("Failed to find CPU_INDEX")?;
	unsafe {
		CPU_VALUE_TABLE = cpu_value_table_ptr as _;
		CPU_INDEX = cpu_index_ptr as _;
	}
	Ok(())
}
