use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::time::Instant;

fn resolve_target(hostname: &str, port: &str) -> io::Result<SocketAddr> {
    let target = format!("{}:{}", hostname, port);
    target
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to resolve target address"))
}

fn send_batch_tcp(stream: &mut TcpStream, batch: &[Vec<u8>]) -> io::Result<()> {
    // Build a single buffer to minimize syscalls while avoiding extra allocations per line.
    let estimated: usize = batch.iter().map(|l| l.len() + 1).sum();
    let mut buf = Vec::with_capacity(estimated);
    for line in batch {
        buf.extend_from_slice(line);
        buf.push(b'\n');
    }

    stream.write_all(&buf)
}

fn send_lines_tcp(file_path: &str, remote: SocketAddr) -> io::Result<usize> {
    const CHUNK: usize = 64;

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut stream = TcpStream::connect(remote)?;
    stream.set_nodelay(true)?;

    let mut batch: Vec<Vec<u8>> = Vec::with_capacity(CHUNK);
    let mut lines_sent = 0usize;

    for line in reader.lines() {
        let line = line?.into_bytes();
        batch.push(line);

        if batch.len() == CHUNK {
            send_batch_tcp(&mut stream, &batch)?;
            lines_sent += batch.len();
            batch.clear();
        }
    }

    if !batch.is_empty() {
        send_batch_tcp(&mut stream, &batch)?;
        lines_sent += batch.len();
    }

    stream.flush()?;
    Ok(lines_sent)
}

fn send_lines_udp(file_path: &str, remote: SocketAddr) -> io::Result<usize> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(remote)?;

    let mut buffer = Vec::with_capacity(1024);
    let mut lines_sent = 0usize;

    for line in reader.lines() {
        let line = line?;
        buffer.clear();
        buffer.extend_from_slice(line.as_bytes());
        buffer.push(b'\n');
        socket.send(&buffer)?;
        lines_sent += 1;
    }

    Ok(lines_sent)
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
    let remote_addr = resolve_target(hostname, port)?;

    let start_time = Instant::now();
    let total_lines = match protocol.as_str() {
        "tcp" => send_lines_tcp(file_path, remote_addr)?,
        "udp" => send_lines_udp(file_path, remote_addr)?,
        _ => {
            eprintln!("Invalid protocol specified: {}", protocol);
            std::process::exit(1);
        }
    };

    let elapsed_time = start_time.elapsed();
    println!("Total events sent: {}", total_lines);
    println!("Total time: {:.2} seconds", elapsed_time.as_secs_f32());
    println!(
        "Average events per second: {:.2}",
        total_lines as f32 / elapsed_time.as_secs_f32()
    );

    Ok(())
}
