use std::{fs, io};
use std::fs::File;
use std::io::Read;
use fs2::FileExt;
use crate::types;
use crate::context::FrogContext;

pub fn run(context: FrogContext, spec_path: String, container_id: String) -> Result<(), Box<dyn std::error::Error>> {
    // read the spec
    let spec_content = if spec_path == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&spec_path)?
    };

    let spec: types::ContainerSpec = serde_json::from_str(&spec_content)?;

    let _lock = context.lock_container(&container_id)?;
    let state = types::ContainerState {
        id: container_id.clone(),
        spec,
        status: "created".to_string(),
    };

    context.write_state(&container_id, state)?;

    Ok(())
}