// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Manages the procdump configuration
//
//--------------------------------------------------------------------
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::io::Result;

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
    pub trigger_threshold_file_descriptors : u32,
    pub trigger_signal : u32,
    pub trigger_threshold_seconds : u32,
    pub trigger_threshold_timer : bool,
    pub waiting_process_name : bool,
    pub diagnostics_logging_enabled : bool,
    pub gcore_process_id : i32,
    pub process_name : String,
    pub core_dump_path : String,
    pub core_dump_name : String,
    pub exit_monitor : bool,
    pub overwrite_existing_dump: bool,
}

//--------------------------------------------------------------------
//
// ApplyDefaults - Apply default values to configuration
//
//--------------------------------------------------------------------
pub fn apply_defaults(config:&mut ProcDumpConfiguration)
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
            process_name: Default::default(),
            number_of_dumps_collected: 0,
            number_of_dumps_to_collect: u32::MAX,
            trigger_threshold_cpu: u32::MAX,
            trigger_threshold_cpu_below: false,
            trigger_threshold_mem: u32::MAX,
            trigger_threshold_mem_below: false,
            trigger_threshold_threads: u32::MAX,
            trigger_threshold_file_descriptors: u32::MAX,
            trigger_threshold_timer: false,
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
            overwrite_existing_dump: false,
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
    println!("ProcDump v0.1 - Sysinternals process dump utility");
    println!("Copyright (C) 2020 Microsoft Corporation. All rights reserved. Licensed under the MIT license.");
    println!("Mark Russinovich, Mario Hewardt");
    println!("Sysinternals - www.sysinternals.com");
    println!();

    println!("Monitors a process and writes a core dump file when the process exceeds the");
    println!("specified criteria.");
    println!();
 }

// -----------------------------------------------------------------
// print_usage - Prints the procdump usage
// -----------------------------------------------------------------
pub fn print_usage(){
    println!();
    println!("Capture Usage:");
    println!("   procdump [-n Count]");
    println!("            [-s Seconds]");
    println!("            [-c|-cl CPU_Usage]");
    println!("            [-m|-ml Commit_Usage]");
    println!("            [-tc Thread_Threshold]");
    println!("            [-fc FileDescriptor_Threshold]");
    println!("            [-sig Signal_Number]");
    println!("            [-pf Polling_Frequency]");
    println!("            [-o]");
    println!("            [-log]");
    println!("            {{");
    println!("             {{{{[-w] Process_Name | [-pgid] PID}} [Dump_File | Dump_Folder]}}");
    println!("            }}");
    println!();
    println!("Options:");
    println!("   -n      Number of dumps to write before exiting.");
    println!("   -s      Consecutive seconds before dump is written (default is 10).");
    println!("   -c      CPU threshold above which to create a dump of the process.");
    println!("   -cl     CPU threshold below which to create a dump of the process.");
    println!("   -m      Memory commit threshold in MB at which to create a dump.");
    println!("   -ml     Trigger when memory commit drops below specified MB value.");
    println!("   -tc     Thread count threshold above which to create a dump of the process.");
    println!("   -fc     File descriptor count threshold above which to create a dump of the process.");
    println!("   -sig    Signal number to intercept to create a dump of the process.");
    println!("   -pf     Polling frequency.");
    println!("   -o      Overwrite existing dump file.");
    println!("   -log    Writes extended ProcDump tracing to syslog.");
    println!("   -w      Wait for the specified process to launch if it's not running.");
    println!("   -pgid   Process ID specified refers to a process group ID.");
    println!();
}

