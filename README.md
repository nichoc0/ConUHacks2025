# Network Security Monitor

A local network security monitoring tool that analyzes your own network traffic for potential security threats like ARP poisoning and DNS spoofing.

## Overview

This tool monitors your machine's network traffic (non-promiscuous by default) and uses AI to detect suspicious patterns. It includes:

- Real-time packet analysis
- AI-powered threat detection
- Local MongoDB persistence with 30-minute sync intervals
- Web-based dashboard for monitoring (implemented using Streamlit)
- Optional promiscuous mode toggle (use responsibly)

## Architecture

![component-diagram](./system-diagrams/Network%20Security%20Monitor.png)

The system consists of:

- Rust backend with packet capture capabilities and API endpoints for event retrieval and report generation
- Python Streamlit dashboard for visualizing events (replacing the previous React frontend)
- MongoDB for persistent storage
- Integration with an LLM API for threat analysis
- Connection to threat databases for pattern matching

See `system-diagrams/component-diagram.txt` for detailed architecture visualization.

## Legal & Privacy

By default, this tool only monitors your own machine's traffic (non-promiscuous mode) to ensure legal compliance. The promiscuous mode toggle should only be used in environments where you have explicit permission to monitor network traffic.

## Installation & Usage

1. Start the Rust backend (which now exposes several API endpoints on port 8080).
2. Launch the Streamlit dashboard:
   • Install Streamlit (e.g., `pip install streamlit requests`)
   • Run the dashboard: `streamlit run streamlit_dashboard.py`
3. Use the dashboard to view network events and expand each event to see a placeholder report.
  
[Additional installation instructions remain the same...]
