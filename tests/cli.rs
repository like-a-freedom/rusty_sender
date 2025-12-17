use std::io::Write;
use std::net::UdpSocket;
use std::process::Command;
use std::time::Duration;
use tempfile::NamedTempFile;

#[test]
fn cli_usage_exits_nonzero() {
    let bin = env!("CARGO_BIN_EXE_rusty_sender");
    let status = Command::new(bin).status().expect("failed to run binary");
    assert!(!status.success());
}

#[test]
fn cli_udp_success() -> Result<(), Box<dyn std::error::Error>> {
    let bin = env!("CARGO_BIN_EXE_rusty_sender");

    let server = UdpSocket::bind("127.0.0.1:0")?;
    server.set_read_timeout(Some(Duration::from_millis(500)))?;
    let server_addr = server.local_addr()?;

    let mut tmp = NamedTempFile::new()?;
    write!(tmp, "l1\nl2\n")?;
    let path = tmp.path();

    let status = Command::new(bin)
        .env("BATCH_SIZE", "1")
        .args([
            path.to_str().unwrap(),
            "127.0.0.1",
            &server_addr.port().to_string(),
            "udp",
        ])
        .status()?;

    assert!(status.success());

    let mut buf = [0u8; 256];
    let mut total = Vec::new();
    for _ in 0..2 {
        if let Ok(n) = server.recv(&mut buf) {
            total.extend_from_slice(&buf[..n]);
        }
    }

    let lines: Vec<&[u8]> = total
        .split(|b| *b == b'\n')
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(lines.len(), 2);

    Ok(())
}
