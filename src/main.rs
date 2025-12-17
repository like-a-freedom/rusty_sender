use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::time::Instant;

const DEFAULT_BATCH_SIZE: usize = 64;

fn batch_size_from_env() -> usize {
    std::env::var("BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_BATCH_SIZE)
}

fn batch_size_from_args(args: &[String]) -> Option<usize> {
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(val) = a.strip_prefix("--batch-size=") {
            return val.parse::<usize>().ok().filter(|&n| n > 0);
        }
        if a == "--batch-size" && i + 1 < args.len() {
            return args[i + 1].parse::<usize>().ok().filter(|&n| n > 0);
        }
        i += 1;
    }
    None
}

fn resolve_target(hostname: &str, port: &str) -> io::Result<SocketAddr> {
    let target = format!("{}:{}", hostname, port);
    target
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::other("failed to resolve target address"))
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

fn send_lines_tcp(file_path: &str, remote: SocketAddr, batch_size: usize) -> io::Result<usize> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut stream = TcpStream::connect(remote)?;
    stream.set_nodelay(true)?;

    let mut batch: Vec<Vec<u8>> = Vec::with_capacity(batch_size);
    let mut lines_sent = 0usize;

    for line in reader.lines() {
        let line = line?.into_bytes();
        batch.push(line);

        if batch.len() == batch_size {
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

    let batch_size = batch_size_from_env();
    let mut batch: Vec<Vec<u8>> = Vec::with_capacity(batch_size);
    let mut lines_sent = 0usize;

    for line in reader.lines() {
        let line = line?;
        batch.push(line.into_bytes());

        if batch.len() == batch_size {
            let estimated: usize = batch.iter().map(|l| l.len() + 1).sum();
            let mut buf = Vec::with_capacity(estimated);
            for l in &batch {
                buf.extend_from_slice(l);
                buf.push(b'\n');
            }
            socket.send(&buf)?;
            lines_sent += batch.len();
            batch.clear();
        }
    }

    if !batch.is_empty() {
        let estimated: usize = batch.iter().map(|l| l.len() + 1).sum();
        let mut buf = Vec::with_capacity(estimated);
        for l in &batch {
            buf.extend_from_slice(l);
            buf.push(b'\n');
        }
        socket.send(&buf)?;
        lines_sent += batch.len();
    }

    Ok(lines_sent)
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    // Minimal CLI parsing: support positional args plus an optional --batch-size flag
    if args.len() < 5 {
        eprintln!(
            "Usage: {} [--batch-size N] <file_path> <hostname> <port> <tcp/udp>",
            args[0]
        );
        std::process::exit(1);
    }

    let file_path = &args[1];
    let hostname = &args[2];
    let port = &args[3];
    let protocol = args[4].to_lowercase();
    let remote_addr = resolve_target(hostname, port)?;

    let start_time = Instant::now();
    let batch_size = batch_size_from_args(&args).unwrap_or_else(batch_size_from_env);
    eprintln!("Using batch size: {}", batch_size);

    let total_lines = match protocol.as_str() {
        "tcp" => send_lines_tcp(file_path, remote_addr, batch_size)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::{TcpListener, UdpSocket};
    use std::thread;
    use tempfile::NamedTempFile;

    #[test]
    fn test_batch_size_from_args_equals() {
        let args = vec!["prog".into(), "--batch-size=128".into()];
        assert_eq!(batch_size_from_args(&args), Some(128));
    }

    #[test]
    fn test_batch_size_from_args_space() {
        let args = vec!["prog".into(), "--batch-size".into(), "32".into()];
        assert_eq!(batch_size_from_args(&args), Some(32));
    }

    #[test]
    fn test_batch_size_from_args_invalid() {
        let args = vec!["prog".into(), "--batch-size".into(), "x".into()];
        assert_eq!(batch_size_from_args(&args), None);
    }

    use serial_test::serial;

    #[serial]
    fn test_batch_size_from_env() {
        unsafe { std::env::set_var("BATCH_SIZE", "16") };
        assert_eq!(batch_size_from_env(), 16);
        unsafe { std::env::remove_var("BATCH_SIZE") };
    }

    #[test]
    fn test_send_lines_tcp_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let mut tmp = NamedTempFile::new()?;
        write!(tmp, "a\nb\nc\n")?;
        let path = tmp.path().to_str().unwrap().to_string();

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;

        let handle = thread::spawn(move || {
            // Use batch size 2 to force batching behaviour
            send_lines_tcp(&path, addr, 2).unwrap()
        });

        let (mut stream, _) = listener.accept()?;
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf)?;

        let sent = handle.join().unwrap();
        // split by newline and discard empty trailing
        let lines: Vec<&[u8]> = buf.split(|b| *b == b'\n').filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), sent);
        assert_eq!(lines.len(), 3);
        Ok(())
    }

    #[serial]
    fn test_send_lines_udp_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let mut tmp = NamedTempFile::new()?;
        write!(tmp, "1\n2\n3\n4\n5\n")?;
        let path = tmp.path().to_str().unwrap().to_string();

        // server
        let server = UdpSocket::bind("127.0.0.1:0")?;
        server.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
        let server_addr = server.local_addr()?;

        // set small batch size via env
        unsafe { std::env::set_var("BATCH_SIZE", "2") };
        let sent = send_lines_udp(&path, server_addr)?;
        unsafe { std::env::remove_var("BATCH_SIZE") };

        let mut total_bytes = Vec::new();
        for _ in 0..(sent / 2 + 1) {
            let mut buf = [0u8; 2048];
            match server.recv(&mut buf) {
                Ok(n) => total_bytes.extend_from_slice(&buf[..n]),
                Err(_e) => {
                    // if timeout, break
                    break;
                }
            }
        }

        let lines: Vec<&[u8]> = total_bytes.split(|b| *b == b'\n').filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), sent);
        assert_eq!(sent, 5);
        Ok(())
    }
}
