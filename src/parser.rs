// store the entries
// on update/tick flag duplicates
// dumps to json

use multimap::MultiMap;

// let sender_mac = format_mac(&payload[8..14]);
// let sender_ip = Ipv4Addr::new(payload[14], payload[15], payload[16], payload[17])

pub struct Storage {
    entries: MultiMap<String, String>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            entries: MultiMap::new(),
        }
    }
    pub fn add(&mut self, sender_mac: &String, sender_ip: &String) {
        self.entries.insert(sender_mac.clone(), sender_ip.clone());
    }

    pub fn scan(&mut self) {
        // check for arp spoofing or man in the middle
        // send event to llm
    }
    fn jsonify(&mut self) {


    }
}
