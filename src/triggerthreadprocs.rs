// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Contains all monitor trigger threads
//
//--------------------------------------------------------------------
extern crate nix;
extern crate sysinfo;
use crate::dumpwriter::write_dump;
use crate::procdumpconfiguration::ProcDumpConfiguration;
use std::fs;
use std::thread::park_timeout;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use nix::unistd::*;
use sysinfo::*;

// --------------------------------------------------------------------
// should_continue_monitoring - returns true if monitor thread should
// continue monitoring, otherwise false
// --------------------------------------------------------------------
pub fn should_continue_monitoring(config: &Arc<Mutex<ProcDumpConfiguration>>) -> bool
{
    let lock = config.lock().unwrap();

    // Have we exceeded dump count?
    if lock.number_of_dumps_collected >= lock.number_of_dumps_to_collect
    {
        return false;
    }

    // Is target process terminated?
    if lock.process_terminated
    {
        return false;
    }

    // check if any process are running with PGID
    if lock.process_pgid != i32::MAX
    {
/*       let pgid = Pid::from_raw(-1 * lock.process_pgid);
      let res = kill(pgid, None);
        if res.is_err()
        {
            println!("Error");
            lock.process_terminated = true;
            return false;
        }*/
    }

    // check if any process are running with PID
    if lock.process_id != i32::MAX
    {
        /*
            let pid = Pid::from_raw(lock.process_id);

        let res = kill(pid, None);
        if res.is_err()
        {
            println!("Target process {} is no longer alive", lock.process_id);
            lock.process_terminated = true;
            return false;
        }*/
    }

    true
}

// --------------------------------------------------------------------
// cpu_monitoring_thread - Monitors for cpu consumption based on config
// --------------------------------------------------------------------
pub fn cpu_monitoring_thread(config: Arc<Mutex<ProcDumpConfiguration>>) -> u32
{
    let lock = config.lock().unwrap();
    let timeout = lock.polling_frequency/1000;
    let in_between_dumps = lock.threshold_seconds;
    let pid = lock.process_id;
    let trigger_below = lock.trigger_threshold_mem_below;
    let trigger_threshold = lock.trigger_threshold_cpu;
    drop(lock);

    let mut trigger_type = String::new();
    trigger_type.push_str("cpu");

    let hz: i64 = nix::unistd::sysconf(SysconfVar::CLK_TCK).unwrap().unwrap();

    let mut sys = System::new_all();

    while should_continue_monitoring(&config)
    {
        // Read /proc/{pid}/stat file to get process statistics
        let stat_path = format!("/proc/{}/stat", pid);
        let statcontents = fs::read_to_string(stat_path).expect("Stat file not found.");

        // Get proc stats for CPU
        let utime = statcontents.split(" ").nth(13).unwrap().parse::<u64>().unwrap();
        let stime = statcontents.split(" ").nth(14).unwrap().parse::<u64>().unwrap();
        let starttime = statcontents.split(" ").nth(21).unwrap().parse::<u64>().unwrap();
        sys.refresh_all();
        let uptime = sys.uptime();

        let total_time = (utime + stime) / u64::try_from(hz).unwrap();
        let elapsed_time = uptime - (starttime/u64::try_from(hz).unwrap());
        let cpu_usage = ((total_time as f64/elapsed_time as f64) * 100 as f64)  as u32;

        println!("CPU usage:{}% on process ID: {}", cpu_usage, pid);

        if (trigger_below && cpu_usage < trigger_threshold.into()) || (!trigger_below && cpu_usage >= trigger_threshold.into())
        {
            println!("Trigger: CPU usage:{}% on process ID: {}", cpu_usage, pid);
            write_dump(&config, &trigger_type);
            if !should_continue_monitoring(&config)
            {
                // We've reached a stop state, exit
                break;
            }

            // Wait for time between dumps
            let timeout_remaining = in_between_dumps;
            let elapsed = park_thread(timeout_remaining.into());
            if elapsed < Duration::from_secs(in_between_dumps.into())
            {
                // Thread was unparked as a result of cancellation...exit
                break;
            }
        }
        else
        {
            // Wait for polling frequency
            let timeout_remaining = timeout;
            let elapsed = park_thread(timeout_remaining);
            if elapsed < Duration::from_secs(timeout)
            {
                // Thread was unparked as a result of cancellation...exit
                break;
            }
        }
    }

    0
}

