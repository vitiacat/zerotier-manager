use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Deserialize, Debug)]
pub struct StatusResponse {
    pub address: String,
    pub online: bool,
    #[serde(rename = "versionBuild")]
    pub version_build: u32,
    #[serde(rename = "versionMajor")]
    pub version_major: u32,
    #[serde(rename = "versionMinor")]
    pub version_minor: u32,
    #[serde(rename = "versionRev")]
    pub version_rev: u32,
}

#[derive(Deserialize, Debug)]
pub struct NetworkResponse {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "v4AssignMode")]
    pub v4_assign_mode: NetworkV4AssignMode,
    #[serde(rename = "creationTime")]
    pub creation_time: i64,
    pub private: bool,
    #[serde(rename = "enableBroadcast")]
    pub enable_broadcast: bool,
    pub mtu: u32,
    pub routes: Vec<NetworkRoute>,
    #[serde(rename = "ipAssignmentPools")]
    pub ip_assignment_pools: Vec<NetworkIPAssignmentPool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkV4AssignMode {
    pub zt: bool,
}

impl Display for NetworkResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = if self.name.is_none() || self.name.as_ref().unwrap().is_empty() {
            self.id.clone()
        } else {
            format!("{} ({})", self.name.as_ref().unwrap(), self.id)
        };
        write!(f, "{}", str)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkRoute {
    pub target: String,
    pub via: Option<String>,
}

impl Display for NetworkRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} via {}",
            self.target,
            self.via.as_ref().unwrap_or(&"(null)".to_string())
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkIPAssignmentPool {
    #[serde(rename = "ipRangeStart")]
    pub ip_range_start: String,
    #[serde(rename = "ipRangeEnd")]
    pub ip_range_end: String,
}

impl Display for NetworkIPAssignmentPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.ip_range_start, self.ip_range_end)
    }
}

#[derive(Deserialize, Debug)]
pub struct NetworkDNS {
    pub domain: String,
    pub servers: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MemberResponse {
    pub id: String,
    pub nwid: String,
    pub name: Option<String>,
    pub authorized: bool,
    #[serde(rename = "authenticationExpiryTime")]
    pub authentication_expiry_time: u64,
    #[serde(rename = "creationTime")]
    pub creation_time: u64,
    #[serde(rename = "lastAuthorizedTime")]
    pub last_authorized_time: u64,
    #[serde(rename = "lastDeauthorizedTime")]
    pub last_deauthorized_time: u64,
    #[serde(rename = "ipAssignments")]
    pub ip_assignments: Vec<String>,
    pub tags: Vec<String>,
}

impl Display for MemberResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ips = self.ip_assignments.join(", ");
        write!(
            f,
            "{}",
            if self.name.is_some() {
                format!(
                    "{} ({} ** {})",
                    self.name.as_ref().unwrap(),
                    self.id,
                    if self.ip_assignments.is_empty() {
                        "No IPs"
                    } else {
                        &*ips
                    }
                )
            } else {
                format!(
                    "{} ({})",
                    self.id,
                    if self.ip_assignments.is_empty() {
                        "No IPs"
                    } else {
                        &*ips
                    }
                )
            },
        )
    }
}
