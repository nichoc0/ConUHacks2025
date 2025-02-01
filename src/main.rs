// src/main.rs
mod sniff;
mod dashboard;
mod parser;
// mod llm;

mod db;
use crossbeam_channel::{unbounded, Receiver};
use parser::Storage;
use std::thread;
use sniff::NetworkEvent;
use chrono::{Local, TimeZone};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use crate::db::NetworkDB;


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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = unbounded();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Try to connect to MongoDB
    let db = NetworkDB::new().await;
    let mongodb_available = db.is_ok();
    
    if mongodb_available {
        println!("Connected to MongoDB successfully");
    } else {
        println!("MongoDB connection failed - falling back to terminal display only");
    }

    ctrlc::set_handler(move || {
        println!("\nShutting down...");
        r.store(false, Ordering::SeqCst);
    })?;

    println!("CONUHACKS::Starting network capture on interface en0...");
    println!("Press Ctrl+Z to stop and view statistics.\n");
    println!("Starting network capture on interface en0...");
    println!("Press Ctrl+C to stop and view statistics.\n");

    let capture_thread = thread::spawn(move || {
        // If linux, wlp0s20f3 interface
        if let Err(e) = sniff::start_sniffing(Some("en0"), tx) {
            eprintln!("Packet capture error: {}", e);
        }
    });

    // Only start MongoDB-related tasks if available
    let detection_handle = if mongodb_available {
        let db_for_detection = db.as_ref().unwrap().clone();
        let detection_running = running.clone();
        Some(tokio::spawn(async move {
            while detection_running.load(Ordering::SeqCst) {
                if let Ok(suspicious) = db_for_detection.detect_suspicious_traffic().await {
                    if !suspicious.is_empty() {
                        println!("\n=== Suspicious Activity Detected ===");
                        for activity in suspicious {
                            println!("Type: {}", activity.activity_type);
                            println!("Source: {}", activity.source);
                            println!("Details: {}", activity.details);
                            println!("Timestamp: {}", 
                                Local.timestamp_opt(activity.timestamp as i64, 0)
                                    .unwrap()
                                    .format("%Y-%m-%d %H:%M:%S"));
                            println!("---");
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        }))
    } else {
        None
    };

    process_events(rx, running, db.ok()).await;

    capture_thread.join().unwrap();
    if let Some(handle) = detection_handle {
        handle.await?;
    }
    Ok(())
}

async fn process_events(rx: Receiver<NetworkEvent>, running: Arc<AtomicBool>, db: Option<NetworkDB>) {
    let mut stats = NetworkStats::new();
    let mut last_display = std::time::Instant::now();
    let display_interval = Duration::from_secs(1);

    // Display header
    println!("{:<12} {:<8} {:<30} {:<30} {:<10}",

        "Time", "Proto", "Source", "Destination", "Size");
    println!("{}", "=".repeat(92));

    while running.load(Ordering::SeqCst) {
        if let Ok(event) = rx.try_recv() {
            // Store in MongoDB if available
            if let Some(db_instance) = &db {
                if let Err(e) = db_instance.store_event(event.clone()).await {
                    eprintln!("Error storing event in MongoDB: {}", e);
                }
            }

            display_event(&event);
            stats.update(&event);

            if last_display.elapsed() >= display_interval {
                clear_screen();
                stats.display();
                last_display = std::time::Instant::now();
            }
        } else {
            thread::sleep(Duration::from_millis(100));
        }
    }

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
    print!("\x1B[2J\x1B[1H");
}