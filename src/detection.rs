use mongodb::{bson::doc, Collection};
use futures::StreamExt;
use std::error::Error;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::sniff::NetworkEvent;


#[derive(Debug)]
#[derive(Serialize)]
pub struct SuspiciousActivity {
    pub activity_type: String,
    pub source: String,
    pub details: String,
    pub timestamp: f64,
<<<<<<< HEAD
=======
    llm: ()
>>>>>>> ea95057e5c0585f893ff307c2072aa5253f4e7f1
}

#[derive(Clone)]
pub struct TrafficAnalyzer {
    tcp_collection: Collection<NetworkEvent>,
    udp_collection: Collection<NetworkEvent>,
    dns_collection: Collection<NetworkEvent>,
    suspicious_collection: Collection<SuspiciousActivity>,
}

impl TrafficAnalyzer {
    pub async fn new(db: mongodb::Database) -> Self {
        Self {
            tcp_collection: db.collection("tcp_events"),
            udp_collection: db.collection("udp_events"),
            dns_collection: db.collection("dns_events"),
            suspicious_collection: db.collection("sus_events"),
        }
    }

    pub async fn store_suspicious_event(&self, activity: SuspiciousActivity) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.suspicious_collection.insert_one(activity).await?;
        Ok(())
    }

    pub async fn detect_suspicious_traffic(&self) -> Result<Vec<SuspiciousActivity>, Box<dyn Error + Send + Sync>> {

        let mut suspicious_activities = Vec::new();

        // Detect port scanning (many destinations from one source)
        self.detect_port_scanning(&mut suspicious_activities).await?;

        // Detect high data transfers
        self.detect_large_transfers(&mut suspicious_activities).await?;

        // Detect DNS flood (high query rate from a single source)
        self.detect_dns_flood(&mut suspicious_activities).await?;

        // Detect rare port connections
        self.detect_rare_ports(&mut suspicious_activities).await?;

        Ok(suspicious_activities)
    }

    async fn detect_port_scanning(&self, suspicious_activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut cursor = self.tcp_collection.aggregate([
            doc! { "$group": {
                "_id": "$source",
                "distinct_destinations": { "$addToSet": "$destination" },
                "count": { "$sum": 1 }
            }},
            doc! { "$match": { "count": { "$gt": 50 } } }
        ]).await?;

        while let Some(Ok(doc)) = cursor.next().await {
            suspicious_activities.push(SuspiciousActivity {
                activity_type: "Port Scanning".to_string(),
                source: doc.get_str("_id")?.to_string(),
                details: "High number of distinct destination connections".to_string(),
                timestamp: get_current_timestamp(),
<<<<<<< HEAD
=======
                llm: llm::run(serialized.clone())  // Convert llm result to String
>>>>>>> ea95057e5c0585f893ff307c2072aa5253f4e7f1
            });
        }
        Ok(())
    }


    async fn detect_large_transfers(&self, suspicious_activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut cursor = self.tcp_collection.aggregate([
            doc! { "$group": {
                "_id": "$source",
                "total_bytes": { "$sum": "$payload_size" }
            }},
            doc! { "$match": { "total_bytes": { "$gt": 1000000 } } }
        ]).await?;

        while let Some(Ok(doc)) = cursor.next().await {
            suspicious_activities.push(SuspiciousActivity {
                activity_type: "Large Data Transfer".to_string(),
                source: doc.get_str("_id")?.to_string(),
                details: format!("Transferred {} bytes", doc.get_i64("total_bytes")?),
                timestamp: get_current_timestamp(),
<<<<<<< HEAD
=======
                llm: llm::run(serialized.clone())

>>>>>>> ea95057e5c0585f893ff307c2072aa5253f4e7f1
            });
        }
        Ok(())
    }

    async fn detect_dns_flood(&self, suspicious_activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut cursor = self.dns_collection.aggregate([
            doc! { "$group": {
                "_id": "$source",
                "query_count": { "$sum": 1 }
            }},
            doc! { "$match": { "query_count": { "$gt": 100 } } }
        ]).await?;

        while let Some(Ok(doc)) = cursor.next().await {
            suspicious_activities.push(SuspiciousActivity {
                activity_type: "Suspicious DNS Activity".to_string(),
                source: doc.get_str("_id")?.to_string(),
                details: format!("High query rate: {} queries", doc.get_i32("query_count")?),
                timestamp: get_current_timestamp(),
<<<<<<< HEAD
=======
                llm: llm::run(serialized.clone())
>>>>>>> ea95057e5c0585f893ff307c2072aa5253f4e7f1
            });
        }
        Ok(())
    }

    async fn detect_rare_ports(&self, suspicious_activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let common_ports = vec![22, 53, 80, 443, 3306];
        let mut cursor = self.tcp_collection.aggregate([
            doc! { "$match": {
                "destination": { "$not": { "$in": common_ports } }
            }},
            doc! { "$group": {
                "_id": "$destination",
                "count": { "$sum": 1 }
            }},
            doc! { "$match": { "count": { "$gt": 5 } } }
        ]).await?;

        while let Some(Ok(doc)) = cursor.next().await {
            suspicious_activities.push(SuspiciousActivity {
                activity_type: "Rare Port Activity".to_string(),
                source: "Unknown".to_string(),
                details: format!("Connections to rare port {}", doc.get_i32("_id")?),
                timestamp: get_current_timestamp(),
<<<<<<< HEAD
=======
                llm: llm::run(serialized.clone())

>>>>>>> ea95057e5c0585f893ff307c2072aa5253f4e7f1
            });
        }
        Ok(())
    }
<<<<<<< HEAD
=======

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
>>>>>>> ea95057e5c0585f893ff307c2072aa5253f4e7f1
}

fn get_current_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64()
}
