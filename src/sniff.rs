
use pcap::{Capture, Device, Active};
use crossbeam_channel::Sender;
use etherparse::{Ethernet2HeaderSlice, EtherType, IpNumber, Ipv4HeaderSlice, Ipv6HeaderSlice, TcpHeaderSlice, UdpHeaderSlice};
use serde::Serialize;
use std::net::Ipv4Addr;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::parser::Storage;

#[derive(Debug, Serialize, Clone)]
pub struct NetworkEvent {
    pub protocol: String,
    pub source: String,
    pub destination: String,
    pub payload_size: usize,
    pub timestamp: f64,
}

const DNS_PORT: u16 = 53;

pub fn start_sniffing(interface: Option<&str>, sender: Sender<NetworkEvent>) -> Result<(), pcap::Error> {
    let mut cap = create_capture(interface)?;

    while let Ok(packet) = cap.next() {
        if let Some(event) = parse_packet(&packet) {
            sender.send(event).unwrap_or_else(|e| eprintln!("Channel error: {}", e));
        }
    }

    Ok(())
}

fn create_capture(interface: Option<&str>) -> Result<Capture<Active>, pcap::Error> {
    let device = match interface {
        Some(name) => Device::list()?.into_iter().find(|d| d.name == name),
        None => Device::list()?.into_iter().next(),
    }.ok_or_else(|| pcap::Error::InvalidString)?;

    let mut cap = Capture::from_device(device)?
        .promisc(false)  // Non-promiscuous by default for legal compliance
        .snaplen(2048)
        .immediate_mode(true)
        .open()
}

fn parse_packet(packet: &pcap::Packet) -> Option<NetworkEvent> {
    let eth = Ethernet2HeaderSlice::from_slice(packet.data).ok()?;
    let ether_type = eth.ether_type();

    let (protocol, source, destination) = match ether_type {
        EtherType::ARP => parse_arp(&packet.data[14..]),
        EtherType::IPV4 => parse_ipv4(&packet.data[14..])?,
        EtherType::IPV6 => parse_ipv6(&packet.data[14..])?,
        _ => return None,
    };

    Some(NetworkEvent {
        protocol,
        source,
        destination,
        payload_size: packet.data.len(),
        timestamp: get_timestamp(),
    })
}

fn parse_arp(payload: &[u8]) -> (String, String, String) {
    if payload.len() < 28 {
        return ("ARP (Malformed)".to_string(), "N/A".to_string(), "N/A".to_string());
    }
    // If the same sender ip but different mac addres  we need to eventually flag the event.
    let sender_mac = format_mac(&payload[8..14]);
    let sender_ip = Ipv4Addr::new(payload[14], payload[15], payload[16], payload[17]);

    // Store the data to be parsed later for threat analysis.
    let mut storage = Storage::new();
    storage.add(&sender_mac.to_string(), &sender_ip.to_string());
    
    let target_mac = format_mac(&payload[18..24]);
    let target_ip = Ipv4Addr::new(payload[24], payload[25], payload[26], payload[27]);

    (
        "ARP".to_string(),
        format!("{} ({})", sender_mac, sender_ip),
        format!("{} ({})", target_mac, target_ip),
    )
}

fn parse_ipv4(payload: &[u8]) -> Option<(String, String, String)> {
    let ip_header = Ipv4HeaderSlice::from_slice(payload).ok()?;
    let src_ip = ip_header.source_addr();
    let dst_ip = ip_header.destination_addr();

    let payload = &payload[ip_header.slice().len()..];

    match ip_header.protocol() {
        IpNumber::TCP => parse_tcp(payload, src_ip.to_string(), dst_ip.to_string()),
        IpNumber::UDP => parse_udp(payload, src_ip.to_string(), dst_ip.to_string()),
        IpNumber::ICMP => Some(("ICMP".to_string(), src_ip.to_string(), dst_ip.to_string())),
        _ => Some(("Unknown IP".to_string(), src_ip.to_string(), dst_ip.to_string())),
    }
}

fn parse_ipv6(payload: &[u8]) -> Option<(String, String, String)> {
    let ip_header = Ipv6HeaderSlice::from_slice(payload).ok()?;
    let src_ip = ip_header.source_addr();
    let dst_ip = ip_header.destination_addr();

    let payload = &payload[ip_header.slice().len()..];

    match ip_header.next_header() {
        IpNumber::TCP => parse_tcp(payload, src_ip.to_string(), dst_ip.to_string()),
        IpNumber::UDP => parse_udp(payload, src_ip.to_string(), dst_ip.to_string()),
        IpNumber::IPV6_ICMP => Some(("ICMPv6".to_string(), src_ip.to_string(), dst_ip.to_string())),
        _ => Some(("Unknown IPv6".to_string(), src_ip.to_string(), dst_ip.to_string())),
    }
}

fn parse_tcp(payload: &[u8], src_ip: String, dst_ip: String) -> Option<(String, String, String)> {
    let tcp_header = TcpHeaderSlice::from_slice(payload).ok()?;
    let src_port = tcp_header.source_port();
    let dst_port = tcp_header.destination_port();

    let protocol = if is_dns_port(src_port, dst_port) {
        "DNS".to_string()
    } else {
        "TCP".to_string()
    };

    Some((
        protocol,
        format!("{}:{}", src_ip, src_port),
        format!("{}:{}", dst_ip, dst_port),
    ))
}

fn parse_udp(payload: &[u8], src_ip: String, dst_ip: String) -> Option<(String, String, String)> {
    let udp_header = UdpHeaderSlice::from_slice(payload).ok()?;
    let src_port = udp_header.source_port();
    let dst_port = udp_header.destination_port();

    let protocol = if is_dns_port(src_port, dst_port) {
        "DNS".to_string()
    } else {
        "UDP".to_string()
    };

    Some((
        protocol,
        format!("{}:{}", src_ip, src_port),
        format!("{}:{}", dst_ip, dst_port),
    ))
}

fn is_dns_port(src: u16, dst: u16) -> bool {
    src == DNS_PORT || dst == DNS_PORT
}

fn format_mac(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}

fn get_timestamp() -> f64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    now.as_secs() as f64 + (now.subsec_micros() as f64 / 1_000_000.0)
}