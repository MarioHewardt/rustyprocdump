// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Helpers for retrieving process stats
//
//--------------------------------------------------------------------
use std::{fs};

//--------------------------------------------------------------------
//
// get_process_name_by_pid - returns the pid of the specified process
// , i32::MAX otherwise.
//
//--------------------------------------------------------------------
pub fn get_process_name_by_pid(pid: i32) -> String
{
    let mut cmd_path = String::new();
    cmd_path.push_str("/proc/");
    cmd_path.push_str(&pid.to_string());
    cmd_path.push_str("/cmdline");

    let cmd_line = std::fs::read_to_string(&cmd_path);
    if !cmd_line.is_err()
    {
        let line = cmd_line.unwrap();
        if !line.is_empty()
        {
            let raw_line = line.replace(char::from(0), " ");

            // get the image name
            let mut image_name = raw_line.split(" ").nth(0).unwrap();
            if image_name.eq("sudo")
            {
                image_name = raw_line.split(" ").nth(1).unwrap();
            }

            return image_name.split('/').last().unwrap().to_string();
        }
    }

    String::from("")
}


//--------------------------------------------------------------------
//
// look_up_process_pid_by_name - returns pid if the specified process
// is running, i32::MAX otherwise
//
//--------------------------------------------------------------------
pub fn get_process_pid_by_name(process_name: &String) -> i32
{
    // TODO: Get rid of all expect since it panics.
    for entry in fs::read_dir("/proc/").expect("I told you this directory exists")
    {
        let entry = entry.expect("I couldn't read something inside the directory");
        let path = entry.path();

        let pid = path.file_name().unwrap().to_str().unwrap().to_lowercase();

        // If we can't convert the read pid to i32 its not a pid, move on
        let _ = match pid.parse::<i32>()
        {
            Ok(pid) => pid,
            Err(_err) => { continue; },
        };

        let process_name_found = get_process_name_by_pid(pid.parse::<i32>().unwrap());
        if process_name_found.eq(process_name)
        {
            return pid.parse::<i32>().unwrap();
        }
    }

    i32::MAX
}


//--------------------------------------------------------------------
//
// get_process_pgid - returns pgid of the specified process
//--------------------------------------------------------------------
pub fn get_process_pgid(pid: i32) -> u64
{
    let stat_path = format!("/proc/{}/stat", pid);
    let statcontents = match fs::read_to_string(stat_path)
    {
        Ok(stat) => stat,
        Err(_) => return u64::MAX,
    };

    let pgid = statcontents.split(" ").nth(4).unwrap().parse::<u64>().unwrap();

    return pgid;
}

//--------------------------------------------------------------------
//
// get_process_start_time - returns pgid of the specified process
//--------------------------------------------------------------------
pub fn get_process_start_time(pid: i32) -> u64
{
    let stat_path = format!("/proc/{}/stat", pid);
    let statcontents = match fs::read_to_string(stat_path)
    {
        Ok(contents) => contents,
        Err(_) => return u64::MAX,
    };

    let start_time = statcontents.split(" ").nth(21).unwrap().parse::<u64>().unwrap();

    return start_time;
}

//--------------------------------------------------------------------
//
// is_process_running - returns true if the specified process is
// running, false otherwise.
//--------------------------------------------------------------------
pub fn is_process_running(pid: i32) -> bool
{
    let stat_path = format!("/proc/{}/stat", pid);
    let _ = match fs::read_to_string(stat_path)
    {
        Ok(_) => return true,
        Err(_) => return false,
    };
}