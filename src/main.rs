use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpStream, UdpSocket};
use std::time::{Duration, Instant};

fn read_file_line_by_line(file_path: &str) -> io::Result<Vec<String>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut lines = Vec::new();
    for line_result in reader.lines() {
        let line = line_result?;
        lines.push(line);
    }

    Ok(lines)
}

fn print_progress(current: usize, total: usize) {
    let percentage = (current as f32 / total as f32) * 100.0;
    print!(
        "\rProgress: [{:50}] {}%",
        "=".repeat((percentage as usize) / 2),
        percentage as usize
    );
    io::stdout().flush().unwrap();
}

fn print_stats(total_events: usize, elapsed: Duration) {
    let elapsed_secs = elapsed.as_secs_f32();
    let avg_events_per_sec = if elapsed_secs > 0.0 {
        total_events as f32 / elapsed_secs
    } else {
        0.0
    };
    println!("Total events sent: {}", total_events);
    println!("Total time: {:.2} seconds", elapsed_secs);
    println!("Average events per second: {:.2}", avg_events_per_sec);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Error handling for command line arguments
    if args.len() < 4 {
        eprintln!("Usage: {} <file_path> <hostname> <port> <tcp/udp>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let hostname = &args[2];
    let port = &args[3];
    let protocol = &args[4].to_lowercase();
    let lines = read_file_line_by_line(file_path)?;
    let total_lines = lines.len();
    let mut lines_sent = 0;
    let start_time = Instant::now();
    let remote_host = format!("{}:{}", hostname, port);

    match protocol.as_str() {
        "tcp" => {
            let mut stream = TcpStream::connect(remote_host)?;
            for line in lines {
                writeln!(stream, "{}", line)?;
                lines_sent += 1;
                print_progress(lines_sent, total_lines);
            }
        }
        "udp" => {
            let local_address = format!("0.0.0.0:{}", 0);
            let socket = UdpSocket::bind(&local_address)?;
            for line in lines {
                socket.send_to(line.as_bytes(), &remote_host)?;
                lines_sent += 1;
                print_progress(lines_sent, total_lines);
            }
        }
        _ => {
            eprintln!("Invalid protocol specified: {}", protocol);
            std::process::exit(1);
        }
    }

    let elapsed_time = start_time.elapsed();
    println!();
    print_stats(total_lines, elapsed_time);

    Ok(())
}
