// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Manages the procdump configuration
//
//--------------------------------------------------------------------
use std::env;

//
// Constants used
//
const DEFAULT_POLLING_INTERVAL: u64 = 1000;     // ms
const DEFAULT_DELTA_TIME: u32 = 10;             // secs
const DEFAULT_NUMBER_OF_DUMPS: u32 = 1;

//
// Struct used to communicate the current procdump configuration
//
pub struct ProcDumpConfiguration{
    pub process_id: i32,
    pub process_pgid: i32,
    pub polling_frequency: u64,
    pub number_of_dumps_to_collect: u32,
    pub number_of_dumps_collected: u32,
    pub threshold_seconds: u32,
    pub is_process_group_set : bool,
    pub trigger_threshold_cpu : u32,
    pub trigger_threshold_cpu_below : bool,
    pub trigger_threshold_mem : u32,
    pub trigger_threshold_mem_below : bool,
    pub trigger_threshold_threads : u32,
    pub trigger_threshold_file_descriptors : i32,
    pub trigger_signal : u32,
    pub trigger_threshold_seconds : u32,
    pub waiting_process_name : bool,
    pub diagnostics_logging_enabled : bool,
    pub gcore_process_id : i32,
    pub core_dump_path : String,
    pub core_dump_name : String,
    pub exit_monitor : bool,
}

//--------------------------------------------------------------------
//
// ApplyDefaults - Apply default values to configuration
//
//--------------------------------------------------------------------
pub fn ApplyDefaults(config:&mut ProcDumpConfiguration)
{
    if config.number_of_dumps_to_collect == u32::MAX
    {
        config.number_of_dumps_to_collect = DEFAULT_NUMBER_OF_DUMPS;
    }

    if config.threshold_seconds  == u32::MAX
    {
        config.threshold_seconds = DEFAULT_DELTA_TIME;
    }

    if config.polling_frequency == u64::MAX
    {
        config.polling_frequency = DEFAULT_POLLING_INTERVAL;
    }
}

// -----------------------------------------------------------------
// Default values trait for ProcDumpConfiguration
// -----------------------------------------------------------------
impl Default for ProcDumpConfiguration
{
    fn default() -> ProcDumpConfiguration
    {
        //MAXIMUM_CPU = 100 * (int)sysconf(_SC_NPROCESSORS_ONLN);
        //HZ = sysconf(_SC_CLK_TCK);
        //sysinfo(&(self->SystemInfo));
        ProcDumpConfiguration
        {
            process_id: -1,
            is_process_group_set: false,
            process_pgid: -1,
            number_of_dumps_collected: 0,
            number_of_dumps_to_collect: u32::MAX,
            trigger_threshold_cpu: u32::MAX,
            trigger_threshold_cpu_below: false,
            trigger_threshold_mem: u32::MAX,
            trigger_threshold_mem_below: false,
            trigger_threshold_threads: u32::MAX,
            trigger_threshold_file_descriptors: -1,
            trigger_signal: u32::MAX,
            trigger_threshold_seconds: u32::MAX,
            waiting_process_name: false,
            diagnostics_logging_enabled: false,
            gcore_process_id: -1,
            polling_frequency: u64::MAX,
            core_dump_path: Default::default(),
            core_dump_name: Default::default(),
            threshold_seconds: u32::MAX,
            exit_monitor: false,
        }
    }
}

// -----------------------------------------------------------------
// init_procdump - Initialize ProcDump
// -----------------------------------------------------------------
pub fn init_procdump() -> i32
{
    // Open logger
    // Check kernel version


    0
}


// -----------------------------------------------------------------
// print_banner - Prints the procdump banner
// -----------------------------------------------------------------
pub fn print_banner(){
    println!("\nRustyProcDump v0.000000001 - Sysinternals process dump utility");
    println!();

    println!("Monitors a process and writes a core dump file when the process exceeds the");

    println!("specified criteria.");
    println!();
 }

// -----------------------------------------------------------------
// print_usage - Prints the procdump usage
// -----------------------------------------------------------------
pub fn print_usage(){
    println!("Capture Usage:");
    println!("   procdump [mc|-ml CPU_Usage] PID");
    println!();
}

// -----------------------------------------------------------------
// get_options - Parses command line and populates the config
// -----------------------------------------------------------------
pub fn get_options(config: &mut ProcDumpConfiguration) -> i32 {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2
    {
        print_banner();
        return -1;
    }

    let mut _i = 0;

    while _i < args.len(){

        if _i==0 {
            _i+=1;
            continue;
        }

        if args[_i].eq("/h") || args[_i].eq("-h") {
            print_usage();
            return -1;
        }
        else if args[_i].eq("/m") || args[_i].eq("-m") || args[_i].eq("/ml") || args[_i].eq("-ml") {
            if args.get(_i+1)==None {
                print_usage();
                return -1;
            }

            let s = args.get(_i+1).unwrap();

            config.trigger_threshold_mem = s.parse::<u32>().unwrap();
            if args[_i].eq("/ml") || args[_i].eq("-ml"){
                config.trigger_threshold_mem_below = true;
            }

            _i+=1;
        }
        else if args[_i].eq("/pf") || args[_i].eq("-pf") || args[_i].eq("/pf") || args[_i].eq("-pf") {
            if args.get(_i+1)==None {
                print_usage();
                return -1;
            }

            let s = args.get(_i+1).unwrap();

            config.polling_frequency = s.parse::<u64>().unwrap();

            _i+=1;
        }
        else {
            config.process_id = args[_i].parse::<i32>().unwrap();
        }

        _i+=1;
    }

    0
}


// -----------------------------------------------------------------
// print_triggers - Prints active triggers
// -----------------------------------------------------------------
pub fn print_triggers(config: &ProcDumpConfiguration){
    println!("Mem trigger value: {}MB", config.trigger_threshold_mem);
    println!("Mem trigger below: {}", config.trigger_threshold_mem_below);
    println!("PID: {}", config.process_id);
}