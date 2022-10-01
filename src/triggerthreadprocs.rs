// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Contains the memory consumption monitoring thread
//
//--------------------------------------------------------------------
use crate::procdumpconfiguration;
use std::fs;
use std::{thread, time};
use std::process::Command;


// --------------------------------------------------------------------
// cpu_monitoring_thread - Monitors for cpu consumption based on config
// --------------------------------------------------------------------
pub fn cpu_monitoring_thread(config: &procdumpconfiguration::ProcDumpConfiguration) -> u32
{

    0
}

// --------------------------------------------------------------------
// thread_monitoring_thread - Monitors for thread count  based on config
// --------------------------------------------------------------------
pub fn thread_monitoring_thread(config: &procdumpconfiguration::ProcDumpConfiguration) -> u32
{

    0
}

// --------------------------------------------------------------------
// file_monitoring_thread - Monitors for file desc count  based on config
// --------------------------------------------------------------------
pub fn file_monitoring_thread(config: &procdumpconfiguration::ProcDumpConfiguration) -> u32
{

    0
}

// --------------------------------------------------------------------
// signal_monitoring_thread - Monitors for signal based on config
// --------------------------------------------------------------------
pub fn signal_monitoring_thread(config: &procdumpconfiguration::ProcDumpConfiguration) -> u32
{

    0
}


// --------------------------------------------------------------------
// timer_monitoring_thread - Timer based monito  based on config
// --------------------------------------------------------------------
pub fn timer_monitoring_thread(config: &procdumpconfiguration::ProcDumpConfiguration) -> u32
{

    0
}


// --------------------------------------------------------------------
// mem_monitoring_thread - Monitors for mem consumption based on config
// --------------------------------------------------------------------
pub fn mem_monitoring_thread(config: &procdumpconfiguration::ProcDumpConfiguration) -> u32
{

    let polling_frequency = time::Duration::from_millis(config.polling_frequency);

    // assume pagesize
    let pagesize = 4;

    loop {
        //
        // Read /proc/{pid}/stat file to get process statistics
        //
        let stat_path = format!("/proc/{}/stat", config.process_id);
        let statcontents = fs::read_to_string(stat_path).expect("Stat file not found.");

        //
        // Get resident set from stat
        //
        let mut rss = statcontents.split(" ").nth(23).unwrap().parse::<u32>().unwrap();
        rss = (rss * pagesize) >> 10;

        //
        // Get swap for stat
        //
        let mut swap = statcontents.split(" ").nth(35).unwrap().parse::<u32>().unwrap();
        swap = (swap * pagesize) >> 10;

        let mem_usage = rss + swap;

        if (config.trigger_threshold_mem_below && mem_usage < config.trigger_threshold_mem) ||
            (!config.trigger_threshold_mem_below && mem_usage >= config.trigger_threshold_mem) {

                //
                // Memory consumption trigger triggered
                //
                println!("Trigger: Memory usage:{}MB on process ID: {}", mem_usage, config.process_id);

                //
                // Trigger dump gen...
                //
                let string_pid: String = config.process_id.to_string();
                Command::new("gcore").arg(string_pid).output().expect("Failed to execute gcore.");

                println!("Core dump generated.");
                break;
            }


        thread::sleep(polling_frequency);
    }

    0
}