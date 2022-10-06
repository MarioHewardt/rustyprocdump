// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License

//--------------------------------------------------------------------
//
// Helpers to write the dump file
//
//--------------------------------------------------------------------
extern crate chrono;

use chrono::Local;
use sysinfo::NetworkExt;
use std::os::unix::net::{UnixListener, UnixStream};
use crate::procdumpconfiguration::ProcDumpConfiguration;
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::path::Path;
use std::str;
use std::env;
use std::io::{Read, Write};
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use core::mem::*;

struct Diagnostics_Header
{
    header_string: Vec<u8>,                     // "DOTNET_IPC_V1"
    packet_size : u16,
    req_f1 : u8,
    req_f2 : u8,
    req_f3 : u16,
    dump_file_len: u32,
    dump_file: Vec<u16>,
    dump_type: u32,                             // dump type
    diagnostics: u32,
}

//--------------------------------------------------------------------
//
// write_dump - writes a core dump based on config
//
//-------------------------------------------------------------------
pub fn write_dump(config: &Arc<Mutex<ProcDumpConfiguration>>, trigger_type: &String) -> bool
{
    let mut lock = config.lock().unwrap();

    // Get current date
    let current = Local::now();
    let dump_date = current.format("%Y-%m-%d_%H:%M:%S").to_string();

    // Construct the dump prefix
    let gcore_prefix_name: String;
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
    //let core_dump_name = gcore_prefix_name.clone();

    // Check if file already exists and if we have the overwrite flag set
    if Path::new(&core_dump_file_name).exists() && !lock.overwrite_existing_dump
    {
        println!("Dump file {} already exists and was not overwritten (use -o to overwrite)", core_dump_file_name);
        return false;
    }


    // Check if the target process is a CLR process
    drop(lock);
    let socket_name = get_clr_socket_name(config);
    lock = config.lock().unwrap();
    if !socket_name.is_empty()
    {
        // Target process is a CLR process, use the CLR runtime to dump...
        if generate_core_clr_dump(&socket_name, &core_dump_file_name)
        {
            // Success
        }
        else
        {
            // Failure
        }


    }
    else
    {
        // Target process is NOT a CLR process, use gcore to dump...
        let gcore_res = Command::new("gcore").arg("-o").arg(gcore_prefix_name).arg(lock.process_id.to_string()).output().expect("Failed to execute gcore.");
        //let gcore_stdout = gcore_res.stdout;
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
    }

    true
}

//--------------------------------------------------------------------
// get_clr_socket_name - Return the socket name of the CLR target
// process or "" if no socket was found (i.e., target process was
// not a CLR process)
//-------------------------------------------------------------------
pub fn get_clr_socket_name(config: &Arc<Mutex<ProcDumpConfiguration>>) -> String
{
    let mut lock = config.lock().unwrap();

    // If $TMPDIR is set, use it as the path, otherwise we use /tmp
    // per https://github.com/dotnet/diagnostics/blob/master/documentation/design-docs/ipc-protocol.md
    let mut tmp_folder = String::new();
    let tmp_dir = env::var("TMPDIR").is_ok();
    if !tmp_dir
    {
        tmp_folder = format!("/tmp/dotnet-diagnostic-{}", lock.process_id);
    }
    else
    {
        tmp_folder = env::var("TMPDIR").unwrap();
    }

    // Enumerate all open domain sockets exposed from the process. If one
    // exists by the following prefix, we assume its a .NET Core process:
    //    dotnet-diagnostic-{%d:PID}
    // The sockets are found in /proc/net/unix
    if let Ok(lines) = read_lines("/proc/net/unix")
    {
        let mut i:bool = true;
        for line in lines
        {
            if let Ok(ip) = line
            {
                // Skip first line
                if i==true
                {
                    i=false;
                    continue;
                }

                // example of /proc/net/unix line:
                // 0000000000000000: 00000003 00000000 00000000 0001 03 20287 @/tmp/.X11-unix/X0
                if ip.split(" ").nth(7).is_some()
                {
                    let sock: &str = ip.split(" ").nth(7).unwrap();
                    if sock.contains(&tmp_folder)
                    {
                        return sock.to_owned();
                    }
                }
            }
        }
    }

    String::new()
}

