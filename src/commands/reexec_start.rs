use std::ffi::CString;
use std::fs;
use std::os::unix::fs::chroot;
use log::log;
use nix::unistd::{chdir, execvp};
use crate::context::FrogContext;
use crate::errors::ContainerError;
use crate::types::ContainerSpec;

pub fn run(context: FrogContext, container_id: String) -> Result<(), ContainerError> {
    println!("rexec run");
    log::info!("Re-executing start command");
    fs::create_dir_all("./test").map_err(ContainerError::wrap)?;
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
