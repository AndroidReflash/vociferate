use mdns_sd::{ServiceDaemon, ServiceInfo, DaemonEvent, ServiceEvent};
use crate::discovery::error::DiscoveryError;
use crate::network::PeerInfo;
use std::net::SocketAddr;
use log::{debug, info};

pub struct MdnsService {
    daemon: Option<ServiceDaemon>,
}

impl MdnsService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let daemon = ServiceDaemon::new()?;
        Ok(Self { daemon: Some(daemon) })
    }

    pub async fn browse(&mut self, timeout: std::time::Duration, local_name: Option<&str>) -> Result<Vec<PeerInfo>, Box<dyn std::error::Error>> {
        let daemon = self.daemon.as_mut().unwrap();
        let receiver = daemon.browse(&*crate::discovery::SERVICE_TYPE)?;

        let mut peers = Vec::new();
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if let Ok(event) = receiver.recv_async().await {
                info!("Browse event: {:?}", event);
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let port = info.get_port();
                        let addrs = info.get_addresses();
                        let addr = addrs
                            .iter()
                            .next()
                            .map(|ip| SocketAddr::new(ip.clone(), port))
                            .unwrap_or_else(|| SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), 0));
                        
                        let full_name = info.get_fullname().to_string();
                        let display_name = full_name
                            .strip_suffix(&format!(".{}", crate::discovery::SERVICE_TYPE))
                            .unwrap_or(&full_name)
                            .to_string();

                        // Get hostname from mDNS (e.g., "device-name.local.")
                        let hostname = info.get_hostname().to_string();

                        // Skip self
                        if let Some(local) = local_name {
                            if display_name == local {
                                info!("Skipping self: {}", display_name);
                                continue;
                            }
                        }

                         let device_id = info.get_properties()
                              .get("device_id")
                              .map(|p| p.val_str().to_string())
                              .unwrap_or_else(|| "айди нет".to_string());

                        let caps = crate::protocol::Capabilities::default();
                        let peer = PeerInfo {
                            id: device_id, // выглядит как сраная поебень
                            name: display_name,
                            addr,
                            capabilities: caps,
                            session_id: rand::random(),
                            hostname,
                        };
                         info!("Discovered peer: {:?}", peer);
                        peers.push(peer);
                    }
                    ServiceEvent::ServiceFound(name, _) => {
                        info!("ServiceFound: {}", name);
                    }
                    ServiceEvent::ServiceRemoved(name, _) => {
                        info!("ServiceRemoved: {}", name);
                    }
                    _ => {
                        info!("Other service event: {:?}", event);
                    }
                }
            }
        }

        Ok(peers)
    }

    pub fn register(
        &mut self,
        name: String,
        addr: std::net::SocketAddr,
    ) -> Result<(), DiscoveryError> {
        let mut properties = std::collections::HashMap::new();
        properties.insert("device_name".to_string(), name.clone());

        let ip_str = addr.ip().to_string();
        // mDNS requires hostname to end with .local
        // Generate a valid hostname from the device name

        let sanitized: String = name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c.to_ascii_lowercase() } else { '-' })
            .collect();

        let hostname = format!("{}.local.", sanitized.trim_matches('-'));
        info!("hostname {}", hostname);
        let info = ServiceInfo::new(
            crate::discovery::SERVICE_TYPE,
            &name,           // instance name (display name)
            &hostname,       // hostname - MUST end with .local
            &[ip_str.as_str()][..],  // IP addresses
            addr.port(),
            properties,
        ).map_err(|e| DiscoveryError::RegisterFailed(e.to_string()))?;

        self.daemon
            .as_mut()
            .unwrap()
            .register(info)
            .map_err(|e| DiscoveryError::RegisterFailed(e.to_string()))?;
        Ok(())
    }

    pub fn unregister(&mut self, info: &ServiceInfo) -> Result<(), DiscoveryError> {
        self.daemon
            .as_mut()
            .unwrap()
            .unregister(info.get_fullname())
            .map_err(|e| DiscoveryError::UnregisterFailed(e.to_string()))?;
        Ok(())
    }
}