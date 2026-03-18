use mdns_sd::{ServiceDaemon, ServiceInfo};
use tracing::{info, error};
use rand::Rng;

const AIRPLAY_SERVICE_TYPE: &str = "_airplay._tcp.local.";
const AIRPLAY_PORT: u16 = 7100;

/// Advertises the PC as an AirPlay receiver on the local network via mDNS.
/// iPhones will see "Phone Mirror" in their Screen Mirroring list.
pub struct AirPlayAdvertiser {
    daemon: Option<ServiceDaemon>,
    fullname: Option<String>,
}

impl AirPlayAdvertiser {
    pub fn new() -> Self {
        Self {
            daemon: None,
            fullname: None,
        }
    }

    /// Start advertising the AirPlay service on the network.
    pub fn start(&mut self) -> anyhow::Result<()> {
        let daemon = ServiceDaemon::new()?;

        // Generate a stable-ish device ID (MAC-like format)
        let device_id = Self::generate_device_id();
        let instance_name = "Phone Mirror";

        // TXT records that make iPhones recognize us as an AirPlay receiver
        let properties = vec![
            ("deviceid".to_string(), device_id.clone()),
            ("features".to_string(), "0x527FFFF7,0x1E".to_string()),
            ("model".to_string(), "AppleTV3,2".to_string()),
            ("srcvers".to_string(), "220.68".to_string()),
            ("flags".to_string(), "0x44".to_string()),
            ("pk".to_string(), hex::encode([0u8; 32])), // Placeholder public key
            ("pi".to_string(), uuid::Uuid::new_v4().to_string()),
            ("vv".to_string(), "2".to_string()),
        ];

        let service = ServiceInfo::new(
            AIRPLAY_SERVICE_TYPE,
            instance_name,
            &format!("{}.", hostname::get()?.to_string_lossy()),
            "",  // Let mdns-sd pick the IP
            AIRPLAY_PORT,
            properties.as_slice(),
        )?;

        let fullname = service.get_fullname().to_string();
        daemon.register(service)?;

        info!(
            "AirPlay advertiser started: {} on port {} (deviceid={})",
            fullname, AIRPLAY_PORT, device_id
        );

        self.daemon = Some(daemon);
        self.fullname = Some(fullname);

        Ok(())
    }

    pub fn stop(&mut self) {
        if let (Some(daemon), Some(fullname)) = (self.daemon.take(), self.fullname.take()) {
            let _ = daemon.unregister(&fullname);
            let _ = daemon.shutdown();
            info!("AirPlay advertiser stopped");
        }
    }

    /// Generate a MAC-like device ID
    fn generate_device_id() -> String {
        let mut rng = rand::rng();
        let bytes: [u8; 6] = rng.random();
        format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
        )
    }

    pub fn port(&self) -> u16 {
        AIRPLAY_PORT
    }
}

impl Drop for AirPlayAdvertiser {
    fn drop(&mut self) {
        self.stop();
    }
}
