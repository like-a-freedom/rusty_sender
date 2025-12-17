use std::hint::black_box as std_black_box;
use std::net::UdpSocket;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn udp_send_bench(c: &mut Criterion) {
    let receiver = UdpSocket::bind("127.0.0.1:0").expect("bind receiver");
    let sender = UdpSocket::bind("127.0.0.1:0").expect("bind sender");
    sender
        .connect(receiver.local_addr().expect("receiver addr"))
        .expect("connect sender");

    let payload = b"<134>Jan  1 00:00:00 host app: test event line";

    let mut group = c.benchmark_group("udp_send");
    group.throughput(Throughput::Bytes(payload.len() as u64));

    group.bench_function("send_single", |b| {
        b.iter(|| {
            let bytes = sender.send(std_black_box(payload)).expect("send");
            std_black_box(bytes);
        });
    });

    group.finish();
}

criterion_group!(benches, udp_send_bench);
criterion_main!(benches);
