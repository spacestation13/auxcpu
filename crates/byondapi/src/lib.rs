use meowtonin::byond_fn;

#[byond_fn]
pub fn find_signatures() -> bool {
	auxcpu_core::find_signatures().expect("failed to find cpu value signatures");
	true
}

/* don't use this for now
#[byond_fn]
pub fn reset_cpu_table() {
	auxcpu_core::clear_cpu_table();
}
*/

#[byond_fn]
pub fn current_true_cpu() -> f32 {
	auxcpu_core::read_cpu()
}

#[byond_fn]
pub fn current_cpu_index() -> u8 {
	auxcpu_core::current_index() as u8
}

#[byond_fn]
pub fn current_true_map_cpu() -> Option<f32> {
	auxcpu_core::read_map_cpu().ok()
}

#[byond_fn]
pub fn current_map_cpu_index() -> Option<u8> {
	auxcpu_core::current_map_index()
		.map(|index| index as u8)
		.ok()
}

#[byond_fn]
pub fn true_cpu_at_index(index: usize) -> f32 {
	auxcpu_core::read_cpu_at_index(index).expect("failed to read cpu")
}

#[byond_fn]
pub fn true_map_cpu_at_index(index: usize) -> Option<f32> {
	auxcpu_core::read_map_cpu_at_index(index).ok()
}

#[byond_fn]
pub fn cpu_values() -> [f32; 16] {
	auxcpu_core::cpu_table()
}

#[byond_fn]
pub fn map_cpu_values() -> [f32; 16] {
	auxcpu_core::map_cpu_table()
}

#[byond_fn]
pub fn map_cpu_signatures_found() -> bool {
	auxcpu_core::map_cpu_signatures_found()
}

#[cfg(feature = "maptick")]
#[byond_fn]
pub fn maptick_init() -> Option<String> {
	match auxcpu_maptick::init() {
		Ok(_) => None,
		Err(error) => Some(error),
	}
}

#[cfg(feature = "maptick")]
#[byond_fn]
pub fn maptick_shutdown() {
	auxcpu_maptick::shutdown();
}

#[cfg(feature = "maptick")]
#[byond_fn]
pub fn maptick() -> f32 {
	auxcpu_maptick::last_maptick()
}
