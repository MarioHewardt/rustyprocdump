// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Main process monitoring
//
//--------------------------------------------------------------------
use crate::procdumpconfiguration::ProcDumpConfiguration;
use crate::procdumpconfiguration::print_configuration;
use crate::processhelpers::*;
use crate::triggerthreadprocs;
use std::collections::HashMap;
use std::{thread, time};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::fs;

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

    println!("Press Ctrl-C to end monitoring without terminating the process(es).");
    println!();

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
            config.process_id = get_process_pid_by_name(&config.process_name);

            if config.process_id == i32::MAX
            {
                println!("No process matching the specified name ({}) can be found.", config.process_name);
                return;
            }
        }
        else if config.process_id != i32::MAX && !is_process_running(config.process_id)
        {
            println!("No process matching the specified PID ({}) can be found.", config.process_id);
            return;
        }

        let stat_path = format!("/proc/{}/stat", config.process_id);
        let statcontents = fs::read_to_string(stat_path).expect("Stat file not found.");

        config.process_start_time = statcontents.split(" ").nth(21).unwrap().parse::<u64>().unwrap();
        config.process_name = get_process_name_by_pid(config.process_id);

        let config_clone = config.clone();
        let mut entry = MonitoredProcessMapEntry
        {
            active: false,
            starttime: get_process_start_time(config_clone.process_id),
            config: Arc::new(Mutex::new(config_clone)),
            threads: Vec::new(),
        };

        print_configuration(config);
        println!();

        if !start_monitor(&mut entry)
        {
            println!("Failed to start monitor for pid: {}", config.process_id);
            return;
        }
        monitored_process_map.insert(config.process_id, entry);

        let entry_o = monitored_process_map.get_mut(&config.process_id).unwrap();

        wait_for_monitor_exit(entry_o);
        println!("Stopping monitor for process {} ({})", config.process_name, config.process_id);
        monitored_process_map.remove(&config.process_id);
    }
    else
    {
        print_configuration(config);
        println!();

        let mut num_monitored_process = 0;
        loop
        {
            // Multi process monitoring

            // Get PGID of process
            let pgid = get_process_pgid(config.process_pgid);

            if config.is_process_group_set && pgid == u64::MAX
            {
                println!("No process matching the specified PGID can be found.");
                return;
            }

            // Iterate over all running processes
            for entry in fs::read_dir("/proc/").expect("I told you this directory exists")
            {
                let entry = entry.expect("I couldn't read something inside the directory");
                let path = entry.path();
                let pid = path.file_name().unwrap().to_str().unwrap().to_lowercase();
                let proc_pid = match pid.parse::<i32>()
                {
                    Ok(pid) => pid,
                    Err(_err) => { continue; },
                };

                if config.is_process_group_set
                {
                    // We're monitoring a process group (-pgid)
                    let pgid = get_process_pgid(proc_pid);
                    if pgid != u64::MAX && config.process_pgid as u64 == pgid
                    {
                        let start_time = get_process_start_time(proc_pid);

                        if !monitored_process_map.contains_key(&proc_pid)
                        {
                            // New process, setup new monitor
                            let mut config_clone = config.clone();
                            config_clone.process_id = proc_pid;
                            config_clone.process_name = get_process_name_by_pid(proc_pid);
                            let mut entry = MonitoredProcessMapEntry
                            {
                                active: true,
                                starttime: get_process_start_time(config_clone.process_id),
                                config: Arc::new(Mutex::new(config_clone)),
                                threads: Vec::new(),
                            };

                            if !start_monitor(&mut entry)
                            {
                                println!("Failed to start monitor for pid: {}", config.process_id);
                            }

                            monitored_process_map.insert(proc_pid, entry);
                            num_monitored_process += 1;
                        }
                        else
                        {
                            // We've already seen this process...
                            // If the active flag = true, its an active monitor
                            // If the active flag = false, check to see if starttimes are different...
                            // if they are, we have a case of PID reuse (highly unlikely)
                            let entry = monitored_process_map.get(&proc_pid).unwrap();
                            if entry.active == false && entry.starttime != start_time
                            {
                                // PID reuse

                                // First remove existing entry since we have to setup a new monitor (monitoring threads etc)
                                let lock = entry.config.lock().unwrap();
                                let pid = lock.process_id;
                                drop(lock);

                                monitored_process_map.remove(&pid);
                                num_monitored_process -= 1;

                                let mut config_clone = config.clone();
                                config_clone.process_id = proc_pid;
                                config_clone.process_name = get_process_name_by_pid(proc_pid);
                                let mut entry = MonitoredProcessMapEntry
                                {
                                    active: true,
                                    starttime: get_process_start_time(config_clone.process_id),
                                    config: Arc::new(Mutex::new(config_clone)),
                                    threads: Vec::new(),
                                };

                                if !start_monitor(&mut entry)
                                {
                                    println!("Failed to start monitor for pid: {}", config.process_id);
                                }

                                monitored_process_map.insert(proc_pid, entry);
                                num_monitored_process += 1;
                            }
                        }
                    }
                }
                else if config.waiting_process_name
                {
                    // We are monitoring for a process name (-w)
                    let name_for_pid = get_process_name_by_pid(proc_pid);

                    if !name_for_pid.is_empty() && name_for_pid.eq(&config.process_name)
                    {
                        let start_time = get_process_start_time(proc_pid);

                        if !monitored_process_map.contains_key(&proc_pid)
                        {
                            // New process, setup new monitor
                            let mut config_clone = config.clone();
                            config_clone.process_id = proc_pid;
                            config_clone.process_name = get_process_name_by_pid(proc_pid);
                            let mut entry = MonitoredProcessMapEntry
                            {
                                active: true,
                                starttime: get_process_start_time(config_clone.process_id),
                                config: Arc::new(Mutex::new(config_clone)),
                                threads: Vec::new(),
                            };

                            if !start_monitor(&mut entry)
                            {
                                println!("Failed to start monitor for pid: {}", config.process_id);
                            }

                            monitored_process_map.insert(proc_pid, entry);
                            num_monitored_process += 1;
                        }
                        else
                        {
                            // We've already seen this process...
                            // If the active flag = true, its an active monitor
                            // If the active flag = false, check to see if starttimes are different...
                            // if they are, we have a case of PID reuse (highly unlikely)
                            let entry = monitored_process_map.get(&proc_pid).unwrap();
                            if entry.active == false && entry.starttime != start_time
                            {
                                // PID reuse

                                // First remove existing entry since we have to setup a new monitor (monitoring threads etc)
                                let lock = entry.config.lock().unwrap();
                                let pid = lock.process_id;
                                drop(lock);

                                monitored_process_map.remove(&pid);
                                num_monitored_process -= 1;

                                let mut config_clone = config.clone();
                                config_clone.process_id = proc_pid;
                                config_clone.process_name = get_process_name_by_pid(proc_pid);
                                let mut entry = MonitoredProcessMapEntry
                                {
                                    active: true,
                                    starttime: get_process_start_time(config_clone.process_id),
                                    config: Arc::new(Mutex::new(config_clone)),
                                    threads: Vec::new(),
                                };

                                if !start_monitor(&mut entry)
                                {
                                    println!("Failed to start monitor for pid: {}", config.process_id);
                                }

                                monitored_process_map.insert(proc_pid, entry);
                                num_monitored_process += 1;
                            }
                        }
                    }
                }
            }

            // Iterate over the list of monitored processes and stash the ones which we should stop monitoring
            let mut del_items: Vec<i32> = Vec::new();
            for (_pid, entry) in &mut monitored_process_map
            {
                if entry.active
                {
                    let lock = entry.config.lock().unwrap();
                    if lock.is_quit || lock.number_of_dumps_collected == lock.number_of_dumps_to_collect
                    {
                        del_items.push(lock.process_id);
                    }
                }
            }

            // Now walk the deleted list and delete from hashmap
            for item in &del_items
            {
                let entry_o = monitored_process_map.get_mut(&item).unwrap();
                println!("Stopping monitors for process: {}", item);
                wait_for_monitor_exit(entry_o);
                entry_o.active = false;
                num_monitored_process -= 1;
            }

            // Exit if we are monitoring PGID and there are no more processes to monitor.
            // If we are monitoring for processes based on a process name we keep monitoring
            if num_monitored_process == 0 && config.waiting_process_name == false
            {
                println!("Break");
                break;
            }

            thread::sleep(time::Duration::from_millis(config.polling_frequency));
        }
    }

    println!("Leaving loop");

}


