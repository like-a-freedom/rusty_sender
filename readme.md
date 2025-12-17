# What's this?

It's a very simple utility that read file line by line and then send the line to specified TCP or UDP port. Mainly, it is used to send events to the SIEM system for performance testing reasons.

## How to use

Syntax: `./rusty_sender [--batch-size N] <file_path> <hostname> <port> <tcp/udp>`

Note: `--batch-size` (command-line flag) overrides the `BATCH_SIZE` environment variable if both are provided.

Example: `./rusty_sender "/windows_events" "10.10.10.100" "5140" "udp"`

## Environment configuration

You can control the batch size (number of events grouped into a single network send) using the
`BATCH_SIZE` environment variable. Default is `64`.

Examples:

- Run with default batch size (64):
	`./rusty_sender "/windows_events" "10.10.10.100" "5140" "udp"`
- Run with a larger batch size (128):
	`BATCH_SIZE=128 ./rusty_sender "/windows_events" "10.10.10.100" "5140" "udp"`

Notes:
- Increasing the batch size reduces syscall overhead and can increase EPS, but may increase latency
	and the size of UDP datagrams (they may become fragmented if larger than MTU). Tune for your
	environment.

## Event files

Where to place files
- Event files can be located anywhere the process has read access; pass the path as the first
  positional argument to the program (for example, `./rusty_sender ./mocks/windows_events ...`).
- This repository includes sample event files under `example_data/` which are good
  starting points for tests and benchmarking.

Expected format
- Files must be plain text (UTF-8 recommended) with **one event per line**. Each line is treated
  as a single event and will be sent exactly as read (the program appends a newline when batching).
- Lines must not contain embedded newlines — an event must fit entirely on a single line.
- Line endings (LF or CRLF) are supported; lines are read using standard text line semantics.

Practical limits & tips
- For UDP: keep individual datagram payloads under your MTU to avoid fragmentation; as a rule of
  thumb keep the *total batched payload* under ~1400 bytes or tune `--batch-size` to smaller values.
- For TCP: there is no strict per-packet size limitation, but very large batches increase latency and
  memory use.
- Keep individual lines reasonably sized (e.g., < 1 KB) for best throughput and lower memory
  pressure.
- Compressed files are not supported; provide uncompressed text files.

Example (two-line file):
```
2025-12-17T12:34:56Z host1.example.com EVENT_TYPE=LOGIN user=jdoe src_ip=10.0.0.1
2025-12-17T12:34:57Z host1.example.com EVENT_TYPE=LOGOUT user=jdoe src_ip=10.0.0.1
```


