use crate::context::FrogContext;
use crate::errors::{ContainerError, WrapError};
use crate::spec::{ContainerSpec, ContainerState};
use nix::libc;
use nix::libc::{mount_attr, O_CLOEXEC, O_DIRECTORY, O_PATH};
use nix::mount::{mount, umount2, MntFlags, MsFlags};
use nix::sys::stat::{mknod, Mode};
use nix::unistd::{chdir, execvp, fchdir, pivot_root};
use std::ffi::CString;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use crate::types;
use crate::types::Mount;

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

    // we prepare the pivot_root call by acquiring file descriptors for the old and new root directory
    // we must use the flags below so that we open the directory and get the file descriptor
    let old_root = OpenOptions::new()
        .custom_flags(O_PATH | O_DIRECTORY | O_CLOEXEC)
        .read(true)
        .open("/")
        .map_err(WrapError::wrapper("opening old root"))
        .map_err(ContainerError::wrap)?;

    let new_root = OpenOptions::new()
        .custom_flags(O_PATH | O_DIRECTORY | O_CLOEXEC)
        .read(true)
        .open(state.spec.rootfs.as_str())
        .map_err(WrapError::wrapper("opening new root"))
        .map_err(ContainerError::wrap)?;

    fchdir(&new_root)
        .map_err(WrapError::wrapper("changing working directory to rootfs"))
        .map_err(ContainerError::wrap)?;

    // we use pivot_root to set up the new root fs
    // since we have chdir'd into the rootfs directory we can use "." to refer to it
    // passing in "." and "." effectively layers the old and new rootfs on top of each other
    // when we later umount the old root, its data is no longer accessible
    // this technique is necessary because we cannot guarantee that the container's root fs is writable or has a /mnt directory (or any directory really)
    // we cannot use chroot because that does not give us the required isolation
    pivot_root(".", ".")
        .map_err(WrapError::wrapper("pivoting root"))
        .map_err(ContainerError::wrap)?;

    // we change the working directory to the old root fs using the file descriptor
    // this is necessary because the following umount call does not work with file descriptors, only with paths
    fchdir(&old_root)
        .map_err(WrapError::wrapper(
            "changing working directory to old rootfs",
        ))
        .map_err(ContainerError::wrap)?;

    // unmount the old root with umount2 because we need to use MNT_DETACH to lazily unmount the filesystem
    // "." refers to the current working directory, aka the old root
    // otherwise the umount call will fail with EBUSY
    // this is fine as the container will not have any access to the original filesystem anymore
    umount2(".", MntFlags::MNT_DETACH)
        .map_err(WrapError::wrapper("umounting old root"))
        .map_err(ContainerError::wrap)?;

    // we change the working directory back to the new rootfs so all the following mount calls will mount in the correct fs
    fchdir(&new_root)
        .map_err(WrapError::wrapper("changing working directory to rootfs"))
        .map_err(ContainerError::wrap)?;

    // we now mount new rootfs with shared settings, since it can no longer affect the old root
    // we do this because some software expects this
    mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_SHARED | MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(WrapError::wrapper("making mounts shared"))
    .map_err(ContainerError::wrap)?;

    for m in state.spec.mounts.iter().map(Mount::from) {
        mount(
            Some(m.destination.as_str()),
            m.destination.as_str(),
            Some(m.fs_type.as_str()),
            m.flags,
            m.options.as_deref(),
        )
            .map_err(WrapError::wrapper("mounting proc"))
            .map_err(ContainerError::wrap)?;
    }

    mknod(
        "/dev/null",
        nix::sys::stat::SFlag::S_IFCHR,
        Mode::from_bits_truncate(0o777),
        libc::makedev(1, 3),
    )
    .map_err(WrapError::wrapper("creating /dev/null"))
    .map_err(ContainerError::wrap)?;

    mknod(
        "/dev/zero",
        nix::sys::stat::SFlag::S_IFCHR,
        Mode::from_bits_truncate(0o777),
        libc::makedev(1, 5),
    )
    .map_err(WrapError::wrapper("creating /dev/zero"))
    .map_err(ContainerError::wrap)?;

    mknod(
        "/dev/full",
        nix::sys::stat::SFlag::S_IFCHR,
        Mode::from_bits_truncate(0o777),
        libc::makedev(1, 7),
    )
    .map_err(WrapError::wrapper("creating /dev/full"))
    .map_err(ContainerError::wrap)?;

    mknod(
        "/dev/random",
        nix::sys::stat::SFlag::S_IFCHR,
        Mode::from_bits_truncate(0o777),
        libc::makedev(1, 8),
    )
    .map_err(WrapError::wrapper("creating /dev/random"))
    .map_err(ContainerError::wrap)?;

    mknod(
        "/dev/urandom",
        nix::sys::stat::SFlag::S_IFCHR,
        Mode::from_bits_truncate(0o777),
        libc::makedev(1, 9),
    )
    .map_err(WrapError::wrapper("creating /dev/urandom"))
    .map_err(ContainerError::wrap)?;

    mknod(
        "/dev/tty",
        nix::sys::stat::SFlag::S_IFCHR,
        Mode::from_bits_truncate(0o777),
        libc::makedev(5, 0),
    )
    .map_err(WrapError::wrapper("creating /dev/tty"))
    .map_err(ContainerError::wrap)?;

    mount(
        Some("sysfs"),
        "/sys",
        Some("sysfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_RELATIME | MsFlags::MS_RDONLY | MsFlags::MS_NODEV,
        None::<&str>,
    )
    .map_err(WrapError::wrapper("mounting sys"))
    .map_err(ContainerError::wrap)?;

    // we change the working directory to the container's working directory before unmounting the old root
    chdir(state.spec.work_dir.as_deref().unwrap_or("/"))
        .map_err(WrapError::wrapper("changing working directory"))
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
