use mongodb::{
    Client, Collection, Database,
    options::ClientOptions,
    bson::doc,
};
use crate::sniff::NetworkEvent;
use futures::StreamExt;
use futures::TryStreamExt; // added for try_next
use std::error::Error;

#[derive(Clone)]
pub struct NetworkDB {
    database: Database,
    tcp_collection: Collection<NetworkEvent>,
    udp_collection: Collection<NetworkEvent>,
    arp_collection: Collection<NetworkEvent>,
    dns_collection: Collection<NetworkEvent>,
    sus_collection: Collection<NetworkEvent>,
}

impl NetworkDB {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
        let client = Client::with_options(client_options)?;
        let db = client.database("network_monitor");

        Ok(Self {
            database: db.clone(),
            tcp_collection: db.collection("tcp_events"),
            udp_collection: db.collection("udp_events"),
            arp_collection: db.collection("arp_events"),
            dns_collection: db.collection("dns_events"),
            sus_collection: db.collection("sus_events"),
        })
    }

    pub fn get_database_instance(&self) -> Database {
        self.database.clone()
    }

    pub async fn store_event(&self, event: NetworkEvent) -> Result<(), Box<dyn Error>> {
        let collection = match event.protocol.as_str() {
            "TCP" => &self.tcp_collection,
            "UDP" => &self.udp_collection,
            "ARP" => &self.arp_collection,
            "DNS" => &self.dns_collection,
            _ => return Ok(()),
        };
        collection.insert_one(event).await?;
        Ok(())
    }

    pub async fn store_suspicious_event(&self, event: NetworkEvent) -> Result<(), Box<dyn Error>> {
        self.sus_collection.insert_one(event).await?;
        Ok(())
    }

    pub async fn refresh_logs(&self) -> Result<(), Box<dyn Error>> {
        let collections = [&self.tcp_collection, &self.udp_collection, &self.arp_collection, &self.dns_collection, &self.sus_collection];
        for collection in collections {
            collection.delete_many(doc! {}, None).await?;
        }
        println!("MongoDB logs cleared.");
        Ok(())
    }

    // New: Get normal network events from TCP, UDP, ARP, and DNS collections
    pub async fn get_normal_events(&self) -> Result<Vec<NetworkEvent>, Box<dyn Error>> {
        let mut events = Vec::new();
        let collections = [&self.tcp_collection, &self.udp_collection, &self.arp_collection, &self.dns_collection];
        for coll in collections {
            let mut cursor = coll.find(doc! {}, None).await?;
            while let Some(event) = cursor.try_next().await? {
                events.push(event);
            }
        }
        Ok(events)
    }

    // New: Get suspicious network events from the sus_events collection
    pub async fn get_suspicious_events(&self) -> Result<Vec<NetworkEvent>, Box<dyn Error>> {
        let mut events = Vec::new();
        let mut cursor = self.sus_collection.find(doc! {}, None).await?;
        while let Some(event) = cursor.try_next().await? {
            events.push(event);
        }
        Ok(events)
    }
}
