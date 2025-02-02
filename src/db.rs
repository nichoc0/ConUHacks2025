use mongodb::{
    Client, Collection,
    options::ClientOptions,
    bson::doc,
};
use crate::sniff::NetworkEvent;
use futures::StreamExt;
use std::error::Error;

#[derive(Clone)]
pub struct NetworkDB {
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
            tcp_collection: db.collection("tcp_events"),
            udp_collection: db.collection("udp_events"),
            arp_collection: db.collection("arp_events"),
            dns_collection: db.collection("dns_events"),
            sus_collection: db.collection("sus_events"),
        })
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
            collection.delete_many(doc! {}).await?;
        }
        println!("MongoDB logs cleared.");
        Ok(())
    }
}
