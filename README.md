# 🦀 Network Security Monitor

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.9+-blue?style=for-the-badge&logo=python)](https://www.python.org/)
[![MongoDB](https://img.shields.io/badge/MongoDB-Latest-green?style=for-the-badge&logo=mongodb)](https://www.mongodb.com/)
[![Streamlit](https://img.shields.io/badge/Streamlit-Latest-red?style=for-the-badge&logo=streamlit)](https://streamlit.io/)
[![LLM](https://img.shields.io/badge/LLM-Powered-pink?style=for-the-badge)](https://together.ai/)

</div>

## 🎯 Project Overview

A privacy-first network security monitoring tool that analyzes your local network traffic for potential security threats. Built with Rust for performance and reliability, featuring AI-powered threat detection while keeping your data private.

### Key Features

- **🔒 Privacy-First Design**: Only analyzes metadata, not packet contents
- **🚀 High-Performance**: Written in Rust for blazing-fast packet processing
- **🤖 AI-Powered Insights**: LLM-based threat analysis without exposing sensitive data
- **📊 Real-Time Dashboard**: Built with Streamlit and async MongoDB integration
- **🌐 Cross-Platform**: Works on Linux, macOS, and Windows
- **💻 On-Device Processing**: All analysis happens locally except for anonymized LLM queries

### Platform Showcase

<div align="center">

![Architecture Diagram](./system-diagrams/Network%20Security%20Monitor.png)

</div>

## 🏗️ Technical Achievements

- **Rust Implementation**: Learned and implemented core networking functionality in Rust
- **Async Streamlit**: Successfully integrated async MongoDB operations with Streamlit
- **Privacy-Preserving Design**: Developed a secure architecture that respects user privacy
- **Cross-Platform Networking**: Handled platform-specific network capture requirements
- **Real-Time Processing**: Efficient packet processing and analysis pipeline

## 🚀 Getting Started

```bash
# Clone the repository
git clone https://github.com/your-username/network-security-monitor

# Install Rust dependencies
cargo build

# Install Python dependencies
pip install -r requirements.txt

# Start MongoDB (required for event storage)
mongod

# Run the Rust backend
cargo run

# Launch the dashboard
streamlit run streamlit_dashboard.py
```

## 🔒 Legal & Privacy

This tool is designed for monitoring your own network traffic only. By default, it operates in non-promiscuous mode, capturing only your machine's traffic to ensure legal compliance. The promiscuous mode toggle should only be used in environments where you have explicit permission to monitor network traffic.

### Privacy Features

- Only metadata is analyzed, never packet contents
- All processing happens on your device
- LLM queries are anonymized and contain no identifying information
- Local MongoDB storage with configurable retention

## 🛠️ Technical Architecture

### Core Components

- **Packet Capture**: High-performance Rust-based packet sniffing
- **Threat Detection**: AI-powered pattern recognition
- **Event Storage**: Local MongoDB with async operations
- **Dashboard**: Streamlit interface with real-time updates

## 💡 Challenges & Learning

- **Rust Learning Curve**: Overcame memory safety and ownership concepts
- **Async Integration**: Successfully bridged Streamlit's sync nature with async MongoDB
- **Cross-Platform Support**: Handled different packet capture implementations
- **Privacy Design**: Balanced security insights with data privacy

## 📈 Future Development

- Implement additional threat detection patterns
- Add support for custom detection rules
- Enhance LLM-based analysis capabilities
- Expand visualization options in the dashboard

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

## 📜 License

This project is licensed under the GPL 3.0 License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

*Built with ❤️ during ConUHacks 2025*

</div>
