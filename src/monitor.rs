// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Main process monitoring
//
//--------------------------------------------------------------------
use crate::procdumpconfiguration::ProcDumpConfiguration;
use crate::procdumpconfiguration::print_configuration;
use crate::processhelpers::look_up_process_by_pid;
use crate::processhelpers::look_up_process_pid_by_name;
use crate::processhelpers::look_up_process_name_by_pid;
use crate::triggerthreadprocs;
use std::collections::HashMap;
use std::thread;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub struct MonitoredProcessMapEntry
{
    pub active: bool,
    pub starttime: u64,
    pub config: Arc<Mutex<ProcDumpConfiguration>>,      // ProcDumpConfiguration is shared and hence protected by Mutex wrapped by an Arc for atomic reference couting
    pub threads: Vec<Option<JoinHandle<u32>>>,
}

// -----------------------------------------------------------------
// monitor_processes - Monitors all processes and creates monitors
// based on the configuration
// -----------------------------------------------------------------
pub fn monitor_processes(config: &mut ProcDumpConfiguration)
{
    let mut monitored_process_map: HashMap<i32, MonitoredProcessMapEntry>;
    monitored_process_map = HashMap::new();

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
            // Set the process ID so the monitor can target.
            config.process_id = look_up_process_pid_by_name(&config.process_name);

            if config.process_id == i32::MAX
            {
                println!("No process matching the specified name ({}) can be found.", config.process_name);
                return;
            }
        }
        else if config.process_id != i32::MAX && !look_up_process_by_pid(config.process_id)
        {
            println!("No process matching the specified PID ({}) can be found.", config.process_id);
            return;
        }

        config.process_name = look_up_process_name_by_pid(config.process_id);
        // TODO: Get starttime

        let config_clone = config.clone();
        let mut entry = MonitoredProcessMapEntry
        {
            active: false,
            starttime: 0,
            config: Arc::new(Mutex::new(config_clone)),
            threads: Vec::new(),
        };

        print_configuration(config);

        if !start_monitor(&mut entry)
        {
            println!("Failed to start monitor for pid: {}", config.process_id);
            return;
        }
        monitored_process_map.insert(config.process_id, entry);

        let entry_o = monitored_process_map.get_mut(&config.process_id).unwrap();

        wait_for_monitor_exit(entry_o);
        monitored_process_map.remove(&config.process_id);
    }

}


// -----------------------------------------------------------------
// start_monitor - Starts a monitor based on the configuration
// -----------------------------------------------------------------
pub fn start_monitor(entry: &mut MonitoredProcessMapEntry) -> bool
{
    {
        let mut guard = entry.config.lock().unwrap();
        if guard.trigger_threshold_mem != u32::MAX
        {
            let config_clone = guard.clone();

            let thread = thread::Builder::new().name("Memory monitor thread".to_string()).spawn(move || triggerthreadprocs::mem_monitoring_thread(config_clone));
            if thread.is_err()
            {
                return false;
            }

            entry.threads.push(Some(thread.unwrap()));
        }
    }

    if entry.config.trigger_threshold_cpu != u32::MAX
    {
        let config_clone = entry.config.clone();

        let thread = thread::Builder::new().name("CPU monitor thread".to_string()).spawn(move || triggerthreadprocs::cpu_monitoring_thread(config_clone));
        if thread.is_err()
        {
            return false;
        }

        entry.threads.push(Some(thread.unwrap()));
    }

    if entry.config.trigger_threshold_threads != u32::MAX
    {
        let config_clone = entry.config.clone();

        let thread = thread::Builder::new().name("Thread monitor thread".to_string()).spawn(move || triggerthreadprocs::thread_monitoring_thread(config_clone));
        if thread.is_err()
        {
            return false;
        }

        entry.threads.push(Some(thread.unwrap()));

    }

    if entry.config.trigger_threshold_file_descriptors != u32::MAX
    {
        let config_clone = entry.config.clone();

        let thread = thread::Builder::new().name("File monitor thread".to_string()).spawn(move || triggerthreadprocs::file_monitoring_thread(config_clone));
        if thread.is_err()
        {
            return false;
        }

        entry.threads.push(Some(thread.unwrap()));
    }

    if entry.config.trigger_signal != u32::MAX
    {
        let config_clone = entry.config.clone();

        let thread = thread::Builder::new().name("Signal monitor thread".to_string()).spawn(move || triggerthreadprocs::signal_monitoring_thread(config_clone));
        if thread.is_err()
        {
            return false;
        }

        entry.threads.push(Some(thread.unwrap()));

    }

    if entry.config.trigger_threshold_timer
    {
        let config_clone = entry.config.clone();

        let thread = thread::Builder::new().name("Timer monitor thread".to_string()).spawn(move || triggerthreadprocs::timer_monitoring_thread(config_clone));
        if thread.is_err()
        {
            return false;
        }

        entry.threads.push(Some(thread.unwrap()));

    }

    true
}

// -----------------------------------------------------------------
// wait_for_monitor_exit - Waits for a monitor to exit
// -----------------------------------------------------------------
pub fn wait_for_monitor_exit(entry: &mut MonitoredProcessMapEntry) -> bool
{
    for i in 0..entry.threads.len()
    {
        let join_handle = std::mem::take(&mut entry.threads[i]);
        join_handle.unwrap().join();
    }

    true
}