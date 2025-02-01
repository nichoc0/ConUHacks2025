// src/sniff.rs
use pcap::{Capture, Device, Active};
use crossbeam_channel::Sender;
use etherparse::{ArpHeader, Ipv4Addr, MacAddr, UdpHeaderSlice, InternetSlice};
use std::time::Duration;

#[derive(Debug, serde::Serialize)]
pub struct NetworkEvent {
    pub event_type: String,
    pub source: String,
    pub target: String,
    pub payload: Vec<u8>,
    pub anomaly_score: u8,
    pub explanation: String,
}

const ARP_OPERATION_REPLY: u16 = 2;
const DNS_PORT: u16 = 53;

pub fn start_sniffing(
    interface: Option<&str>,
    sender: Sender<NetworkEvent>,
) -> Result<(), pcap::Error> {
    let mut cap = create_capture(interface)?;
    configure_capture(&mut cap)?;
    
    process_packets(&mut cap, sender)
}

fn create_capture(interface: Option<&str>) -> Result<Capture<Active>, pcap::Error> {
    let device = match interface {
        Some(name) => Device::from_name(name)?,
        None => Device::lookup()?.ok_or(pcap::Error::NoSuchDevice)?,
    };

    Capture::from_device(device)?
        .promisc(false)  // Reduce permissions needed
        .snaplen(2048)   // Optimized for ARP/DNS
        .timeout(500)    // Faster packet processing
        .immediate_mode(true)  // Better real-time behavior
        .open()
}

fn configure_capture(cap: &mut Capture<Active>) -> Result<(), pcap::Error> {
    cap.filter("arp or port 53", true)?;  // More efficient BPF
    cap.setnonblock()?;  // Better for cross-platform async
    Ok(())
}

fn process_packets(cap: &mut Capture<Active>, sender: Sender<NetworkEvent>) -> Result<(), pcap::Error> {
    while let Ok(packet) = cap.next_packet() {
        if let Some(event) = analyze_packet(packet) {
            sender.send(event).unwrap_or_else(|e| 
                eprintln!("Channel error: {}", e)
            );
        }
    }
    Ok(())
}

fn analyze_packet(packet: pcap::Packet) -> Option<NetworkEvent> {
    let mut event = NetworkEvent {
        event_type: String::new(),
        source: String::new(),
        target: String::new(),
        payload: packet.data.to_vec(),
        anomaly_score: 0,
        explanation: String::new(),
    };

    // Layer 2 parsing
    let eth = etherparse::Ethernet2HeaderSlice::from_slice(packet.data).ok()?;
    
    match eth.ether_type() {
        // ARP Processing
        [0x08, 0x06] => handle_arp(eth.payload(), &mut event),
        // IPv4/IPv6 Processing
        [0x08, 0x00] | [0x86, 0xDD] => handle_ip(eth.payload(), &mut event),
        _ => return None,
    }

    (event.anomaly_score > 0).then_some(event)
}

fn handle_arp(payload: &[u8], event: &mut NetworkEvent) {
    let arp = ArpHeader::from_slice(payload).unwrap_or_else(|_| {
        event.anomaly_score += 10;
        event.explanation = "Malformed ARP packet".into();
        return;
    });

    event.event_type = "ARP".into();
    event.source = format!(
        "{}/{}", 
        MacAddr::from_bytes(&arp.sender_hw_addr),
        Ipv4Addr::from_bytes(&arp.sender_proto_addr)
    );
    event.target = format!(
        "{}/{}",
        MacAddr::from_bytes(&arp.target_hw_addr),
        Ipv4Addr::from_bytes(&arp.target_proto_addr)
    );

    // ARP Poisoning Detection
    if arp.operation() == ARP_OPERATION_REPLY {
        event.anomaly_score += 30;
        event.explanation = "Unexpected ARP reply detected".into();
        
        if arp.sender_proto_addr == arp.target_proto_addr {
            event.anomaly_score += 50;
            event.explanation = "Possible ARP spoofing (same IP in sender/target)".into();
        }
    }
}

fn handle_ip(payload: &[u8], event: &mut NetworkEvent) {
    let ip = InternetSlice::from_ip_slice(payload).unwrap_or_else(|_| {
        event.anomaly_score += 10;
        event.explanation = "Malformed IP packet".into();
        return;
    });

    let (src_port, dst_port) = match ip {
        InternetSlice::Ipv4(ipv4, _) => handle_transport(ipv4.payload()),
        InternetSlice::Ipv6(ipv6, _) => handle_transport(ipv6.payload()),
    };

    // DNS Processing
    if src_port == DNS_PORT || dst_port == DNS_PORT {
        event.event_type = "DNS".into();
        event.source = format!("{}:{}", ip.source_addr(), src_port);
        event.target = format!("{}:{}", ip.destination_addr(), dst_port);
        
        // Basic DNS Poisoning Detection
        if payload.len() > 512 {
            event.anomaly_score += 30;
            event.explanation = "Oversized DNS payload".into();
        }
    }
}

fn handle_transport(payload: &[u8]) -> (u16, u16) {
    UdpHeaderSlice::from_slice(payload)
        .map(|udp| (udp.source_port(), udp.destination_port()))
        .unwrap_or((0, 0))
}