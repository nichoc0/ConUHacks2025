use mongodb::{
    Client, Collection,
    options::ClientOptions,
    bson::doc,
    IndexModel,
};
use crate::sniff::NetworkEvent;
use futures::StreamExt;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct NetworkDB {
    events_collection: Collection<NetworkEvent>,
}

impl NetworkDB {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
        let client = Client::with_options(client_options)?;
        let db = client.database("network_monitor");
        let events_collection = db.collection("network_events");

        // Create indexes for better query performance
        let index_model = IndexModel::builder()
            .keys(doc! {
                "timestamp": 1,
                "protocol": 1,
                "source": 1,
                "destination": 1
            })
            .build();
        
        events_collection.create_index(index_model).await?;

        Ok(Self {
            events_collection,
        })
    }

    pub async fn store_event(&self, event: NetworkEvent) -> Result<(), Box<dyn Error>> {
        self.events_collection.insert_one(event).await?;
        Ok(())
    }

    pub async fn detect_suspicious_traffic(&self) -> Result<Vec<SuspiciousActivity>, Box<dyn Error>> {
        let mut suspicious_activities = Vec::new();

        // 1. Check for port scanning
        let mut port_scanners = self.events_collection
            .aggregate([
                doc! {
                    "$group": {
                        "_id": {
                            "source": "$source",
                            "window": {
                                "$subtract": [
                                    "$timestamp",
                                    { "$mod": ["$timestamp", 60.0] }
                                ]
                            }
                        },
                        "distinct_destinations": { "$addToSet": "$destination" },
                        "count": { "$sum": 1 }
                    }
                },
                doc! {
                    "$match": {
                        "count": { "$gt": 50 },
                        "distinct_destinations.0": { "$exists": true }
                    }
                }
            ]).await?;

        while let Some(result) = port_scanners.next().await {
            if let Ok(doc) = result {
                suspicious_activities.push(SuspiciousActivity {
                    activity_type: "Port Scanning".to_string(),
                    source: doc.get_document("_id")?.get_str("source")?.to_string(),
                    details: format!("Multiple connections to different destinations"),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)?
                        .as_secs_f64(),
                });
            }
        }

        // 2. Check for unusual data transfer volumes
        let mut large_transfers = self.events_collection
            .aggregate([
                doc! {
                    "$group": {
                        "_id": {
                            "source": "$source",
                            "destination": "$destination"
                        },
                        "total_bytes": { "$sum": "$payload_size" },
                        "count": { "$sum": 1 }
                    }
                },
                doc! {
                    "$match": {
                        "total_bytes": { "$gt": 1000000 }
                    }
                }
            ]).await?;

        while let Some(result) = large_transfers.next().await {
            if let Ok(doc) = result {
                suspicious_activities.push(SuspiciousActivity {
                    activity_type: "Large Data Transfer".to_string(),
                    source: doc.get_document("_id")?.get_str("source")?.to_string(),
                    details: format!("Large data transfer: {} bytes", 
                                   doc.get_i64("total_bytes")?),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)?
                        .as_secs_f64(),
                });
            }
        }

        // 3. Check for suspicious DNS activity
        let mut dns_queries = self.events_collection
            .aggregate([
                doc! {
                    "$match": {
                        "protocol": "DNS",
                    }
                },
                doc! {
                    "$group": {
                        "_id": "$source",
                        "query_count": { "$sum": 1 }
                    }
                },
                doc! {
                    "$match": {
                        "query_count": { "$gt": 100 }
                    }
                }
            ]).await?;

        while let Some(result) = dns_queries.next().await {
            if let Ok(doc) = result {
                suspicious_activities.push(SuspiciousActivity {
                    activity_type: "Suspicious DNS Activity".to_string(),
                    source: doc.get_str("_id")?.to_string(),
                    details: format!("High DNS query rate: {} queries", 
                                   doc.get_i32("query_count")?),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)?
                        .as_secs_f64(),
                });
            }
        }

        Ok(suspicious_activities)
    }
}

#[derive(Debug)]
pub struct SuspiciousActivity {
    pub activity_type: String,
    pub source: String,
    pub details: String,
    pub timestamp: f64,
}