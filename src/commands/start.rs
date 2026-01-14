use crate::context::FrogContext;
use crate::errors::ContainerError;

pub fn run(
    context: FrogContext,
    container_id: String,
) -> Result<(), ContainerError> {
    let (exists, _lock) = context.lock_container(&container_id).map_err(ContainerError::wrap)?;
    if !exists {
        return Err(ContainerError::NotFound);
    }

    let mut state = context.read_state(&container_id).map_err(ContainerError::wrap)?;
    if state.status != "stopped" {
        return Err(ContainerError::InvalidState("Container is not stopped".to_string()));
    }

    // TODO: start the container

    state.status = "running".to_string();
    context.write_state(&container_id, state).map_err(ContainerError::wrap)?;

    Ok(())
}