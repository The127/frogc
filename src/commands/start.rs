use crate::cli::ReExecCommands;
use crate::context::FrogContext;
use crate::errors::ContainerError;
use crate::reexec;

pub fn run(context: FrogContext, container_id: String) -> Result<(), ContainerError> {
    let (exists, _lock) = context
        .lock_container(&container_id)
        .map_err(ContainerError::wrap)?;
    if !exists {
        return Err(ContainerError::NotFound);
    }

    let mut state = context
        .read_state(&container_id)
        .map_err(ContainerError::wrap)?;
    if state.status != "stopped" {
        return Err(ContainerError::InvalidState(
            "Container is not stopped".to_string(),
        ));
    }

    let child_pid = reexec::run(ReExecCommands::Start {
        container_id: container_id.clone(),
    }).map_err(ContainerError::wrap)?;

    nix::sys::wait::waitpid(child_pid, None).map_err(ContainerError::wrap)?;

    state.pid = Some(child_pid.as_raw() as u32);

    state.status = "running".to_string();
    context
        .write_state(&container_id, state)
        .map_err(ContainerError::wrap)?;

    Ok(())
}
