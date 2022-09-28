// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Manages the procdump configuration
//
//--------------------------------------------------------------------
use std::env;

//
// Struct used to communicate the current procdump configuration
//
pub struct ProcDumpConfiguration{
    pub mem_trigger_value: u32,
    pub mem_trigger_below_value: bool,
    pub process_id: u32,
    pub polling_frequency: u64,
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

            config.mem_trigger_value = s.parse::<u32>().unwrap();
            if args[_i].eq("/ml") || args[_i].eq("-ml"){
                config.mem_trigger_below_value = true;
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
            config.process_id = args[_i].parse::<u32>().unwrap();
        }

        _i+=1;
    }

    0
}


// -----------------------------------------------------------------
// print_triggers - Prints active triggers
// -----------------------------------------------------------------
pub fn print_triggers(config: &ProcDumpConfiguration){
    println!("Mem trigger value: {}MB", config.mem_trigger_value);
    println!("Mem trigger below: {}", config.mem_trigger_below_value);
    println!("PID: {}", config.mem_trigger_below_value);
}