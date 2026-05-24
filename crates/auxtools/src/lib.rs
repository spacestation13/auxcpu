use auxtools::*;

#[init(full)]
fn find_signatures() -> Result<(), String> {
	auxcpu_core::find_signatures()?;
	Ok(())
}

/* don't use this for now
#[hook("/proc/reset_cpu_table")]
fn reset_cpu_table() {
	auxcpu_core::clear_cpu_table();
	Ok(Value::NULL)
}
*/

#[hook("/proc/current_true_cpu")]
fn current_true_cpu() {
	Ok(Value::from(auxcpu_core::read_cpu()))
}

#[hook("/proc/current_cpu_index")]
fn current_cpu_index() {
	Ok(Value::from(auxcpu_core::current_index() as u32))
}

#[hook("/proc/current_true_map_cpu")]
fn current_true_map_cpu() {
	match auxcpu_core::read_map_cpu() {
		Ok(value) => Ok(Value::from(value)),
		Err(_) => Ok(Value::NULL),
	}
}

#[hook("/proc/current_map_cpu_index")]
fn current_map_cpu_index() {
	match auxcpu_core::current_map_index() {
		Ok(index) => Ok(Value::from(index as u32)),
		Err(_) => Ok(Value::NULL),
	}
}

#[hook("/proc/true_cpu_at_index")]
fn true_cpu_at_index(index: Value) {
	let index = index.as_number()? as usize;
	auxcpu_core::read_cpu_at_index(index)
		.map(Value::from)
		.map_err(|error| runtime!("{}", error))
}

#[hook("/proc/true_map_cpu_at_index")]
fn true_map_cpu_at_index(index: Value) {
	let index = index.as_number()? as usize;
	match auxcpu_core::read_map_cpu_at_index(index) {
		Ok(value) => Ok(Value::from(value)),
		Err(_) => Ok(Value::NULL),
	}
}

#[hook("/proc/cpu_values")]
fn cpu_values() {
	let list = List::with_size(16);
	for (index, value) in auxcpu_core::cpu_table().into_iter().enumerate() {
		list.set(index as u32 + 1, value)?;
	}
	Ok(list.into())
}

#[hook("/proc/map_cpu_values")]
fn map_cpu_values() {
	let list = List::with_size(16);
	for (index, value) in auxcpu_core::map_cpu_table().into_iter().enumerate() {
		list.set(index as u32 + 1, value)?;
	}
	Ok(list.into())
}

#[hook("/proc/map_cpu_signatures_found")]
fn map_cpu_signatures_found() {
	Ok(Value::from(auxcpu_core::map_cpu_signatures_found()))
}

#[cfg(feature = "maptick")]
#[hook("/proc/maptick_init")]
pub fn maptick_init() {
	match auxcpu_maptick::init() {
		Ok(_) => Ok(Value::from_string("ok").unwrap()),
		Err(error) => Err(runtime!("{}", error)),
	}
}

#[cfg(feature = "maptick")]
#[hook("/proc/maptick_shutdown")]
pub fn maptick_shutdown() {
	auxcpu_maptick::shutdown();
	Ok(Value::NULL)
}

#[cfg(feature = "maptick")]
#[hook("/proc/maptick")]
pub fn maptick() {
	Ok(Value::from(auxcpu_maptick::last_maptick()))
}
