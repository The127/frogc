use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerSpec {
    #[serde(rename = "rootfs")]
    pub rootfs: String,

    #[serde(rename = "workDir")]
    pub work_dir: Option<String>,

    #[serde(rename = "cmd")]
    pub cmd: Vec<String>,

    #[serde(rename = "cpu")]
    pub cpu: Option<u32>,

    #[serde(rename = "memory")]
    pub memory: Option<u64>,

    #[serde(rename = "mounts")]
    pub mounts: Vec<Mount>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mount {
    #[serde(rename = "destination")]
    pub destination: String,

    #[serde(rename = "type")]
    pub fs_type: String,

    #[serde(rename = "source")]
    pub source: String,
    pub rbind: Option<bool>,
    #[serde(rename = "options")]
    pub options: Vec<String>,

    #[serde(rename = "uidMappings")]
    pub uid_mappings: Vec<LinuxIdMapping>,

    #[serde(rename = "gidMappings")]
    pub gid_mappings: Vec<LinuxIdMapping>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinuxIdMapping {
    pub container_id: u32,
    pub host_id: u32,
    pub size: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerState {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "spec")]
    pub spec: ContainerSpec,

    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "pid")]
    pub pid: Option<u32>,
}
