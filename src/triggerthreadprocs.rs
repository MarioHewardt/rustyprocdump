// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Contains the memory consumption monitoring thread
//
//--------------------------------------------------------------------
extern crate nix;
use crate::procdumpconfiguration::ProcDumpConfiguration;
use std::fs;
use std::{thread, time};
use std::process::Command;
use std::thread::park_timeout;
use std::time::{Instant, Duration};
use nix::sys::signal::*;
use nix::unistd::Pid;
use std::sync::{Arc, Mutex};

// --------------------------------------------------------------------
// should_continue_monitoring - returns true if monitor thread should
// continue monitoring, otherwise false
// --------------------------------------------------------------------
pub fn should_continue_monitoring(config: &Arc<Mutex<ProcDumpConfiguration>>) -> bool
{
    let mut lock = config.lock().unwrap();

    // Have we exceeded dump count?
    if lock.number_of_dumps_collected > lock.number_of_dumps_to_collect
    {
        return false;
    }

    // Is target process terminated?
    if lock.process_terminated
    {
        return false;
    }

    // check if any process are running with PGID
    let pgid = Pid::from_raw(-1 * lock.process_pgid);
    if lock.process_pgid != i32::MAX
    {
        let res = kill(pgid, None);
        if res.is_err()
        {
            lock.process_terminated = true;
            return false;
        }
    }

    // check if any process are running with PID
    let pid = Pid::from_raw(lock.process_id);
    if lock.process_id != i32::MAX
    {
        let res = kill(pid, None);
        if res.is_err()
        {
            lock.process_terminated = true;
            return false;
        }
    }

    true
}

// --------------------------------------------------------------------
// cpu_monitoring_thread - Monitors for cpu consumption based on config
// --------------------------------------------------------------------
pub fn cpu_monitoring_thread(config: Arc<Mutex<ProcDumpConfiguration>>) -> u32
{

    0
}

// --------------------------------------------------------------------
// thread_monitoring_thread - Monitors for thread count  based on config
// --------------------------------------------------------------------
pub fn thread_monitoring_thread(config: Arc<Mutex<ProcDumpConfiguration>>) -> u32
{

    0
}

// --------------------------------------------------------------------
// file_monitoring_thread - Monitors for file desc count  based on config
// --------------------------------------------------------------------
pub fn file_monitoring_thread(config: Arc<Mutex<ProcDumpConfiguration>>) -> u32
{

    0
}

// --------------------------------------------------------------------
// signal_monitoring_thread - Monitors for signal based on config
// --------------------------------------------------------------------
pub fn signal_monitoring_thread(config: Arc<Mutex<ProcDumpConfiguration>>) -> u32
{

    0
}


// --------------------------------------------------------------------
// timer_monitoring_thread - Timer based monitor  based on config
// --------------------------------------------------------------------
pub fn timer_monitoring_thread(config: Arc<Mutex<ProcDumpConfiguration>>) -> u32
{
    let lock = config.lock().unwrap();
    let timeout = Duration::from_secs(lock.polling_frequency/1000);
    drop(lock);

    while should_continue_monitoring(&config)
    {
        let beginning_park = Instant::now();
        let mut timeout_remaining = timeout;

        park_timeout(timeout_remaining);
        let elapsed = beginning_park.elapsed();
        if elapsed >= timeout
        {
            //
            // Polling frequency has elapsed...generate a dump
            //
            {
                let lock = config.lock().unwrap();
                println!("Trigger: Timer:{}(s) on process ID: {}", lock.polling_frequency/1000, lock.process_id);
            }

            // Write Dump
        }
        else
        {
            //
            // Thread was unparked as a result of cancellation...exit
            //
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
    let polling_frequency = time::Duration::from_millis(lock.polling_frequency);
    let pid = lock.process_id;
    let string_pid: String = lock.process_id.to_string();
    let trigger_below = lock.trigger_threshold_mem_below;
    let trigger_threshold = lock.trigger_threshold_mem;
    drop(lock);

    // TODO: assume pagesize
    let pagesize = 4;

    loop {
        //
        // Read /proc/{pid}/stat file to get process statistics
        //
        let stat_path = format!("/proc/{}/stat", pid);
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

        if (trigger_below && mem_usage < trigger_threshold) ||
            (!trigger_below && mem_usage >= trigger_threshold) {

                //
                // Memory consumption trigger triggered
                //
                println!("Trigger: Memory usage:{}MB on process ID: {}", mem_usage, string_pid);

                //
                // Trigger dump gen...
                //
                Command::new("gcore").arg(string_pid).output().expect("Failed to execute gcore.");

                println!("Core dump generated.");
                break;
            }


        thread::sleep(polling_frequency);
    }

    0
}