# Smart Firewall Assistant

[![Crates.io](https://img.shields.io/crates/v/{{project-name}}.svg)](https://crates.io/crates/{{project-name}})
[![Docs.rs](https://docs.rs/{{project-name}}/badge.svg)](https://docs.rs/{{project-name}})
[![CI](https://github.com/{{gh-username}}/{{project-name}}/workflows/CI/badge.svg)](https://github.com/{{gh-username}}/{{project-name}}/actions)

A privacy-focused firewall with AI-powered insights to help you make informed decisions about network connections.

## Why This Exists

Ever used Little Snitch or similar firewalls and found yourself googling "Is it safe to block process X?" only to find vague or outdated answers? We've been there. While traditional firewalls tell you what's connecting where, they don't help you decide whether blocking is actually safe for your system.

### Key Features

- **Smart Connection Analysis**: Our LLM-powered assistant analyzes network connections and explains:

  - What the process likely does
  - Potential impacts of blocking it
  - Privacy implications of allowing it
  - Recommended action based on privacy/functionality balance
- **Traditional Firewall Controls**:

  - Block/allow connections on demand
  - Set rules for specific durations or permanently
  - Monitor all network activity
- **Privacy First**: All analysis happens locally on your device

## How It's Different

Traditional firewalls like Little Snitch show you process names and destinations, leaving you to research whether blocking is safe. Our assistant provides immediate, contextual guidance about the privacy/functionality tradeoffs of each connection.

## Why Rust?

Memory safety vulnerabilities account for 60-90% of discovered security issues in production code. We're building a security tool, so we need to practice what we preach. Rust's guarantees make our firewall more secure by design.

While choosing Rust adds some complexity (just like our mascot 🦀, we sometimes move sideways before going forward), we believe the industry is moving towards memory-safe languages. By building in Rust today, we're betting on - and contributing to - that safer future.

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install {{project-name}}`

## License

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
