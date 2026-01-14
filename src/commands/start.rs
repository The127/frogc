use std::ffi::CString;
use std::os::unix::fs::chroot;
use nix::libc;
use nix::unistd::{chdir, execvp};
use crate::context::FrogContext;
use crate::errors::ContainerError;
use nix::sched::{CloneFlags, clone};
use nix::unistd::{Pid, fork};
use crate::types::{ContainerSpec, ContainerState};

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

    unsafe {
        let flags = CloneFlags::CLONE_NEWNS
            | CloneFlags::CLONE_NEWPID
            | CloneFlags::CLONE_NEWUTS
            | CloneFlags::CLONE_NEWIPC;

        const STACK_SIZE: usize = 1024 * 1024;
        let mut stack = vec![0u8; STACK_SIZE];

        let child_pid = clone(
            Box::new(|| child_main(container_id.clone(), state.spec.clone())),
            &mut stack,
            flags,
            Some(libc::SIGCHLD),
        ).map_err(ContainerError::wrap)?;

        state.pid = Some(child_pid.as_raw() as u32)
    }

    state.status = "running".to_string();
    context
        .write_state(&container_id, state)
        .map_err(ContainerError::wrap)?;

    Ok(())
}

unsafe fn child_main(id: String, spec: ContainerSpec) -> isize {
    nix::mount::mount(
        None::<&str>,
        "/",
        None::<&str>,
        nix::mount::MsFlags::MS_REC | nix::mount::MsFlags::MS_PRIVATE,
        None::<&str>,
    ).unwrap();

    nix::unistd::sethostname(id).unwrap();

    chroot(spec.rootfs).unwrap();
    chdir("/").unwrap();

    let cmd = CString::new(spec.cmd[0].as_str()).unwrap();
    let args: Vec<CString> = spec.cmd.iter()
        .map(|s| CString::new(s.as_str()).unwrap())
        .collect();

    let result = execvp(&cmd, &args);
    match result {
        Ok(_) => {
            unreachable!()
        }
        Err(_) => {
            1
        }
    }
}
