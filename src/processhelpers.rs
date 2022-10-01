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
        println!("{:?}", path);


    }


    true
}