// -----------------------------------------------------------------
// get_options - Parses command line and populates the config
// -----------------------------------------------------------------
pub fn get_options(config: &mut ProcDumpConfiguration) -> i32
{
    let args: Vec<String> = env::args().collect();
    let mut process_specified = false;

    if args.len() < 2
    {
        print_usage();
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
        else if args[_i].eq("/c") || args[_i].eq("-c") || args[_i].eq("/cl") || args[_i].eq("-cl") {
            if args.get(_i+1)==None {
                print_usage();
                return -1;
            }

            let s = args.get(_i+1).unwrap();

            config.trigger_threshold_cpu = s.parse::<u32>().unwrap();
            if args[_i].eq("/cl") || args[_i].eq("-cl"){
                config.trigger_threshold_cpu_below = true;
            }

            _i+=1;
        }
        else if args[_i].eq("/tc") || args[_i].eq("-tc") {
            if args.get(_i+1)==None {
                print_usage();
                return -1;
            }

            let s = args.get(_i+1).unwrap();

            config.trigger_threshold_threads = s.parse::<u32>().unwrap();

            _i+=1;
        }
        else if args[_i].eq("/fc") || args[_i].eq("-fc") {
            if args.get(_i+1)==None {
                print_usage();
                return -1;
            }

            let s = args.get(_i+1).unwrap();

            config.trigger_threshold_file_descriptors = s.parse::<u32>().unwrap();

            _i+=1;
        }
        else if args[_i].eq("/sig") || args[_i].eq("-sig") {
            if args.get(_i+1)==None {
                print_usage();
                return -1;
            }

            let s = args.get(_i+1).unwrap();

            config.trigger_signal = s.parse::<u32>().unwrap();

            _i+=1;
        }
        else if args[_i].eq("/n") || args[_i].eq("-n") {
            if args.get(_i+1)==None {
                print_usage();
                return -1;
            }

            let s = args.get(_i+1).unwrap();

            config.number_of_dumps_to_collect = s.parse::<u32>().unwrap();

            _i+=1;
        }
        else if args[_i].eq("/s") || args[_i].eq("-s") {
            if args.get(_i+1)==None {
                print_usage();
                return -1;
            }

            let s = args.get(_i+1).unwrap();

            config.threshold_seconds = s.parse::<u32>().unwrap();

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
        else if args[_i].eq("/log") || args[_i].eq("-log")
        {
            config.diagnostics_logging_enabled = true;
        }
        else if args[_i].eq("/o") || args[_i].eq("-o")
        {
            config.overwrite_existing_dump = true;
        }
        else if args[_i].eq("/w") || args[_i].eq("-w")
        {
            config.waiting_process_name = true;
        }
        else if args[_i].eq("/pgid") || args[_i].eq("-pgid")
        {
            config.is_process_group_set = true;
        }
        else
        {
            //
            // Process targets
            //
            if process_specified && !config.core_dump_path.is_empty()
            {
                print_usage();
                return -1;
            }
            else if !process_specified
            {
                process_specified = true;

                let pid = match args[_i].parse::<i32>()
                {
                    Ok(pid) => pid,
                    Err(_e) => i32::MAX,
                };

                if pid == i32::MAX
                {
                    config.process_name = args[_i].clone();
                }
                else
                {
                    if config.is_process_group_set
                    {
                        config.process_pgid = pid;
                    }
                    else
                    {
                        config.process_id = pid;
                    }



                }
            }
            else if config.core_dump_path.is_empty()
            {
                if fs::metadata(&args[_i]).is_ok()
                {
                    let file = fs::metadata(&args[_i]).unwrap();
                    if file.is_dir()
                    {
                        config.core_dump_path = args[_i].clone();
                    }
                }
                else
                {
                    let path = Path::new(&args[_i]);
                    config.core_dump_path = path.parent().unwrap().to_str().unwrap().to_string();
                    config.core_dump_name = path.file_name().unwrap().to_str().unwrap().to_string();
                }

                if fs::metadata(&config.core_dump_path).is_ok()
                {
                    let file = fs::metadata(&config.core_dump_path).unwrap();
                    if !file.is_dir()
                    {
                        println!("Invalid directory ({}) provided for core dump output.", config.core_dump_path);
                        print_usage();
                        return -1;
                    }
                }
            }
        }

        _i+=1;
    }

    //
    // Validate multi arguments
    //

    // If no path was provided, assume the current directory
    if config.core_dump_path.is_empty()
    {
        config.core_dump_path = ".".to_string();
    }

    // Wait
    if config.waiting_process_name && config.process_id != i32::MAX
    {
        print_usage();
        return -1;
    }

    // If number of dumps to collect is set, but there is no other criteria, enable Timer here...
    if (config.trigger_threshold_cpu == u32::MAX) &&
        (config.trigger_threshold_mem == u32::MAX) &&
        (config.trigger_threshold_threads == u32::MAX) &&
        (config.trigger_threshold_file_descriptors == u32::MAX)
    {
        config.trigger_threshold_timer = true;
    }

    // Signal trigger can only be specified alone
    if config.trigger_signal != u32::MAX
    {
        if (config.trigger_threshold_cpu != u32::MAX) ||
            (config.trigger_threshold_mem != u32::MAX) ||
            (config.trigger_threshold_threads != u32::MAX) ||
            (config.trigger_threshold_file_descriptors != u32::MAX)
        {
            println!("Signal trigger must be the only trigger specified.");
            print_usage();
            return -1;
        }

        if config.polling_frequency != u64::MAX
        {
            println!("Polling interval has no meaning during signal monitoring.");
            print_usage();
            return -1;
        }

        config.trigger_threshold_timer = false;
    }

    // If we are monitoring multiple process, setting dump name doesn't make sense (path is OK)
    if (config.is_process_group_set || config.waiting_process_name) && !config.core_dump_name.is_empty()
    {
        println!("Setting core dump name in multi process monitoring is invalid (path is ok).");
        print_usage();
        return -1;
    }

    apply_defaults(config);

    0
}


// -----------------------------------------------------------------
// print_configuration - Prints the configuration
// -----------------------------------------------------------------
pub fn print_configuration(config: &ProcDumpConfiguration)
{
    if config.trigger_signal != u32::MAX
    {
        println!("** NOTE ** Signal triggers use PTRACE which will impact the performance of the target process");
        println!();
        println!();
    }

    //
    // Process target
    //
    if config.is_process_group_set
    {
        println!("Process Group: {}", config.process_pgid);
    }
    else if config.waiting_process_name
    {
        println!("Process Name: {}", config.process_name);
    }
    else
    {
        println!("Process: {} ({})", config.process_name, config.process_id);
    }

    //
    // Trigger CPU
    //
    if config.trigger_threshold_cpu != u32::MAX
    {
        if config.trigger_threshold_cpu_below
        {
            println!("CPU Threshold: < {}%", config.trigger_threshold_cpu);
        }
        else
        {
            println!("CPU Threshold: >= {}%", config.trigger_threshold_cpu);
        }
    }
    else
    {
        println!("CPU Threshold: n/a");
    }

    //
    // Trigger memory
    //
    if config.trigger_threshold_mem != u32::MAX
    {
        if config.trigger_threshold_mem_below
        {
            println!("Memory Threshold: < {}MB", config.trigger_threshold_mem);
        }
        else
        {
            println!("Memory Threshold: >= {}MB", config.trigger_threshold_mem);
        }
    }
    else
    {
        println!("Memory Threshold: n/a");
    }

    //
    // Trigger thread count
    //
    if config.trigger_threshold_threads != u32::MAX
    {
        println!("Thread Threshold: >= {}", config.trigger_threshold_threads);
    }
    else
    {
        println!("Thread Threshold: n/a");
    }

    //
    // Trigger file desc count
    //
    if config.trigger_threshold_file_descriptors != u32::MAX
    {
        println!("File Descriptor Threshold: >= {}", config.trigger_threshold_file_descriptors);
    }
    else
    {
        println!("File Descriptor Threshold: n/a");
    }

    //
    // Trigger signal
    //
    if config.trigger_signal != u32::MAX
    {
        println!("Signal: {}", config.trigger_signal);
    }
    else
    {
        println!("Signal: n/a");
    }

    println!("Polling interval (ms): {}", config.polling_frequency);
    println!("Threshold (s): {}", config.threshold_seconds);
    println!("Number of dumps: {}", config.number_of_dumps_to_collect);
    println!("Output Directory: {}", config.core_dump_path);

    if !config.core_dump_name.is_empty()
    {
        println!("Custom name for core dumps: {}_<counter>.<pid>", config.core_dump_name);
    }

}