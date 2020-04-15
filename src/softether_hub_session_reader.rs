use crate::softether_reader::{SoftEtherReader, SoftEtherError};
use csv;
use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

pub struct SoftEtherHubSessionReader;

impl SoftEtherHubSessionReader {
    pub fn hub_sessions(
        vpncmd: &str,
        server: &str,
        hub: &str,
        password: &str,
    ) -> Result<Vec<HubSession>, Box<dyn Error>> {
        let mut child = Command::new(vpncmd)
            .arg(server)
            .arg("/SERVER")
            .arg(format!("/HUB:{}", hub))
            .arg(format!("/PASSWORD:{}", password))
            .arg("/CSV")
            .arg("/CMD")
            .arg("SessionList")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        {
            let stdin = child.stdin.as_mut().unwrap();
            // Input Ctrl-D to interrupt password prompt
            stdin.write_all(&[4])?;
        }

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let msg = String::from_utf8_lossy(output.stdout.as_slice());
            return Err(Box::new(SoftEtherError {
                msg: String::from(format!("vpncmd failed ( {} )", msg)),
            }));
        }

        let sessionlist = 
            match SoftEtherHubSessionReader::decode_hub_sessions(&output.stdout) {
                Ok(x) => x,
                Err(x) => {
                    return Err(x);
                }
            };
        let mut sessions = Vec::new();
        for session in sessionlist {
            let sessioninfo = 
                match SoftEtherHubSessionReader::hub_session_info(&vpncmd, &server, &hub, &password, &session) {
                    Ok(x) => x,
                    Err(x) => {
                        return Err(x);
                    }
                };
        	sessions.push(sessioninfo);
        }
        
        return Ok(sessions);
        
    }
    
    fn hub_session_info(
        vpncmd: &str,
        server: &str,
        hub: &str,
        password: &str,
        session: &HubSessionListEntry,
    ) -> Result<HubSession, Box<dyn Error>> {
        let mut child = Command::new(vpncmd)
            .arg(server)
            .arg("/SERVER")
            .arg(format!("/HUB:{}", hub))
            .arg(format!("/PASSWORD:{}", password))
            .arg("/CSV")
            .arg("/CMD")
            .arg("SessionGet")
            .arg(session.name.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        {
            let stdin = child.stdin.as_mut().unwrap();
            // Input Ctrl-D to interrupt password prompt
            stdin.write_all(&[4])?;
        }

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let msg = String::from_utf8_lossy(output.stdout.as_slice());
            return Err(Box::new(SoftEtherError {
                msg: String::from(format!("vpncmd failed ( {} )", msg)),
            }));
        }

        SoftEtherHubSessionReader::decode_hub_session_info(&output.stdout,&session)
    }

    fn decode_hub_sessions(
    	src: &[u8],
    ) -> Result<Vec<HubSessionListEntry>, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_reader(src);
        let mut sessions = Vec::new();

        for entry in rdr.records() {
            let entry = entry?;
            let name = entry.get(0).unwrap_or("");
            let vlan_id = entry.get(1).unwrap_or("");
            let location = entry.get(2).unwrap_or("");
            let user = entry.get(3).unwrap_or("");
            let source = entry.get(4).unwrap_or("");
            let connections = entry.get(5).unwrap_or("");
            let transfer_bytes = entry.get(6).unwrap_or("");
            let transfer_packets = entry.get(7).unwrap_or("");

            let connections = SoftEtherReader::decode_connections(connections)?;
            let transfer_bytes = SoftEtherReader::decode_bytes(transfer_bytes)?;
            let transfer_packets = SoftEtherReader::decode_bytes(transfer_packets)?;

            let session = HubSessionListEntry {
                name: String::from(name),
                vlan_id: String::from(vlan_id),
                location: String::from(location),
                user: String::from(user),
                source: String::from(source),
                connections,
                transfer_bytes,
                transfer_packets,
            };

            sessions.push(session);
        }

        Ok(sessions)
    }
    
    fn decode_hub_session_info(src: &[u8], session: &HubSessionListEntry) -> Result<HubSession, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_reader(src);
        let mut status = HubSession::new();

	    status.name = session.name.to_string();
	    status.vlan_id = session.vlan_id.to_string();
	    status.location = session.location.to_string();
	    status.user = session.user.to_string();
	    status.source = session.source.to_string();
	    status.connections = session.connections;
	    status.transfer_bytes = session.transfer_bytes;
	    status.transfer_packets = session.transfer_packets;
	
        for entry in rdr.records() {
            let entry = entry?;
            let key = entry.get(0).unwrap_or("");
            let val = entry.get(1).unwrap_or("");
            match key.as_ref() {
                "Outgoing Data Size" => {
                    status.outgoing_data_size = SoftEtherReader::decode_bytes(val)?
                }
                "Outgoing Unicast Packets" => {
                    status.outgoing_unicast_packets = SoftEtherReader::decode_packets(val)?
                }
                "Outgoing Unicast Total Size" => {
                    status.outgoing_unicast_bytes = SoftEtherReader::decode_bytes(val)?
                }
                "Outgoing Broadcast Packets" => {
                    status.outgoing_broadcast_packets = SoftEtherReader::decode_packets(val)?
                }
                "Outgoing Broadcast Total Size" => {
                    status.outgoing_broadcast_bytes = SoftEtherReader::decode_bytes(val)?
                }
                "Incoming Data Size" => {
                    status.incoming_data_size = SoftEtherReader::decode_bytes(val)?
                }
                "Incoming Unicast Packets" => {
                    status.incoming_unicast_packets = SoftEtherReader::decode_packets(val)?
                }
                "Incoming Unicast Total Size" => {
                    status.incoming_unicast_bytes = SoftEtherReader::decode_bytes(val)?
                }
                "Incoming Broadcast Packets" => {
                    status.incoming_broadcast_packets = SoftEtherReader::decode_packets(val)?
                }
                "Incoming Broadcast Total Size" => {
                    status.incoming_broadcast_bytes = SoftEtherReader::decode_bytes(val)?
                }
                _ => (),
            }
        }
        Ok(status)
    }
}


