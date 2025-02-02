use mongodb::{bson::{doc, Bson}, Collection};
use futures::StreamExt;
use std::error::Error;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::sniff::NetworkEvent;
use crate::llm;

// Detection thresholds
const PORT_SCAN_THRESHOLD: i32 = 15;
const DNS_FLOOD_THRESHOLD: i32 = 300;
const LARGE_TRANSFER_THRESHOLD: i64 = 10_000_000; // 10MB
const RARE_PORT_THRESHOLD: i32 = 5;
const ARP_SPOOF_THRESHOLD: i32 = 3;

#[derive(Debug, Serialize)]
pub struct SuspiciousActivity {
    pub activity_type: String,
    pub source: String,
    pub details: String,
    pub timestamp: f64,
    llm: ()
}

#[derive(Clone)]
pub struct TrafficAnalyzer {
    tcp_collection: Collection<NetworkEvent>,
    udp_collection: Collection<NetworkEvent>,
    dns_collection: Collection<NetworkEvent>,
    arp_collection: Collection<NetworkEvent>,
    suspicious_collection: Collection<SuspiciousActivity>,
}

impl TrafficAnalyzer {
    pub async fn new(db: mongodb::Database) -> Self {
        Self {
            tcp_collection: db.collection("tcp_events"),
            udp_collection: db.collection("udp_events"),
            dns_collection: db.collection("dns_events"),
            arp_collection: db.collection("arp_events"),
            suspicious_collection: db.collection("sus_events"),
        }
    }

    pub async fn detect_suspicious_traffic(&self) -> Result<Vec<SuspiciousActivity>, Box<dyn Error + Send + Sync>> {
        let mut suspicious_activities = Vec::new();

        self.detect_port_scanning(&mut suspicious_activities).await?;
        self.detect_large_transfers(&mut suspicious_activities).await?;
        self.detect_dns_flood(&mut suspicious_activities).await?;
        self.detect_rare_ports(&mut suspicious_activities).await?;
        self.detect_arp_spoofing(&mut suspicious_activities).await?;
        self.detect_udp_floods(&mut suspicious_activities).await?;

        Ok(suspicious_activities)
    }

    async fn detect_port_scanning(&self, activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let time_window = get_current_timestamp() - 300.0; // 5 minutes

        let pipeline = vec![
            doc! { "$match": {
                "timestamp": { "$gte": time_window },
                "protocol": "TCP"
            }},
            doc! { "$addFields": {
                "dest_ip": { "$arrayElemAt": [{ "$split": ["$destination", ":"] }, 0] },
                "dest_port": { "$toInt": { "$arrayElemAt": [{ "$split": ["$destination", ":"] }, 1] } }
            }},
            doc! { "$group": {
                "_id": { "source": "$source", "dest_ip": "$dest_ip" },
                "unique_ports": { "$addToSet": "$dest_port" },
                "total_attempts": { "$sum": 1 }
            }},
            doc! { "$match": {
                "$expr": { "$gt": [{ "$size": "$unique_ports" }, PORT_SCAN_THRESHOLD] }
            }}
        ];

        let mut cursor = self.tcp_collection.aggregate(pipeline).await?;

        while let Some(Ok(doc)) = cursor.next().await {
            let id = doc.get_document("_id")?;
            let source = id.get_str("source")?;
            let dest_ip = id.get_str("dest_ip")?;
            let port_count = doc.get_array("unique_ports")?.len();
            let serialized: String = serde_json::to_string(&activities).unwrap_or_default();



            activities.push(SuspiciousActivity {
                activity_type: "Port Scanning".into(),
                source: source.into(),
                details: format!("{} unique ports scanned on {}", port_count, dest_ip),
                timestamp: get_current_timestamp(),
                llm: llm::run(serialized.clone())  // Convert llm result to String
            });
        }
        Ok(())
    }

    async fn detect_large_transfers(&self, activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let time_window = get_current_timestamp() - 300.0;

        let pipeline = vec![
            doc! { "$match": { "timestamp": { "$gte": time_window } } },
            doc! { "$group": {
                "_id": "$source",
                "total_bytes": { "$sum": "$payload_size" }
            }},
            doc! { "$match": {
                "total_bytes": { "$gt": LARGE_TRANSFER_THRESHOLD }
            }}
        ];

        let mut cursor = self.tcp_collection.aggregate(pipeline).await?;
        let serialized: String = serde_json::to_string(&activities).unwrap_or_default();


        while let Some(Ok(doc)) = cursor.next().await {
            let source = doc.get_str("_id")?;
            let bytes = doc.get_i64("total_bytes")?;

            activities.push(SuspiciousActivity {
                activity_type: "Large Data Transfer".into(),
                source: source.into(),
                details: format!("{} bytes transferred in 5 minutes", bytes),
                timestamp: get_current_timestamp(),
                llm: llm::run(serialized.clone())

            });
        }
        Ok(())
    }

