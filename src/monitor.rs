// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Main process monitoring
//
//--------------------------------------------------------------------
use crate::procdumpconfiguration;
use crate::processhelpers::get_process_name;
use crate::triggerthreadprocs;
use crate::processhelpers;


pub struct MonitoredProcessMapEntry
{
    pub active: bool,
    pub starttime: u64,
}


// -----------------------------------------------------------------
// monitor_processes - Monitors all processes and creates monitors
// based on the configuration
// -----------------------------------------------------------------
pub fn monitor_processes(config: &mut procdumpconfiguration::ProcDumpConfiguration)
{
    let mut monitored_process_map: Vec<MonitoredProcessMapEntry> = Vec::new();

    if config.waiting_process_name
    {
        println!("Waiting for processes '{}' to launch", config.process_name);
    }
    if config.is_process_group_set
    {
        println!("Monitoring processes of PGID '{}'", config.process_pgid);
    }

    // TODO: Create signal handler thread

    println!();
    println!();
    println!("Press Ctrl-C to end monitoring without terminating the process(es).");

    if !config.waiting_process_name && !config.is_process_group_set
    {
        //
        // Monitoring single process (-p)
        //

        //
        // Make sure target process exists
        //

        // If we have a process name find it to make sure it exists
        if !config.process_name.is_empty()
        {
            if get_process_name(&config.process_name)
            {
                println!("Found process name");
            }
        }
    }

}


