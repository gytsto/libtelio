//! Definition of basic components in a mesh network

use super::EndpointMap as RelayEndpointMap;

use crate::api_config::PathType;
use ipnetwork::IpNetworkError;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom, net::IpAddr};
use telio_crypto::PublicKey;

use super::config::{Config, Peer, PeerBase};

pub use ipnetwork::IpNetwork;
pub use std::net::{Ipv4Addr, SocketAddr};

/// Description of a Node
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Node {
    /// An identifier for a node
    /// Makes it possible to distinguish different nodes in the presence of key reuse
    pub identifier: String,
    /// Public key of the Node
    pub public_key: PublicKey,
    /// State of the node (Connecting, connected, or disconnected)
    pub state: NodeState,
    /// Is the node exit node
    pub is_exit: bool,
    /// Is the node is a vpn server.
    pub is_vpn: bool,
    /// IP addresses of the node
    pub ip_addresses: Vec<IpAddr>,
    /// List of IP's which can connect to the node
    pub allowed_ips: Vec<IpNetwork>,
    /// Endpoint used by node
    pub endpoint: Option<SocketAddr>,
    /// Hostname of the node
    pub hostname: Option<String>,
    /// Flag to control whether the Node allows incoming connections
    pub allow_incoming_connections: bool,
    /// Flag to control whether the Node allows incoming files
    pub allow_peer_send_files: bool,
    /// Connection type in the network mesh (through Relay or hole punched directly)
    pub path: PathType,
}

/// Description of the Exit Node
/// It is the gateway node to the internet
#[derive(Debug, Default, Clone, Serialize)]
pub struct ExitNode {
    /// An identifier for an exit node
    /// Makes it possible to distinguish different exit nodes in the presence of key reuse
    pub identifier: String,
    /// The public key of the exit node
    pub public_key: PublicKey,
    /// List of all allowed Ip's for the Exit Node
    pub allowed_ips: Option<Vec<IpNetwork>>,
    /// Socket address of the Exit Node
    pub endpoint: Option<SocketAddr>,
}

/// Connection state of the node
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeState {
    /// Node is disconnected
    #[default]
    Disconnected,
    /// Trying to connect to the Node
    Connecting,
    /// Node is connected
    Connected,
}

/// Network mesh map of all the nodes
#[derive(Debug, Default)]
pub struct Map {
    /// Public key of self
    pub public_key: PublicKey,
    /// The list of Ip's that can communicate with the device
    pub allowed_ips: Vec<IpNetwork>,
    /// Hash map of all the nodes in the network mesh
    pub nodes: HashMap<PublicKey, Node>,
}

impl TryFrom<&ExitNode> for Node {
    type Error = IpNetworkError;

    fn try_from(other: &ExitNode) -> Result<Self, IpNetworkError> {
        let address_v4 = "0.0.0.0/0".parse()?;
        let address_v6 = "::/0".parse()?;
        Ok(Self {
            public_key: other.public_key,
            is_exit: true,
            is_vpn: other.endpoint.is_some(),
            allowed_ips: other
                .allowed_ips
                .as_ref()
                .cloned()
                .unwrap_or_else(|| vec![address_v4, address_v6]),
            endpoint: other.endpoint,
            ..Default::default()
        })
    }
}

impl From<&PeerBase> for Node {
    fn from(peer: &PeerBase) -> Self {
        Self {
            identifier: peer.identifier.clone(),
            public_key: peer.public_key,
            allowed_ips: peer
                .ip_addresses
                .as_ref()
                .map(|ips| ips.iter().map(|a| (*a).into()).collect())
                .unwrap_or_default(),
            hostname: Some(peer.hostname.to_owned()),
            ..Default::default()
        }
    }
}

impl From<&Peer> for Node {
    fn from(peer: &Peer) -> Self {
        Self {
            public_key: peer.public_key,
            allowed_ips: peer
                .ip_addresses
                .as_ref()
                .map(|ips| ips.iter().map(|a| (*a).into()).collect())
                .unwrap_or_default(),
            hostname: Some(peer.hostname.to_owned()),
            allow_incoming_connections: peer.allow_incoming_connections,
            allow_peer_send_files: peer.allow_peer_send_files,
            ..Default::default()
        }
    }
}

impl Map {
    /// Flags the relay nodes in the network mesh map
    pub fn set_relay_endpoints(&mut self, endpoints: RelayEndpointMap) {
        for (key, endpoint) in endpoints {
            // TODO: check if endpoint already contains primary, assume it should stay primary
            //      think, how to check for old derp endpoint.
            if let Some(node) = self.nodes.get_mut(&key) {
                node.endpoint = Some(endpoint);
            }
        }
    }
}

impl From<&Config> for Map {
    fn from(c: &Config) -> Self {
        let Node {
            public_key,
            allowed_ips,
            ..
        } = From::<&PeerBase>::from(c);
        Self {
            public_key,
            allowed_ips,
            nodes: c
                .peers
                .as_ref()
                .map(|peers| {
                    peers
                        .iter()
                        .map(|p| (p.public_key, From::<&Peer>::from(p)))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}
