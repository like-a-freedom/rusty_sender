# What's this?

It's a very simple utility that read file line by line and then send the line to specified TCP or UDP port. Mainly, it is used to send events to the SIEM system for performance testing reasons.

## How to use

Syntax: `./rusty_sender <file_path> <hostname> <port> <tcp/udp>`

Example: `./rusty_sender "/windows_events" "10.10.10.100" "5140" "udp"`
