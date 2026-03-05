// uncomment the define below if you want support for "true" world.map_cpu (true_maptick)
//#define MAPTICK_HOOK
#define AUXCPU_DLL (world.GetConfig("env", "AUXCPU_DLL") || (world.system_type == MS_WINDOWS ? "../target/i686-pc-windows-msvc/release/auxcpu_auxtools.dll" : "../target/i686-pc-windows-msvc/release/libauxcpu_auxtools.so"))

/proc/current_true_cpu()
	CRASH()

/proc/current_cpu_index()
	CRASH()

/proc/true_cpu_at_index(index)
	CRASH()

/proc/cpu_values()
	CRASH()

#ifdef MAPTICK_HOOK
/proc/maptick_init()
	CRASH()

/proc/maptick_shutdown()
	CRASH()

/proc/maptick()
	CRASH()
#endif

/* don't use this for now
/proc/reset_cpu_table()
	CRASH()
*/

var/static/did_auxtools_init = FALSE

/proc/setup()
	var/init_result = call_ext(AUXCPU_DLL, "auxtools_init")()
	if(init_result != "SUCCESS")
		world.log << "auxtools failed to init: [init_result]"
		return FALSE
	world.log << "auxcpu initialized"
#ifdef MAPTICK_HOOK
	if(maptick_init() != "ok")
		world.log << "auxcpu failed to hook maptick"
		return FALSE
	world.log << "auxcpu hooked maptick"
#endif
	return TRUE

/proc/cleanup()
	if(did_auxtools_init)
#ifdef MAPTICK_HOOK
		maptick_shutdown()
#endif
		call_ext(AUXCPU_DLL, "auxtools_shutdown")()
		did_auxtools_init = FALSE
