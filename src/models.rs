pub mod domain {
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    pub struct NetworkEvent {
        pub protocol: String,
        pub source: String,
        pub destination: String,
        pub payload_size: usize,
        pub timestamp: f64,
    }

    #[derive(Debug)]
    pub struct NetworkStats {
        pub total_packets: usize,
        pub total_bytes: usize,
        pub protocol_counts: HashMap<String, usize>,
        pub unique_sources: HashMap<String, usize>,
        pub unique_destinations: HashMap<String, usize>,
    }
}

pub mod dto {
    use serde::Serialize;

    #[derive(Debug, Serialize, Clone)]
    pub struct NetworkEventDTO {
        pub protocol: String,
        pub source: String,
        pub destination: String,
        pub payload_size: usize,
        pub timestamp: f64,
    }
}
