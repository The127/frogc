use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerSpec {
    #[serde(rename = "rootfs")]
    rootfs: String,

    #[serde(rename = "cmd")]
    cmd: Vec<String>,

    #[serde(rename = "cpu")]
    cpu: Option<u32>,

    #[serde(rename = "memory")]
    memory: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
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
