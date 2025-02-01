// src/main.rs
mod sniff;
mod dashboard;
mod parser;
// mod llm;

use crossbeam_channel::{unbounded, Receiver};
use parser::Storage;
use std::thread;
use sniff::NetworkEvent;
use chrono::Local;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;

// Stats structure to track network statistics
struct NetworkStats {
    total_packets: usize,
    total_bytes: usize,
    protocol_counts: HashMap<String, usize>,
    unique_sources: HashMap<String, usize>,
    unique_destinations: HashMap<String, usize>,
}

impl NetworkStats {
    fn new() -> Self {
        NetworkStats {
            total_packets: 0,
            total_bytes: 0,
            protocol_counts: HashMap::new(),
            unique_sources: HashMap::new(),
            unique_destinations: HashMap::new(),
        }
    }

    fn update(&mut self, event: &NetworkEvent) {
        self.total_packets += 1;
        self.total_bytes += event.payload_size;
        *self.protocol_counts.entry(event.protocol.clone()).or_insert(0) += 1;
        *self.unique_sources.entry(event.source.clone()).or_insert(0) += 1;
        *self.unique_destinations.entry(event.destination.clone()).or_insert(0) += 1;
    }

    fn display(&self) {
        println!("\n=== Network Statistics ===");
        println!("Total Packets: {}", self.total_packets);
        println!("Total Bytes: {} ({:.2} MB)", self.total_bytes, self.total_bytes as f64 / 1_000_000.0);

        println!("\nProtocol Distribution:");
        for (proto, count) in &self.protocol_counts {
            println!("{}: {} packets", proto, count);
        }

        println!("\nTop 5 Sources:");
        let mut sources: Vec<_> = self.unique_sources.iter().collect();
        sources.sort_by(|a, b| b.1.cmp(a.1));
        for (src, count) in sources.iter().take(5) {
            println!("{}: {} packets", truncate(src, 40), count);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = unbounded();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        println!("\nShutting down...");
        r.store(false, Ordering::SeqCst);
    })?;

    println!("CONUHACKS::Starting network capture on interface en0...");
    println!("Press Ctrl+Z to stop and view statistics.\n");

    // Start capture thread
    let capture_thread = thread::spawn(move || {
        // If linux, wlp0s20f3 interface
        if let Err(e) = sniff::start_sniffing(Some("en0"), tx) {
            eprintln!("Packet capture error: {}", e);
        }
    });

    // Process events in main thread
    process_events(rx, running);

    capture_thread.join().unwrap();
    Ok(())
}

fn process_events(rx: Receiver<NetworkEvent>, running: Arc<AtomicBool>) {
    let mut stats = NetworkStats::new();
    let mut last_display = std::time::Instant::now();
    let display_interval = Duration::from_secs(1);

    // Display header
    println!("{:<12} {:<8} {:<30} {:<30} {:<10}",
        "Time", "Proto", "Source", "Destination", "Size");
    println!("{}", "=".repeat(92));

    while running.load(Ordering::SeqCst) {
        if let Ok(event) = rx.try_recv() {
            display_event(&event);
            stats.update(&event);

            // Update display periodically
            if last_display.elapsed() >= display_interval {
                clear_screen();
                stats.display();
                last_display = std::time::Instant::now();
            }
        } else {
            thread::sleep(Duration::from_millis(100));
        }
    }

    // Display final statistics
    clear_screen();
    stats.display();
}

fn display_event(event: &NetworkEvent) {
    println!(
        "{:<12} {:<8} {:<30} {:<30} {:<10}",
        Local::now().format("%H:%M:%S%.3f"),
        event.protocol,
        truncate(&event.source, 29),
        truncate(&event.destination, 29),

        format!("{} B", event.payload_size)

    );
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}