// -----------------------------------------------------------------
// start_monitor - Starts a monitor based on the configuration
// -----------------------------------------------------------------
pub fn start_monitor(entry: &mut MonitoredProcessMapEntry) -> bool
{
    let guard = entry.config.lock().unwrap();
    if guard.trigger_threshold_mem != u32::MAX
    {
        let config_clone = entry.config.clone();

        let thread = thread::Builder::new().name("Memory monitor thread".to_string()).spawn(move || triggerthreadprocs::mem_monitoring_thread(config_clone));
        if thread.is_err()
        {
            return false;
        }

        entry.threads.push(Some(thread.unwrap()));
    }

    if guard.trigger_threshold_cpu != u32::MAX
    {
        let config_clone = entry.config.clone();

        let thread = thread::Builder::new().name("CPU monitor thread".to_string()).spawn(move || triggerthreadprocs::cpu_monitoring_thread(config_clone));
        if thread.is_err()
        {
            return false;
        }

        entry.threads.push(Some(thread.unwrap()));
    }

    if guard.trigger_threshold_threads != u32::MAX
    {
        let config_clone = entry.config.clone();

        let thread = thread::Builder::new().name("Thread monitor thread".to_string()).spawn(move || triggerthreadprocs::thread_monitoring_thread(config_clone));
        if thread.is_err()
        {
            return false;
        }

        entry.threads.push(Some(thread.unwrap()));

    }

    if guard.trigger_threshold_file_descriptors != u32::MAX
    {
        let config_clone = entry.config.clone();

        let thread = thread::Builder::new().name("File monitor thread".to_string()).spawn(move || triggerthreadprocs::file_monitoring_thread(config_clone));
        if thread.is_err()
        {
            return false;
        }

        entry.threads.push(Some(thread.unwrap()));
    }

    if guard.trigger_signal != u32::MAX
    {
        let config_clone = entry.config.clone();

        let thread = thread::Builder::new().name("Signal monitor thread".to_string()).spawn(move || triggerthreadprocs::signal_monitoring_thread(config_clone));
        if thread.is_err()
        {
            return false;
        }

        entry.threads.push(Some(thread.unwrap()));

    }

    if guard.trigger_threshold_timer
    {
        let config_clone = entry.config.clone();

        let thread = thread::Builder::new().name("Timer monitor thread".to_string()).spawn(move || triggerthreadprocs::timer_monitoring_thread(config_clone));
        if thread.is_err()
        {
            return false;
        }

        entry.threads.push(Some(thread.unwrap()));

    }

    println!("Starting monitor for process {} ({})", guard.process_name, guard.process_id);

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
        join_handle.unwrap().join().expect("Failed to join monitor thread");
    }

    true
}