//--------------------------------------------------------------------
// generate_core_clr_dump - Generates a CLR dump using the CLR runtime
// based on specified socket name and dump file name
//-------------------------------------------------------------------
pub fn generate_core_clr_dump(socket_name: &String, dump_file_name: &String) -> bool
{
    let mut payload_size = 0;
    let mut header_size = 0;
    let mut header_string = String::new();
    header_string.push_str("DOTNET_IPC_V1");

    // Figure out packet size
    let dump_file_name_w: Vec<u16> = dump_file_name.encode_utf16().collect();
    payload_size += ((dump_file_name_w.len()+1)) * size_of::<char>();                // Dump file name (+1 for null terminator)
    payload_size += size_of::<i32>();                                           // Dump file name size
    payload_size += size_of::<i32>();                                           // Dump file type
    payload_size += size_of::<i32>();                                           // Flags

    header_size += header_string.len()+1;                                      // Header::Header (+1 for null terminator)
    header_size += size_of::<u16>();                                           // packet size
    header_size += size_of::<u8>();                                            // header field
    header_size += size_of::<u8>();                                            // header field
    header_size += size_of::<u16>();                                           // header field

    let packet_size: u16 = (payload_size + header_size) as u16;

/*/    // Construct packet
    let mut dump_req = Diagnostics_Header::default();

    for b in header_string.as_bytes()
    {
        dump_req.header_string.push(b.clone());
    }

    dump_req.header_string.push(0);     // NULL terminator

    dump_req.packet_size = packet_size as u16;      <////
    dump_req.req_f1 = 1;
    dump_req.req_f2 = 1;
    dump_req.req_f3 = 0;
    dump_req.dump_file_len = dump_file_name_w.len() as u32;
    dump_req.dump_file = dump_file_name_w.clone();
    dump_req.dump_file.push(0);         // NULL terminator
    dump_req.dump_type = 4;                                         // FULL DUMP
    dump_req.diagnostics = 0;
*/
    // ----------------------------------------------------------------------------

    let mut packet_bytes: Vec<u8> = Vec::new();
    for b in header_string.as_bytes()
    {
        packet_bytes.push(b.clone());
    }
    packet_bytes.push(0);   // NULL terminator

    for b in packet_size.to_be_bytes()
    {
        packet_bytes.push(b);
    }

    packet_bytes.push(1 as u8);
    packet_bytes.push(1 as u8);

    let f3 = 0 as u16;
    for b in f3.to_be_bytes()
    {
        packet_bytes.push(b);
    }

    let l = (dump_file_name_w.len()+1) as u32;
    for b in l.to_be_bytes()
    {
        packet_bytes.push(b);
    }

    for b in dump_file_name_w
    {
        for e in b.to_le_bytes()
        {
            packet_bytes.push(e);
        }
    }
    packet_bytes.push(0);   // NULL terminator

    let typ: i32 = 4;
    for b in typ.to_le_bytes()
    {
        packet_bytes.push(b);
    }

    packet_bytes.push(0 as u8);

    // ----------------------------------------------------------------------------


    //let bytes = bincode::serialize(&dump_req).unwrap();


    // Send packet to socket
    let mut unix_stream = UnixStream::connect(socket_name).expect("Could not create stream");
    let b: &[u8] = &packet_bytes;
    unix_stream.write(b).expect("Failed to write to UNIX socket");

    // Read response: Header
    let mut buf: Vec<u8> = Vec::new();
    buf.resize(header_size, 0);
    let mb:&mut [u8] = &mut buf;
    unix_stream.shutdown(std::net::Shutdown::Write).expect("Could not shutdown writing on the stream");
    unix_stream.read_exact(mb).expect("Failed reading header in response");
    if !mb.len() == header_size
    {
        println!("Invalid response.")
    }

    // Read response: code
    let mut response_code:Vec<u8> = Vec::new();
    response_code.resize(size_of::<i32>(), 0);
    let mr:&mut [u8] = &mut response_code;
    unix_stream.read_exact(mr).expect("Failed reading status code in response");

    let code = i32::from_le_bytes(mr.try_into().unwrap());
    if code != 0
    {
        return false;
    }

    true
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}


//--------------------------------------------------------------------
// impl Default for Diagnostics_Header
// Default trait for the diagnostics packet
//-------------------------------------------------------------------
impl Default for Diagnostics_Header
{
    fn default() -> Diagnostics_Header
    {
        Diagnostics_Header
        {
            header_string: Vec::new(),
            packet_size: 0,
            req_f1: 0,
            req_f2: 0,
            req_f3: 0,
            dump_file_len: 0,
            dump_file: Vec::new(),
            dump_type: 0,
            diagnostics: 0,
        }
    }
}