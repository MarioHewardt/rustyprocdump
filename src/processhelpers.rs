// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Helpers for retrieving process stats
//
//--------------------------------------------------------------------
use std::fs;

pub fn get_process_name(process_name: &String) -> bool
{
    // TODO: Get rid of all expect since it panics.
    for entry in fs::read_dir("/proc/").expect("I told you this directory exists")
    {
        let entry = entry.expect("I couldn't read something inside the directory");
        let path = entry.path();
        println!("{:?}", entry.file_name());

        let statcontents = fs::read_to_string(path).expect("Stat file not found.");
        let mut pid = statcontents.split(" ").nth(23).unwrap().parse::<u32>().unwrap();
    }


    true
}