// --------------------------------------------------------------------
// thread_monitoring_thread - Monitors for thread count  based on config
// --------------------------------------------------------------------
pub fn thread_monitoring_thread(config: Arc<Mutex<ProcDumpConfiguration>>) -> u32
{
    let lock = config.lock().unwrap();
    let trigger_thread_threshold = lock.trigger_threshold_threads;
    let timeout = lock.polling_frequency/1000;
    let in_between_dumps = lock.threshold_seconds;
    let pid = lock.process_id;
    drop(lock);

    let mut trigger_type = String::new();
    trigger_type.push_str("threads");

    while should_continue_monitoring(&config)
    {
        // Read /proc/{pid}/stat file to get process statistics
        let stat_path = format!("/proc/{}/stat", pid);
        let statcontents = fs::read_to_string(stat_path).expect("Stat file not found.");

        // Get resident set from stat
        let thread_count = statcontents.split(" ").nth(19).unwrap().parse::<i64>().unwrap();

        if thread_count >= trigger_thread_threshold.into()
        {
            println!("Trigger: Thread count:{} on process ID: {}", thread_count, pid);
            write_dump(&config, &trigger_type);
            if !should_continue_monitoring(&config)
            {
                // We've reached a stop state, exit
                break;
            }

            // Wait for time between dumps
            let timeout_remaining = in_between_dumps;
            let elapsed = park_thread(timeout_remaining.into());
            if elapsed < Duration::from_secs(in_between_dumps.into())
            {
                // Thread was unparked as a result of cancellation...exit
                break;
            }
        }
        else
        {
            // Wait for polling frequency
            let timeout_remaining = timeout;
            let elapsed = park_thread(timeout_remaining);
            if elapsed < Duration::from_secs(timeout)
            {
                // Thread was unparked as a result of cancellation...exit
                break;
            }
        }
    }

    0
}

// --------------------------------------------------------------------
// file_monitoring_thread - Monitors for file desc count  based on config
// --------------------------------------------------------------------
pub fn file_monitoring_thread(config: Arc<Mutex<ProcDumpConfiguration>>) -> u32
{
    let lock = config.lock().unwrap();
    let trigger_file_threshold = lock.trigger_threshold_file_descriptors;
    let timeout = lock.polling_frequency/1000;
    let in_between_dumps = lock.threshold_seconds;
    let pid = lock.process_id;
    drop(lock);

    let mut trigger_type = String::new();
    trigger_type.push_str("file_descriptor");

    while should_continue_monitoring(&config)
    {
        // Read /proc/{pid}/stat file to get process statistics
        let stat_path = format!("/proc/{}/fdinfo", pid);

        let paths = fs::read_dir(stat_path).unwrap();
        let mut num_file_descriptors = 0;
        for _ in paths
        {
            num_file_descriptors += 1;
        }

        if num_file_descriptors >= trigger_file_threshold
        {
            println!("Trigger: File descriptors:{} on process ID: {}", num_file_descriptors, pid);
            write_dump(&config, &trigger_type);
            if !should_continue_monitoring(&config)
            {
                // We've reached a stop state, exit
                break;
            }

            // Wait for time between dumps
            let timeout_remaining = in_between_dumps;
            let elapsed = park_thread(timeout_remaining.into());
            if elapsed < Duration::from_secs(in_between_dumps.into())
            {
                // Thread was unparked as a result of cancellation...exit
                break;
            }
        }
        else
        {
            // Wait for polling frequency
            let timeout_remaining = timeout;
            let elapsed = park_thread(timeout_remaining);
            if elapsed < Duration::from_secs(timeout)
            {
                // Thread was unparked as a result of cancellation...exit
                break;
            }
        }
    }

    0
}

// --------------------------------------------------------------------
// This thread monitors for a specific signal to be sent to target process.
// It uses ptrace (PTRACE_SEIZE) and once the signal with the corresponding
// signal number is intercepted, it detaches from the target process in a stopped state
// followed by invoking gcore to generate the dump. Once completed, a SIGCONT followed by the
// original signal is sent to the target process. Signals of non-interest are simply forwarded
// to the target process.
// --------------------------------------------------------------------
pub fn signal_monitoring_thread(_: Arc<Mutex<ProcDumpConfiguration>>) -> u32
{

    0
}


