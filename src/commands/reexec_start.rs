use crate::context::FrogContext;
use crate::errors::{ContainerError, WrapError};
use crate::types::{ContainerSpec, ContainerState};
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

    setup_mounts(&state)?;

    nix::unistd::sethostname(container_id.clone())
        .map_err(WrapError::wrapper("setting hostname"))
        .unwrap();

    exec_container(state.spec);

    unreachable!()
}

fn setup_mounts(state: &ContainerState) -> Result<(), ContainerError> {
    // when calling copy the child process inherits the mount information
    // we want to make the mounts private so that any changes are isolated/not being propagated to the parent
    mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )
        .map_err(WrapError::wrapper("making mounts private"))
        .map_err(ContainerError::wrap)?;

    // we use pivot_root to set up the new root fs
    // the old root fs is moved to /mnt and must later be unmounted
    // we cannot use chroot because that could be escaped
    pivot_root(state.spec.rootfs.as_str(), format!("{}/mnt", state.spec.rootfs).as_str())
        .map_err(WrapError::wrapper("pivoting root"))
        .map_err(ContainerError::wrap)?;

    // mount required system mounts
    mount(
        None::<&str>,
        "/proc",
        Some("proc"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_RELATIME | MsFlags::MS_NODEV | MsFlags::MS_PRIVATE | MsFlags::MS_REC,
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

    // we change the working directory to the container's working directory before unmounting the old root
    chdir(state.spec.work_dir.as_deref().unwrap_or("/"))
        .map_err(WrapError::wrapper("changing working directory"))
        .map_err(ContainerError::wrap)?;

    // unmount the old root with umount2 because we need to use MNT_DETACH to lazily unmount the filesystem
    // otherwise the umount call will fail with EBUSY
    // this is fine as the container will not have any access to the original filesystem anymore
    umount2("/mnt", MntFlags::MNT_DETACH)
        .map_err(WrapError::wrapper("umounting old root"))
        .map_err(ContainerError::wrap)?;

    Ok(())
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
