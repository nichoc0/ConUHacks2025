# Network Security Monitor

A local network security monitoring tool that analyzes your own network traffic for potential security threats like ARP poisoning and DNS spoofing.

## Overview

This tool monitors your machine's network traffic (non-promiscuous by default) and uses AI to detect suspicious patterns. It includes:

- Real-time packet analysis
- AI-powered threat detection
- Local MongoDB persistence with 30-minute sync intervals
- Web interface for monitoring
- Optional promiscuous mode toggle (use responsibly)

## Architecture

![component-diagram](./system-diagrams/Network%20Security%20Monitor.png)

The system consists of:

- Rust backend with packet capture capabilities
- React frontend for visualization
- MongoDB for persistent storage
- Integration with LLM API for threat analysis
- Connection to threat databases for pattern matching

See `system-diagrams/component-diagram.txt` for detailed architecture visualization.

## Legal & Privacy

By default, this tool only monitors your own machine's traffic (non-promiscuous mode) to ensure legal compliance. The promiscuous mode toggle should only be used in environments where you have explicit permission to monitor network traffic.

## Installation & Usage

[Previous installation instructions remain the same...]
