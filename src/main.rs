mod sniff;
mod dashboard;
mod llm;
mod db;
mod parser;
mod detection;

use crossbeam_channel::{unbounded, Receiver};
use std::thread;
use sniff::NetworkEvent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use crate::db::NetworkDB;
use crate::detection::TrafficAnalyzer;
use tokio::time;

struct NetworkStats {
    total_packets: usize,
    total_bytes: usize,
    protocol_counts: HashMap<String, usize>,
}

impl NetworkStats {
    fn new() -> Self {
        NetworkStats {
            total_packets: 0,
            total_bytes: 0,
            protocol_counts: HashMap::new(),
        }
    }

    fn update(&mut self, event: &NetworkEvent) {
        self.total_packets += 1;
        self.total_bytes += event.payload_size;
        *self.protocol_counts.entry(event.protocol.clone()).or_insert(0) += 1;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = unbounded();
    let running = Arc::new(AtomicBool::new(true));
    let db = NetworkDB::new().await?;
    let analyzer = TrafficAnalyzer::new(db.get_database_instance()).await;
    println!("Connected to MongoDB successfully");

    let capture_thread = thread::spawn(move || {
        if let Err(e) = sniff::start_sniffing(Some("en0"), tx) {
            eprintln!("Packet capture error: {}", e);
        }
    });

    let db_clone = db.clone();
    let running_clone = running.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));
        while running_clone.load(Ordering::SeqCst) {
            interval.tick().await;
            if let Err(e) = db_clone.refresh_logs().await {
                eprintln!("Error clearing logs: {}", e);
            }
        }
    });

    let analyzer_clone = analyzer.clone();
    let running_clone = running.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));
        while running_clone.load(Ordering::SeqCst) {
            interval.tick().await;
            match analyzer_clone.detect_suspicious_traffic().await {
                Ok(suspicious) => {
                    for activity in suspicious {
                        println!("Suspicious Activity Detected: {} from {} - {}", activity.activity_type, activity.source, activity.details);
                        if let Err(e) = analyzer_clone.store_suspicious_event(activity).await {
                            eprintln!("Error inserting suspicious activity: {}", e);
                        }
                    }
                },
                Err(e) => eprintln!("Error detecting suspicious traffic: {}", e),
            }
        }
    });

    process_events(rx, running.clone(), db).await;
    capture_thread.join().unwrap();
    Ok(())
}

async fn process_events(rx: Receiver<NetworkEvent>, running: Arc<AtomicBool>, db: NetworkDB) {
    let mut stats = NetworkStats::new();
    while running.load(Ordering::SeqCst) {
        match rx.try_recv() {
            Ok(event) => {
                stats.update(&event);
                if let Err(e) = db.store_event(event.clone()).await {
                    eprintln!("Error storing event in MongoDB: {}", e);
                }
            },
            Err(_) => thread::sleep(Duration::from_millis(100)),
        }
    }
}
