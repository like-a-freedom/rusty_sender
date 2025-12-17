use std::hint::black_box as std_black_box;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::thread;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn build_batch(payload: &[u8], batch_size: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity((payload.len() + 1) * batch_size);
    for _ in 0..batch_size {
        buf.extend_from_slice(payload);
        buf.push(b'\n');
    }
    buf
}

fn udp_batch_bench(c: &mut Criterion) {
    let receiver = UdpSocket::bind("127.0.0.1:0").expect("bind receiver");
    let sender = UdpSocket::bind("127.0.0.1:0").expect("bind sender");
    sender
        .connect(receiver.local_addr().expect("receiver addr"))
        .expect("connect sender");

    let payload = b"<134>Jan  1 00:00:00 host app: test event line";

    let mut group = c.benchmark_group("udp_send_batch");

    for &batch_size in &[1usize, 8, 32, 64, 128] {
        let batch = build_batch(payload, batch_size);
        group.throughput(Throughput::Bytes(batch.len() as u64));
        group.bench_function(format!("udp_batch_{}", batch_size), |b| {
            b.iter(|| {
                let bytes = sender.send(std_black_box(&batch)).expect("send");
                std_black_box(bytes);
            })
        });
    }

    group.finish();
}

fn tcp_drain_loop(mut stream: TcpStream) {
    let mut buf = [0u8; 64 * 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(_) => continue,
            Err(_) => break,
        }
    }
}

fn tcp_batch_bench(c: &mut Criterion) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    let addr = listener.local_addr().expect("local addr");

    // Accept loop: accept multiple connections and spawn a drain thread for each
    thread::spawn(move || {
        while let Ok((stream, _)) = listener.accept() {
            thread::spawn(move || tcp_drain_loop(stream));
        }
    });

    let mut group = c.benchmark_group("tcp_send_batch");
    let payload = b"<134>Jan  1 00:00:00 host app: test event line";

    for &batch_size in &[1usize, 8, 32, 64, 128] {
        let batch = build_batch(payload, batch_size);
        group.throughput(Throughput::Bytes(batch.len() as u64));

        group.bench_function(format!("tcp_batch_{}", batch_size), |b| {
            let mut stream = TcpStream::connect(addr).expect("connect");
            b.iter(|| {
                let written = stream.write(std_black_box(&batch)).expect("write");
                std_black_box(written);
            })
        });
    }

    group.finish();
}

criterion_group!(benches, udp_batch_bench, tcp_batch_bench);
criterion_main!(benches);
