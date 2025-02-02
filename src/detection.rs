use mongodb::{bson::{doc, Bson}, Collection};
use futures::StreamExt;
use std::error::Error;
use serde::Serialize;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::sniff::NetworkEvent;

const PORT_SCAN_THRESHOLD: i32 = 12; 
const DNS_FLOOD_THRESHOLD: i32 = 250; 
const LARGE_TRANSFER_THRESHOLD: i64 = 8_000_000; 
const RARE_PORT_THRESHOLD: i32 = 10;
const ARP_SPOOF_THRESHOLD: i32 = 2; 
const UDP_FLOOD_THRESHOLD: i32 = 800; 
const PORT_SCAN_WINDOW: f64 = 180.0; 
const LARGE_TRANSFER_WINDOW: f64 = 300.0; 
const DNS_FLOOD_WINDOW: f64 = 60.0;
const UDP_FLOOD_WINDOW: f64 = 60.0; 
#[derive(Debug, Serialize)]
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
    arp_collection: Collection<NetworkEvent>,
    suspicious_collection: Collection<SuspiciousActivity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsMapping {
    query: String,
    resolved_ip: String,
    timestamp: f64,
    is_http: bool,
    source: String,
}

#[derive(Debug, Serialize)]
pub struct SuspiciousDns {
    pub domain: String,
    pub resolved_ip: String,
    pub source: String,
    pub protocol: String,
    pub timestamp: f64,
    pub risk_level: String,
}
impl TrafficAnalyzer {
    pub async fn new(db: mongodb::Database) -> Self {
        Self {
            tcp_collection: db.collection("tcp_events"),
            udp_collection: db.collection("udp_events"),
            dns_collection: db.collection("dns_events"),
            arp_collection: db.collection("arp_events"),
            suspicious_collection: db.collection("sus_events"),
            dns_mapping: db.collection("dns_mappings"),
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
        self.detect_suspicious_dns(&mut suspicious_activities).await?;

        Ok(suspicious_activities)
    }
    async fn detect_suspicious_dns(&self, activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let time_window = get_current_timestamp() - 3600.0; // Look back 1 hour
        
        let pipeline = vec![
            doc! {
                "$match": {
                    "timestamp": { "$gte": time_window },
                    "protocol": "DNS",
                    "query_type": "A"  // DNS A records (domain to IP mapping)
                }
            },
            doc! {
                "$lookup": {
                    "from": "tcp_events",
                    "let": { "resolved_ip": "$resolved_ip" },
                    "pipeline": [
                        {
                            "$match": {
                                "$expr": {
                                    "$and": [
                                        { "$eq": ["$protocol", "TCP"] },
                                        { "$eq": [{ "$arrayElemAt": [{ "$split": ["$destination", ":"] }, 0] }, "$$resolved_ip"] },
                                        { "$in": [{ "$arrayElemAt": [{ "$split": ["$destination", ":"] }, 1] }, ["80"]] }  // HTTP port
                                    ]
                                }
                            }
                        }
                    ],
                    "as": "http_connections"
                }
            },
            doc! {
                "$addFields": {
                    "is_http": { "$gt": [{ "$size": "$http_connections" }, 0] }
                }
            }
        ];

        let mut cursor = self.dns_collection.aggregate(pipeline).await?;

        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    if let (Ok(query), Ok(resolved_ip), Ok(is_http), Ok(source)) = (
                        doc.get_str("query"),
                        doc.get_str("resolved_ip"),
                        doc.get_bool("is_http"),
                        doc.get_str("source")
                    ) {
                        if is_http {
                            // Check for suspicious patterns in DNS queries
                            let risk_level = assess_dns_risk(query);
                            
                            if risk_level != "Low" {
                                activities.push(SuspiciousActivity {
                                    activity_type: "Suspicious DNS".into(),
                                    source: source.into(),
                                    details: format!(
                                        "Suspicious {} domain: {} resolved to {} (HTTP traffic detected)",
                                        risk_level, query, resolved_ip
                                    ),
                                    timestamp: get_current_timestamp(),
                                });

                                // Store the mapping for further analysis
                                let mapping = DnsMapping {
                                    query: query.into(),
                                    resolved_ip: resolved_ip.into(),
                                    timestamp: get_current_timestamp(),
                                    is_http: true,
                                    source: source.into(),
                                };

                                self.store_dns_mapping(mapping).await?;
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error processing DNS document: {}", e),
            }
        }
        Ok(())
    }
    async fn detect_port_scanning(&self, activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let time_window = get_current_timestamp() - PORT_SCAN_WINDOW;
        
        let pipeline = vec![
            doc! { "$match": { 
                "timestamp": { "$gte": time_window },
                "protocol": "TCP" 
            }},
            doc! { "$addFields": { 
                "dest_parts": { "$split": ["$destination", ":"] }
            }},
            doc! { "$match": {
                "dest_parts": { "$size": 2 }  
            }},
            doc! { "$addFields": { 
                "dest_ip": { "$arrayElemAt": ["$dest_parts", 0] },
                "dest_port": { 
                    "$toInt": { 
                        "$arrayElemAt": ["$dest_parts", 1] 
                    }
                }
            }},
            doc! { "$group": { 
                "_id": { 
                    "source": "$source", 
                    "dest_ip": "$dest_ip"
                },
                "unique_ports": { "$addToSet": "$dest_port" },
                "total_attempts": { "$sum": 1 }
            }},
            doc! { "$match": { 
                "$expr": { "$gt": [{ "$size": "$unique_ports" }, PORT_SCAN_THRESHOLD] }
            }}
        ];

        let mut cursor = self.tcp_collection.aggregate(pipeline).await?;

        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    if let Ok(id) = doc.get_document("_id") {
                        if let (Ok(source), Ok(dest_ip)) = (id.get_str("source"), id.get_str("dest_ip")) {
                            if let Ok(ports) = doc.get_array("unique_ports") {
                                activities.push(SuspiciousActivity {
                                    activity_type: "Port Scanning".into(),
                                    source: source.into(),
                                    details: format!("{} unique ports scanned on {}", ports.len(), dest_ip),
                                    timestamp: get_current_timestamp(),
                                });
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error processing port scanning document: {}", e),
            }
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

        while let Some(Ok(doc)) = cursor.next().await {
            let source = doc.get_str("_id")?;
            let bytes = doc.get_i64("total_bytes")?;

            activities.push(SuspiciousActivity {
                activity_type: "Large Data Transfer".into(),
                source: source.into(),
                details: format!("{} bytes transferred in 5 minutes", bytes),
                timestamp: get_current_timestamp(),
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

        while let Some(Ok(doc)) = cursor.next().await {
            let source = doc.get_str("_id")?;
            let count = doc.get_i32("query_count")?;

            activities.push(SuspiciousActivity {
                activity_type: "DNS Flood".into(),
                source: source.into(),
                details: format!("{} DNS queries in 1 minute", count),
                timestamp: get_current_timestamp(),
            });
        }
        Ok(())
    }

    async fn detect_rare_ports(&self, activities: &mut Vec<SuspiciousActivity>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let common_ports = vec![
            20, 21, 22, 23, 25, 53, 80, 110, 143, 443, 465, 587, 993, 995, 
            1433, 1521, 3306, 3389, 5432, 5900, 5901, 6379, 8080, 8443, 
            27017, 27018, 27019,
            2375, 2376,
            6660, 6661, 6662, 6663, 6664, 6665, 6666, 6667, 6668, 6669,
            5060, 5061
        ];
        let common_ports_bson: Vec<Bson> = common_ports.iter().map(|p| Bson::Int32(*p)).collect();
        
        let pipeline = vec![
            doc! { "$match": { "timestamp": { "$gte": get_current_timestamp() - 300.0 } } },
            doc! { "$addFields": { 
                "dest_parts": { "$split": ["$destination", ":"] }
            }},
            doc! { "$match": {
                "dest_parts": { "$size": 2 }
            }},
            doc! { "$addFields": { 
                "port": { 
                    "$toInt": { 
                        "$arrayElemAt": ["$dest_parts", 1] 
                    }
                },
                "dest_ip": { "$arrayElemAt": ["$dest_parts", 0] }
            }},
            doc! { "$match": { 
                "port": { "$nin": common_ports_bson },
                "protocol": "TCP"
            }},
            doc! { "$group": { 
                "_id": {
                    "port": "$port",
                    "dest_ip": "$dest_ip"
                },
                "count": { "$sum": 1 },
                "sources": { "$addToSet": "$source" }
            }},
            doc! { "$match": { 
                "count": { "$gt": RARE_PORT_THRESHOLD }
            }}
        ];

        let mut cursor = self.tcp_collection.aggregate(pipeline).await?;

        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    if let Ok(id) = doc.get_document("_id") {
                        if let (Ok(port), Ok(dest_ip)) = (id.get_i32("port"), id.get_str("dest_ip")) {
                            if let (Ok(count), Ok(sources)) = (doc.get_i32("count"), doc.get_array("sources")) {
                                let sources_str = sources
                                    .iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ");

                                if !is_common_port_range(port) {
                                    activities.push(SuspiciousActivity {
                                        activity_type: "Rare Port Activity".into(),
                                        source: sources_str,
                                        details: format!("{} connections to port {} on {}", count, port, dest_ip),
                                        timestamp: get_current_timestamp(),
                                    });
                                }
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error processing rare ports document: {}", e),
            }
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
            });
        }
        Ok(())
    }
    async fn store_dns_mapping(&self, mapping: DnsMapping) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Assuming we've added a new collection for DNS mappings
        self.db.collection("dns_mappings").insert_one(mapping).await?;
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

            activities.push(SuspiciousActivity {
                activity_type: "UDP Flood".into(),
                source: source.into(),
                details: format!("{} UDP packets in 1 minute", count),
                timestamp: get_current_timestamp(),
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
fn is_common_port_range(port: i32) -> bool {
    matches!(port,
        1..=1023 | // Well-known ports
        1024..=49151 | // Registered ports
        50000..=51000 | // Common application ports
        60000..=61000 // Common application ports
    )
}

fn determine_port_severity(port: i32, connection_count: i32) -> String {
    match (port, connection_count) {
        (p, c) if p < 1024 && c > 20 => "High".into(),
        (p, c) if p < 1024 => "Medium".into(),
        (_, c) if c > 50 => "High".into(),
        (_, c) if c > 20 => "Medium".into(),
        _ => "Low".into()
    }
}
fn assess_dns_risk(domain: &str) -> String {
    // Suspicious patterns to check for
    let suspicious_patterns = vec![
        // Length-based patterns
        (domain.len() > 50, "High"),                    // Unusually long domain
        (domain.chars().filter(|c| c.is_digit(10)).count() > 10, "High"),  // Many numbers
        
        // Character pattern checks
        (domain.contains("--"), "Medium"),              // Double hyphens
        (domain.matches('.').count() > 4, "Medium"),    // Many subdomains
        
        // Common legitimate TLDs (inverse check)
        (!domain.ends_with(".com") && 
         !domain.ends_with(".org") && 
         !domain.ends_with(".net") && 
         !domain.ends_with(".edu") && 
         !domain.ends_with(".gov"), "Medium"),
        
        // Entropy checks
        (calculate_entropy(domain) > 4.5, "High"),      // High randomness
        
        // Sequence checks
        (has_repeating_patterns(domain), "Medium"),
        (has_keyboard_patterns(domain), "Medium"),
    ];

    // Return highest risk level found
    for (condition, risk) in suspicious_patterns {
        if condition {
            return risk.to_string();
        }
    }
    
    "Low".to_string()
}

fn calculate_entropy(s: &str) -> f64 {
    let len = s.len() as f64;
    let mut char_counts = std::collections::HashMap::new();
    
    // Count character frequencies
    for c in s.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }
    
    // Calculate entropy
    char_counts.values()
        .map(|&count| {
            let p = count as f64 / len;
            -p * p.log2()
        })
        .sum()
}

fn has_repeating_patterns(s: &str) -> bool {
    // Check for repeating sequences of 3 or more characters
    let chars: Vec<char> = s.chars().collect();
    for window_size in 3..=6 {
        for i in 0..chars.len() - window_size * 2 {
            let pattern = &chars[i..i + window_size];
            let next_segment = &chars[i + window_size..i + window_size * 2];
            if pattern == next_segment {
                return true;
            }
        }
    }
    false
}

fn has_keyboard_patterns(s: &str) -> bool {
    // Common keyboard patterns
    let keyboard_patterns = vec![
        "qwer", "asdf", "zxcv", "1234", "4321",
        "qaz", "wsx", "edc", "rfv"
    ];
    
    let lower = s.to_lowercase();
    keyboard_patterns.iter().any(|pattern| lower.contains(pattern))
}