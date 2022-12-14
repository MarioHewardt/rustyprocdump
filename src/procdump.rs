// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Manages the procdump configuration
//
//--------------------------------------------------------------------
mod procdumpconfiguration;
mod triggerthreadprocs;
mod monitor;
mod processhelpers;
mod dumpwriter;

//use std::thread;

// -----------------------------------------------------------------
// Main function
// -----------------------------------------------------------------
fn main()
{
    procdumpconfiguration::print_banner();
    procdumpconfiguration::init_procdump();

    // TODO: Check privilege warning

    // Parse cmd line
    let mut config =  Default::default();
    if procdumpconfiguration::get_options(&mut config) < 0
    {
        return;
    }

    // Start monitoring based on config
    monitor::monitor_processes(&mut config);
}

