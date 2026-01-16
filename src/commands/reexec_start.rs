use crate::context::FrogContext;
use crate::errors::{ContainerError, WrapError};
use crate::types::ContainerSpec;
use nix::mount::{MsFlags, mount, umount, umount2, MntFlags};
use nix::unistd::{chdir, execvp, pivot_root};
use std::ffi::CString;
use std::fmt::format;
use std::os::unix::fs::chroot;

pub fn run(context: FrogContext, container_id: String) -> Result<(), ContainerError> {
    let state = context
        .read_state(&container_id)
        .map_err(WrapError::wrapper("reading container state"))
        .map_err(ContainerError::wrap)?;

    mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )
        .map_err(WrapError::wrapper("making mounts private"))
        .map_err(ContainerError::wrap)?;

    pivot_root(state.spec.rootfs.as_str(), format!("{}/mnt", state.spec.rootfs).as_str())
        .map_err(WrapError::wrapper("pivoting root"))
        .map_err(ContainerError::wrap)?;

    mount(
        None::<&str>,
        "/proc",
        Some("proc"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_RELATIME | MsFlags::MS_NODEV,
        None::<&str>,
    )
    .map_err(WrapError::wrapper("mounting proc"))
    .map_err(ContainerError::wrap)?;

    mount(
        None::<&str>,
        "/dev",
        Some("devtmpfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_RELATIME,
        None::<&str>,
    )
    .map_err(WrapError::wrapper("mounting dev"))
    .map_err(ContainerError::wrap)?;

    chdir(state.spec.work_dir.as_deref().unwrap_or("/"))
        .map_err(WrapError::wrapper("changing working directory"))
        .map_err(ContainerError::wrap)?;

    umount2("/mnt", MntFlags::MNT_DETACH)
        .map_err(WrapError::wrapper("umounting old root"))
        .map_err(ContainerError::wrap)?;

    nix::unistd::sethostname(container_id.clone())
        .map_err(WrapError::wrapper("setting hostname"))
        .unwrap();

    exec_container(state.spec);

    unreachable!()
}

fn exec_container(spec: ContainerSpec) {
    let cmd = CString::new(spec.cmd[0].as_str()).unwrap();
    let args: Vec<CString> = spec
        .cmd
        .iter()
        .map(|s| CString::new(s.as_str()).unwrap())
        .collect();

    execvp(&cmd, &args).unwrap();
}