    async fn detect_dns_flood(&self, activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let time_window = get_current_timestamp() - 60.0; // 1 minute

        let pipeline = vec![
            doc! { "$match": {
                "timestamp": { "$gte": time_window },
                "protocol": "DNS"
            }},
            doc! { "$group": {
                "_id": "$source",
                "query_count": { "$sum": 1 }
            }},
            doc! { "$match": {
                "query_count": { "$gt": DNS_FLOOD_THRESHOLD }
            }}
        ];

        let mut cursor = self.dns_collection.aggregate(pipeline).await?;
        let serialized = serde_json::to_string(&activities).unwrap_or_default();


        while let Some(Ok(doc)) = cursor.next().await {
            let source = doc.get_str("_id")?;
            let count = doc.get_i32("query_count")?;

            activities.push(SuspiciousActivity {
                activity_type: "DNS Flood".into(),
                source: source.into(),
                details: format!("{} DNS queries in 1 minute", count),
                timestamp: get_current_timestamp(),
                llm: llm::run(serialized.clone())
            });
        }
        Ok(())
    }

    async fn detect_rare_ports(&self, activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let common_ports = vec![80, 443, 22, 53, 3389, 3306, 1433];
        let common_ports_bson: Vec<Bson> = common_ports.iter().map(|p| Bson::Int32(*p)).collect();

        let pipeline = vec![
            doc! { "$addFields": {
                "port": { "$toInt": { "$arrayElemAt": [{ "$split": ["$destination", ":"] }, 1] } }
            }},
            doc! { "$match": {
                "port": { "$nin": common_ports_bson },
                "protocol": "TCP"
            }},
            doc! { "$group": {
                "_id": "$port",
                "count": { "$sum": 1 },
                "sources": { "$addToSet": "$source" }
            }},
            doc! { "$match": {
                "count": { "$gt": RARE_PORT_THRESHOLD }
            }}
        ];

        let mut cursor = self.tcp_collection.aggregate(pipeline).await?;
        let serialized: String = serde_json::to_string(&activities).unwrap_or_default();


        while let Some(Ok(doc)) = cursor.next().await {
            let port = doc.get_i32("_id")?;
            let count = doc.get_i32("count")?;
            let sources = doc.get_array("sources")?
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            activities.push(SuspiciousActivity {
                activity_type: "Rare Port Activity".into(),
                source: sources,
                details: format!("{} connections to port {}", count, port),
                timestamp: get_current_timestamp(),
                llm: llm::run(serialized.clone())

            });
        }
        Ok(())
    }

    async fn detect_arp_spoofing(&self, activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let pipeline = vec![
            doc! { "$group": {
                "_id": { "$arrayElemAt": [{ "$split": ["$source", " "] }, 2] }, // Extract IP from format "MAC (IP)"
                "macs": { "$addToSet": { "$arrayElemAt": [{ "$split": ["$source", " "] }, 0] } }
            }},
            doc! { "$match": {
                "$expr": { "$gt": [{ "$size": "$macs" }, ARP_SPOOF_THRESHOLD] }
            }}
        ];

        let mut cursor = self.arp_collection.aggregate(pipeline).await?;
        let serialized = serde_json::to_string(&activities).unwrap_or_default();
        while let Some(Ok(doc)) = cursor.next().await {
            let ip = doc.get_str("_id")?;
            let macs = doc.get_array("macs")?
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            activities.push(SuspiciousActivity {
                activity_type: "ARP Spoofing".into(),
                source: ip.into(),
                details: format!("Multiple MACs ({}) claiming same IP", macs),
                timestamp: get_current_timestamp(),
                llm: llm::run(serialized.clone())

            });
        }
        Ok(())
    }

    async fn detect_udp_floods(&self, activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let time_window = get_current_timestamp() - 60.0;

        let pipeline = vec![
            doc! { "$match": {
                "timestamp": { "$gte": time_window },
                "protocol": "UDP"
            }},
            doc! { "$group": {
                "_id": "$source",
                "packet_count": { "$sum": 1 }
            }},
            doc! { "$match": {
                "packet_count": { "$gt": 1000 }
            }}
        ];

        let mut cursor = self.udp_collection.aggregate(pipeline).await?;

        while let Some(Ok(doc)) = cursor.next().await {
            let source = doc.get_str("_id")?;
            let count = doc.get_i32("packet_count")?;

            let serialized = serde_json::to_string(&activities).unwrap_or_default();
            activities.push(SuspiciousActivity {
                activity_type: "UDP Flood".into(),
                source: source.into(),
                details: format!("{} UDP packets in 1 minute", count),
                timestamp: get_current_timestamp(),
                llm: llm::run(serialized.clone())
            });
        }
        Ok(())
    }

    pub async fn store_suspicious_event(&self, activity: SuspiciousActivity) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.suspicious_collection.insert_one(activity).await?;
        Ok(())
    }
}

fn get_current_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64()
}