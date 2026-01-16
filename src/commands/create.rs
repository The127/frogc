use crate::context::FrogContext;
use crate::errors::ContainerError;
use crate::spec;
use std::io::Read;
use std::{fs, io};

pub fn run(
    context: FrogContext,
    spec_path: String,
    container_id: String,
) -> Result<(), ContainerError> {
    // read the spec
    let spec_content = if spec_path == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).map_err(ContainerError::wrap)?;
        buffer
    } else {
        fs::read_to_string(&spec_path).map_err(ContainerError::wrap)?
    };

    let spec: spec::ContainerSpec = serde_json::from_str(&spec_content).map_err(ContainerError::wrap)?;

    let (exists, _lock) = context.lock_container(&container_id).map_err(ContainerError::wrap)?;
    if exists {
        return Err(ContainerError::AlreadyExists);
    }

    let state = spec::ContainerState {
        id: container_id.clone(),
        spec,
        status: "stopped".to_string(),
        pid: None,
    };

    context.write_state(&container_id, state).map_err(ContainerError::wrap)?;

    Ok(())
}
