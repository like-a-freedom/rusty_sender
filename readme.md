# rusty_sender ⚡

Small, fast utility to send event files to a remote TCP or UDP endpoint (useful for SIEM/load testing).

---

## 📋 Table of Contents

- [About](#about)
- [Key features](#key-features)
- [Quick start](#quick-start)
- [Usage](#usage)
- [Event files](#event-files)
- [Configuration](#configuration)
- [Development & tests](#development--tests)
- [CI / Releases](#ci--releases)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

---

## About

`rusty_sender` reads a text file line-by-line and sends each line as an event to a specified TCP or UDP address. It is intentionally minimal and focused on throughput and predictability for benchmarking or ingestion testing.

## Key features

- Small single-binary tool written in Rust
- TCP and UDP sending modes
- Batched sends to reduce syscall overhead
- Configurable batch size via `--batch-size` or `BATCH_SIZE` env var
- Includes sample event files for testing/benchmarks (`example_data/`, `mocks/`)

## Quick start

Prerequisites:
- Rust toolchain (for building from source) or a release binary from GitHub releases

Build and run locally:

```bash
# build
cargo build --release

# run (UDP example)
./target/release/rusty_sender --batch-size 128 ./example_data/other_events 10.10.10.100 5140 udp
```

## Usage

Syntax:

```text
./rusty_sender [--batch-size N] <file_path> <hostname> <port> <tcp/udp>
```

Notes:
- `--batch-size N` (CLI) **overrides** `BATCH_SIZE` environment variable if both are specified.
- Example:

```bash
# use CLI flag
./rusty_sender --batch-size=128 ./example_data/other_events 10.10.10.100 5140 udp

# use environment variable
BATCH_SIZE=128 ./rusty_sender ./example_data/other_events 10.10.10.100 5140 udp
```

## Event files

### Where to place files
- Event files may live anywhere the process can read from — provide the path as the first positional argument.
- Example file locations in this repo:
  - `example_data/` — compact example datasets
  - `mocks/` — additional test event sets

### Format requirements
- Plain text (UTF-8 recommended).
- **One event per line**. Each line is treated as a single event; the program appends a newline when batching.
- Lines must not contain embedded newlines — each event must fit on a single line.
- Line endings LF or CRLF are supported.

### Practical tips and limits
- **UDP:** keep total batched payload < MTU (~1400 bytes) to avoid fragmentation; reduce `--batch-size` as needed.
- **TCP:** more forgiving for payload size, but very large batches increase latency and memory usage.
- Keep individual lines reasonably sized (e.g., < 1 KB) to reduce memory pressure and improve throughput.
- Provide uncompressed text files; compressed files are not supported.

Example file contents:

```
2025-12-17T12:34:56Z host1.example.com EVENT_TYPE=LOGIN user=jdoe src_ip=10.0.0.1
2025-12-17T12:34:57Z host1.example.com EVENT_TYPE=LOGOUT user=jdoe src_ip=10.0.0.1
```

## Configuration

- `--batch-size N` — command-line option that sets the number of lines grouped into a single send.
- `BATCH_SIZE` — environment variable (used when `--batch-size` is not provided).
- Default batch size: `64`.

## Development & tests

- Build: `cargo build --release`
- Run unit/bench tests: `cargo test` and `cargo bench` (benchmarks use `benches/eps.rs` with various batch sizes)
- Lint: `cargo clippy`

## CI / Releases

This repository uses GitHub Actions for build and release automation. Release workflow builds platform-specific artifacts for Linux, macOS and Windows and uploads them to GitHub Releases.

## Contributing

Contributions are welcome — open an issue or a pull request. Keep changes small, include tests where applicable and ensure the build passes (`cargo test`).

### Development guidelines
- Follow project conventions and coding style
- Add tests for new logic where possible
- Update the README for user-facing changes

## License

Distributed under the MIT License. See `LICENSE` for details.

## Contact

Maintainer: [@like-a-freedom](https://github.com/like-a-freedom)

---

Made with ❤️ — lightweight and purposeful.

