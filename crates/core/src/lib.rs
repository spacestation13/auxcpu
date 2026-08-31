use auxcpu_impl::convert_signature;
use auxcpu_sigscan::{Scanner, SignatureAndOffset, find};
use cfg_if::cfg_if;

static mut CPU_VALUE_TABLE: *mut [f32; 16] = std::ptr::null_mut();
static mut CPU_INDEX: *mut u8 = std::ptr::null_mut();

static mut MAP_CPU_VALUE_TABLE: *mut [f32; 16] = std::ptr::null_mut();
static mut MAP_CPU_INDEX: *mut u8 = std::ptr::null_mut();

/// Returns the current CPU index.
pub fn current_index() -> usize {
	unsafe { (CPU_INDEX.read().wrapping_sub(1) & 0xF) as usize }
}

pub fn current_map_index() -> Result<usize, String> {
	if !map_cpu_signatures_found() {
		return Err("MAP_CPU signatures have not been found".to_owned());
	}
	Ok(unsafe { (MAP_CPU_INDEX.read().wrapping_sub(1) & 0xF) as usize })
}

/// Returns the CPU value of the current index.
pub fn read_cpu() -> f32 {
	unsafe { *(*CPU_VALUE_TABLE).get_unchecked(current_index()) }
}

/// Returns the map-send CPU value of the current index.
pub fn read_map_cpu() -> Result<f32, String> {
	read_map_cpu_at_index(current_index())
}

/// Reads the CPU value at the given index.
/// Index must be between 0 and 15.
pub fn read_cpu_at_index(index: usize) -> Result<f32, String> {
	unsafe { *CPU_VALUE_TABLE }
		.get(index)
		.copied()
		.ok_or_else(|| format!("CPU index must be 0-15 (got {})", index))
}

/// Reads the map-send CPU value at the given index.
/// Index must be between 0 and 15.
pub fn read_map_cpu_at_index(index: usize) -> Result<f32, String> {
	if !map_cpu_signatures_found() {
		return Err("MAP_CPU signatures have not been found".to_owned());
	}

	let cpu = read_cpu_at_index(index)?;
	let pre_map_cpu = unsafe { *MAP_CPU_VALUE_TABLE }
		.get(index)
		.copied()
		.ok_or_else(|| format!("map CPU index must be 0-15 (got {})", index))?;
	Ok(cpu - pre_map_cpu)
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

pub fn map_cpu_table() -> [f32; 16] {
	if !map_cpu_signatures_found() {
		return [0.0; 16];
	}

	let mut values = [0.0; 16];
	let cpu_values = cpu_table();
	let pre_map_values = unsafe { *MAP_CPU_VALUE_TABLE };
	for (index, value) in values.iter_mut().enumerate() {
		*value = cpu_values[index] - pre_map_values[index];
	}
	values
}

pub fn map_cpu_signatures_found() -> bool {
	unsafe { !MAP_CPU_VALUE_TABLE.is_null() && !MAP_CPU_INDEX.is_null() }
}

cfg_if! {
	if #[cfg(windows)] {
		const BYONDCORE: &str = "byondcore.dll";
		const CPU_VALUE_TABLE_SIGNATURE: SignatureAndOffset = (5, convert_signature!("F3 0F 5C 0C 85 ?? ?? ?? ?? F3 0F 11 04 85 ?? ?? ?? ??"));
		const CPU_VALUE_TABLE_WRITE_OFFSET: usize = 14;
		const CPU_INDEX_SIGNATURE: SignatureAndOffset = (7, convert_signature!("FE C1 80 E1 0F 88 0D ?? ?? ?? ??"));
		const MAP_CPU_VALUE_TABLE_SIGNATURE: SignatureAndOffset = (5, convert_signature!("F3 0F 11 04 85 ?? ?? ?? ?? 1A C0 22 C1 A2 ?? ?? ?? ??"));
		const MAP_CPU_INDEX_SIGNATURE: SignatureAndOffset = (14, MAP_CPU_VALUE_TABLE_SIGNATURE.1);
	} else {
		const BYONDCORE: &str = "libbyond.so";
		const CPU_VALUE_TABLE_SIGNATURE: SignatureAndOffset = (3, convert_signature!("D8 24 8D ?? ?? ?? ?? D8 C1 D9 15 ?? ?? ?? ?? D9 C9 D9 1C 8D ?? ?? ?? ??"));
		const CPU_VALUE_TABLE_WRITE_OFFSET: usize = 20;
		const CPU_INDEX_SIGNATURE: SignatureAndOffset = (9, convert_signature!("83 C0 01 83 E0 0F DE F9 A2 ?? ?? ?? ??"));
		const MAP_CPU_VALUE_TABLE_SIGNATURE: SignatureAndOffset = (3, convert_signature!("D9 1C 95 ?? ?? ?? ?? D8 0D ?? ?? ?? ?? 31 D2 3C 10 0F 43 C2 A2 ?? ?? ?? ??"));
		const MAP_CPU_INDEX_SIGNATURE: SignatureAndOffset = (21, MAP_CPU_VALUE_TABLE_SIGNATURE.1);
	}
}

pub fn find_signatures() -> Result<(), String> {
	let scanner = Scanner::for_module(BYONDCORE)
		.map_err(|error| format!("Failed to scan {BYONDCORE}: {error}"))?;
	let cpu_value_table_ptr =
		find(&scanner, &CPU_VALUE_TABLE_SIGNATURE).ok_or("Failed to find CPU_VALUE_TABLE")?;
	// gotta make sure these agree
	if find(
		&scanner,
		&(CPU_VALUE_TABLE_WRITE_OFFSET, CPU_VALUE_TABLE_SIGNATURE.1),
	) != Some(cpu_value_table_ptr)
	{
		return Err(
			"CPU_VALUE_TABLE read and write addresses disagree, something's fucked.".to_owned(),
		);
	}
	let cpu_index_ptr = find(&scanner, &CPU_INDEX_SIGNATURE).ok_or("Failed to find CPU_INDEX")?;

	let map_cpu_value_table_ptr = find(&scanner, &MAP_CPU_VALUE_TABLE_SIGNATURE)
		.and_then(|table| find(&scanner, &MAP_CPU_INDEX_SIGNATURE).map(|index| (table, index)));
	unsafe {
		CPU_VALUE_TABLE = cpu_value_table_ptr as _;
		CPU_INDEX = cpu_index_ptr as _;
		if let Some((map_cpu_value_table_ptr, map_cpu_index_ptr)) = map_cpu_value_table_ptr {
			MAP_CPU_VALUE_TABLE = map_cpu_value_table_ptr as _;
			MAP_CPU_INDEX = map_cpu_index_ptr as _;
		} else {
			MAP_CPU_VALUE_TABLE = std::ptr::null_mut();
			MAP_CPU_INDEX = std::ptr::null_mut();
		}
	}
	Ok(())
}
