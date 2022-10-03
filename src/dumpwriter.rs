// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Helpers to write the dump file
//
//--------------------------------------------------------------------
extern crate chrono;
use chrono::Local;
use nix::libc::Elf32_Section;
use crate::procdumpconfiguration::ProcDumpConfiguration;
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::path::Path;
use std::str;

pub fn write_dump(config: &Arc<Mutex<ProcDumpConfiguration>>, trigger_type: &String) -> bool
{
    let mut lock = config.lock().unwrap();

    // Get current date
    let current = Local::now();
    let dump_date = current.format("%Y-%m-%d_%H:%M:%S").to_string();

    // Construct the dump prefix
    let mut gcore_prefix_name: String;
    if !lock.core_dump_name.is_empty()
    {
        gcore_prefix_name = format!("{}/{}_{}", lock.core_dump_path, lock.core_dump_name, lock.number_of_dumps_collected);
    }
    else
    {
        gcore_prefix_name = format!("{}/{}_{}_{}", lock.core_dump_path, lock.process_name, trigger_type, dump_date);
    }

    // Construct the dump file name
    let core_dump_file_name = format!("{}.{}", gcore_prefix_name, lock.process_id);
    let core_dump_name = gcore_prefix_name.clone();

    // Check if file already exists and if we have the overwrite flag set
    if Path::new(&core_dump_file_name).exists() && !lock.overwrite_existing_dump
    {
        println!("Dump file {} already exists and was not overwritten (use -o to overwrite)", core_dump_file_name);
        return false;
    }

    // Run gcore
    let gcore_res = Command::new("gcore").arg("-o").arg(gcore_prefix_name).arg(lock.process_id.to_string()).output().expect("Failed to execute gcore.");
    let gcore_stdout = gcore_res.stdout;
    let gcore_stderr = gcore_res.stderr;

    // If we failed, dump error
    if !gcore_res.status.success()
    {
        println!("Failed to generate dump");
        let res_stdout = str::from_utf8(&gcore_stderr);
        print!("GCORE - {}", res_stdout.unwrap());
        return false;
    }
    else
    {
        println!("Core dump {} generated: {}", lock.number_of_dumps_collected, core_dump_file_name);
        lock.number_of_dumps_collected += 1;
    }



    true
}