#[derive(Debug)]
pub struct HubSessionListEntry {
    pub name: String,
    pub vlan_id: String,
    pub location: String,
    pub user: String,
    pub source: String,
    pub connections: (f64, f64),
    pub transfer_bytes: f64,
    pub transfer_packets: f64,
}

#[derive(Debug)]
pub struct HubSession {
    pub name: String,
    pub vlan_id: String,
    pub location: String,
    pub user: String,
    pub source: String,
    pub connections: (f64, f64),
    pub transfer_bytes: f64,
    pub transfer_packets: f64,
    pub outgoing_data_size: f64,
    pub outgoing_unicast_packets: f64,
    pub outgoing_unicast_bytes: f64,
    pub outgoing_broadcast_packets: f64,
    pub outgoing_broadcast_bytes: f64,
    pub incoming_data_size: f64,
    pub incoming_unicast_packets: f64,
    pub incoming_unicast_bytes: f64,
    pub incoming_broadcast_packets: f64,
    pub incoming_broadcast_bytes: f64,
}

impl HubSession {
    pub fn new() -> HubSession {
        HubSession {
            name: String::from(""),
            vlan_id: String::from(""),
            location: String::from(""),
            user: String::from(""),
            source: String::from(""),
            connections: (0.0, 0.0),
            transfer_bytes: 0.0,
            transfer_packets: 0.0,
            outgoing_data_size: 0.0,
            outgoing_unicast_packets: 0.0,
            outgoing_unicast_bytes: 0.0,
            outgoing_broadcast_packets: 0.0,
            outgoing_broadcast_bytes: 0.0,
            incoming_data_size: 0.0,
            incoming_unicast_packets: 0.0,
            incoming_unicast_bytes: 0.0,
            incoming_broadcast_packets: 0.0,
            incoming_broadcast_bytes: 0.0,
        }
    }
}
