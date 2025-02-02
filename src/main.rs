mod sniff;
mod dashboard;
mod db;
mod parser;
use crossbeam_channel::{unbounded, Receiver};
use std::thread;
use sniff::NetworkEvent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use crate::db::NetworkDB;
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
    println!("Connected to MongoDB successfully");

    let capture_thread = thread::spawn(move || {
        sniff::start_sniffing(Some("en0"), tx).unwrap();
    });

    let db_clone = db.clone();
    let running_clone = running.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));
        while running_clone.load(Ordering::SeqCst) {
            interval.tick().await;
            db_clone.refresh_logs().await.unwrap_or_else(|e| eprintln!("Error clearing logs: {}", e));
        }
    });

    process_events(rx, running.clone(), db).await;
    capture_thread.join().unwrap();
    Ok(())
}

async fn process_events(rx: Receiver<NetworkEvent>, running: Arc<AtomicBool>, db: NetworkDB) {
    let mut stats = NetworkStats::new();
    while running.load(Ordering::SeqCst) {
        if let Ok(event) = rx.try_recv() {
            stats.update(&event);
            if let Err(e) = db.store_event(event.clone()).await {
                eprintln!("Error storing event in MongoDB: {}", e);
            }
        } else {
            thread::sleep(Duration::from_millis(100));
        }
    }
}
