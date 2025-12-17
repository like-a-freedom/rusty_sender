# What's this?

It's a very simple utility that read file line by line and then send the line to specified TCP or UDP port. Mainly, it is used to send events to the SIEM system for performance testing reasons.

## How to use

Syntax: `./rusty_sender <file_path> <hostname> <port> <tcp/udp>`

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