// --------------------------------------------------------------------
// timer_monitoring_thread - Timer based monitor  based on config
// --------------------------------------------------------------------
pub fn timer_monitoring_thread(config: Arc<Mutex<ProcDumpConfiguration>>) -> u32
{
    let lock = config.lock().unwrap();
    let timeout = lock.polling_frequency/1000;
    let in_between_dumps = lock.threshold_seconds;
    let pid = lock.process_id;
    drop(lock);

    let mut trigger_type = String::new();
    trigger_type.push_str("timer");


    while should_continue_monitoring(&config)
    {
        // Wait for polling frequency
        let timeout_remaining = timeout;
        let elapsed = park_thread(timeout_remaining.into());
        if elapsed >= Duration::from_secs(timeout)
        {
            // Polling frequency has elapsed...generate a dump
            {
                let lock = config.lock().unwrap();
                println!("Trigger: Timer:{}(s) on process ID: {}", lock.polling_frequency/1000, lock.process_id);
            }

            // Write Dump
            println!("Trigger: Timer:{}(s) on process ID: {}", timeout, pid);
            write_dump(&config, &trigger_type);
            if !should_continue_monitoring(&config)
            {
                // We've reached a stop state, exit
                break;
            }

            // Wait for time between dumps
            let timeout_remaining = in_between_dumps;
            let elapsed = park_thread(timeout_remaining.into());
            if elapsed < Duration::from_secs(in_between_dumps.into())
            {
                // Thread was unparked as a result of cancellation...exit
                break;
            }
        }
        else
        {
            // Thread was unparked as a result of cancellation...exit
            break;
        }
    }

    0
}


// --------------------------------------------------------------------
// mem_monitoring_thread - Monitors for mem consumption based on config
// --------------------------------------------------------------------
pub fn mem_monitoring_thread(config: Arc<Mutex<ProcDumpConfiguration>>) -> u32
{
    let lock = config.lock().unwrap();
    let timeout = lock.polling_frequency/1000;
    let in_between_dumps = lock.threshold_seconds;
    let pid = lock.process_id;
    let trigger_below = lock.trigger_threshold_mem_below;
    let trigger_threshold = lock.trigger_threshold_mem;
    drop(lock);

    let mut trigger_type = String::new();
    trigger_type.push_str("memory");

    let pagesize = nix::unistd::sysconf(SysconfVar::PAGE_SIZE).unwrap().unwrap() >> 10;

    while should_continue_monitoring(&config)
    {
        // Read /proc/{pid}/stat file to get process statistics
        let stat_path = format!("/proc/{}/stat", pid);
        let statcontents = fs::read_to_string(stat_path).expect("Stat file not found.");

        // Get resident set from stat
        let mut rss = statcontents.split(" ").nth(23).unwrap().parse::<i64>().unwrap();
        rss = (rss * pagesize) >> 10;

        // Get swap for stat
        let mut swap = statcontents.split(" ").nth(35).unwrap().parse::<i64>().unwrap();
        swap = (swap * pagesize) >> 10;

        let mem_usage = rss + swap;

        if (trigger_below && mem_usage < trigger_threshold.into()) || (!trigger_below && mem_usage >= trigger_threshold.into())
        {
            println!("Trigger: Commit usage:{}MB on process ID: {}", mem_usage, pid);
            write_dump(&config, &trigger_type);
            if !should_continue_monitoring(&config)
            {
                // We've reached a stop state, exit
                break;
            }

            // Wait for time between dumps
            let timeout_remaining = in_between_dumps;
            let elapsed = park_thread(timeout_remaining.into());
            if elapsed < Duration::from_secs(in_between_dumps.into())
            {
                // Thread was unparked as a result of cancellation...exit
                break;
            }
        }
        else
        {
            // Wait for polling frequency
            let timeout_remaining = timeout;
            let elapsed = park_thread(timeout_remaining);
            if elapsed < Duration::from_secs(timeout)
            {
                // Thread was unparked as a result of cancellation...exit
                break;
            }
        }
    }

    0
}

// --------------------------------------------------------------------
// park_thread - Helper function to park a thread with a timeout
// --------------------------------------------------------------------
pub fn park_thread(duration: u64) -> Duration
{
    let beginning_park = Instant::now();
    let timeout_remaining = Duration::from_secs(duration);
    park_timeout(timeout_remaining);
    let elapsed = beginning_park.elapsed();

    elapsed
}