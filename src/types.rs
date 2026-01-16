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
