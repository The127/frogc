use std::{fs, io};
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};
use fs2::FileExt;

#[derive(Debug, Serialize, Deserialize)]
struct ContainerSpec {
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
struct ContainerState {
    #[serde(rename = "id")]
    id: String,

    #[serde(rename = "spec")]
    spec: ContainerSpec,

    #[serde(rename = "status")]
    status: String,
}

pub fn run(spec_path: String, container_id: String) -> Result<(), Box<dyn std::error::Error>> {
    // read the spec
    log::info!("reading spec");
    let spec_content = if spec_path == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&spec_path)?
    };

    // parse the spec
    log::info!("parsing spec");
    let spec: ContainerSpec = serde_json::from_str(&spec_content)?;

    // create the run directory
    log::info!("creating run directory");
    let run_dir = format!("/run/frogc/{}", container_id);
    fs::create_dir_all(&run_dir)?;

    // lock the container directory
    log::info!("locking container directory");
    let lock_file_path = format!("{}/lock", run_dir);
    let lock_file = File::create(lock_file_path)?;
    lock_file.try_lock_exclusive()?; // auto unlock on drop

    // write the state file
    log::info!("writing state file");
    let state_file_path = format!("{}/state.json", run_dir);
    let state_file = File::create(state_file_path)?;
    let state = ContainerState {
        id: container_id.clone(),
        spec,
        status: "created".to_string(),
    };
    serde_json::to_writer_pretty(&state_file, &state)?;

    // dummy start
    log::info!("container created");

    Ok(())
}