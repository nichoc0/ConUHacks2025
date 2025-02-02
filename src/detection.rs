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
            });
        }
        Ok(())
    }
}

fn get_current_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64()
}
