mod procdumpconfiguration;
mod triggerthreadprocs;

use std::thread;

// -----------------------------------------------------------------
// Main function
// -----------------------------------------------------------------
fn main(){
    procdumpconfiguration::print_banner();

    // Parse cmd line
    let mut config =  procdumpconfiguration::ProcDumpConfiguration { mem_trigger_value: 0, mem_trigger_below_value: false, process_id: 0, polling_frequency: 1000 };
    procdumpconfiguration::get_options(&mut config);

    // Print the active triggers
    procdumpconfiguration::print_triggers(&config);

    // Now we need to spawn a thread that monitors the target process for memory consumption
    let handle = thread::spawn(move || triggerthreadprocs::mem_monitoring_thread(&config));
    handle.join().unwrap();
}

