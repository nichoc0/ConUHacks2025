// src/sniff.rs
use pcap::{Capture, Device, Active};
use crossbeam_channel::Sender;
use etherparse::{Ethernet2HeaderSlice, InternetSlice, TransportSlice};
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize)]
pub struct NetworkEvent {
    pub protocol: String,
    pub source: String,
    pub destination: String,
    pub payload_size: usize,
}

const DNS_PORT: u16 = 53;

pub fn start_sniffing(interface: Option<&str>, sender: Sender<NetworkEvent>) -> Result<(), pcap::Error> {
    let mut cap = create_capture(interface)?;
    
    while let Ok(packet) = cap.next_packet() {
        if let Some(event) = parse_packet(packet) {
            sender.send(event).unwrap_or_else(|e| eprintln!("Channel error: {}", e));
        }
    }

    Ok(())
}

fn create_capture(interface: Option<&str>) -> Result<Capture<Active>, pcap::Error> {
    let device = match interface {
        Some(name) => Device::from_name(name)?,
        None => Device::lookup()?.ok_or(pcap::Error::NoSuchDevice)?,
    };

    Capture::from_device(device)?
        .promisc(false)  // Reduce permissions needed
        .snaplen(2048)   // Optimized capture size
        .timeout(500)    // Faster packet processing
        .immediate_mode(true)  // Real-time behavior
        .open()
}

fn parse_packet(packet: pcap::Packet) -> Option<NetworkEvent> {
    let eth = Ethernet2HeaderSlice::from_slice(packet.data).ok()?;

    let (protocol, source, destination) = match eth.ether_type() {
        0x0806 => ("ARP".into(), "N/A".into(), "N/A".into()), // ARP does not have IPs
        0x0800 | 0x86DD => parse_ip_packet(eth.payload())?,
        _ => return None, // Ignore unknown protocols
    };

    Some(NetworkEvent {
        protocol,
        source,
        destination,
        payload_size: packet.data.len(),
    })
}

fn parse_ip_packet(payload: &[u8]) -> Option<(String, String, String)> {
    let ip = InternetSlice::from_ip_slice(payload).ok()?;

    let (protocol, src_port, dst_port) = match ip {
        InternetSlice::Ipv4(ipv4, _) => parse_transport_layer(ipv4.payload()),
        InternetSlice::Ipv6(ipv6, _) => parse_transport_layer(ipv6.payload()),
    };

    let source = format!("{}:{}", ip.source_addr(), src_port);
    let destination = format!("{}:{}", ip.destination_addr(), dst_port);

    Some((protocol, source, destination))
}

fn parse_transport_layer(payload: &[u8]) -> (String, u16, u16) {
    match TransportSlice::from_slice(payload) {
        Ok(TransportSlice::Udp(udp)) => {
            let protocol = if udp.source_port() == DNS_PORT || udp.destination_port() == DNS_PORT {
                "DNS"
            } else {
                "UDP"
            };
            (protocol.into(), udp.source_port(), udp.destination_port())
        }
        Ok(TransportSlice::Tcp(tcp)) => ("TCP".into(), tcp.source_port(), tcp.destination_port()),
        Ok(TransportSlice::Icmpv4(_)) | Ok(TransportSlice::Icmpv6(_)) => ("ICMP".into(), 0, 0),
        _ => ("Unknown".into(), 0, 0),
    }
}
