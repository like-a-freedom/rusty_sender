use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const BATCH_SIZE: usize = 1000; // Adjust batch size as needed

fn read_file_line_by_line(file_path: &str) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(|line| line.ok()).collect();
    Ok(lines)
}

fn send_lines_tcp(lines: Vec<String>, remote_host: String) -> Result<(), std::io::Error> {
    let stream = TcpStream::connect(remote_host)?;
    let mut lines_sent = 0;

    for batch in lines.chunks(BATCH_SIZE) {
        let mut batch_str = String::new();
        for line in batch {
            batch_str.push_str(line);
            batch_str.push('\n');
        }
        writeln!(stream, "{}", batch_str)?;
        lines_sent += batch.len();
    }

    Ok(())
}

fn send_lines_udp(lines: Vec<String>, remote_host: String) -> Result<(), std::io::Error> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_nonblocking(true)?;

    let mut lines_sent = 0;

    for batch in lines.chunks(BATCH_SIZE) {
        let mut batch_str = String::new();
        for line in batch {
            batch_str.push_str(line);
            batch_str.push('\n');
        }
        socket.send_to(batch_str.as_bytes(), &remote_host)?;
        lines_sent += batch.len();
    }

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 5 {
        eprintln!("Usage: {} <file_path> <hostname> <port> <tcp/udp>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let hostname = &args[2];
    let port = &args[3];
    let protocol = args[4].to_lowercase();
    let remote_host = format!("{}:{}", hostname, port);

    let lines = read_file_line_by_line(file_path)?;
    let total_lines = lines.len();
    let start_time = Instant::now();

    match protocol.as_str() {
        "tcp" => {
            let lines_arc = Arc::new(Mutex::new(lines));
            let remote_host_tcp = remote_host.clone();
            let thread_handle = thread::spawn(move || {
                let lines = lines_arc.lock().unwrap();
                send_lines_tcp(lines.clone(), remote_host_tcp).unwrap();
            });
            thread_handle.join().unwrap();
        }
        "udp" => {
            let lines_arc = Arc::new(Mutex::new(lines));
            let remote_host_udp = remote_host.clone();
            let thread_handle = thread::spawn(move || {
                let lines = lines_arc.lock().unwrap();
                send_lines_udp(lines.clone(), remote_host_udp).unwrap();
            });
            thread_handle.join().unwrap();
        }
        _ => {
            eprintln!("Invalid protocol specified: {}", protocol);
            std::process::exit(1);
        }
    }

    let elapsed_time = start_time.elapsed();
    println!("Total events sent: {}", total_lines);
    println!("Total time: {:.2} seconds", elapsed_time.as_secs_f32());
    println!(
        "Average events per second: {:.2}",
        total_lines as f32 / elapsed_time.as_secs_f32()
    );

    Ok(())
}
