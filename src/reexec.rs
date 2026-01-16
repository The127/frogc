use crate::cli::ReExecCommands;
use crate::errors::ContainerError;
use nix::libc;
use nix::sched::{CloneFlags, clone};
use nix::unistd::{execv, Pid};
use std::ffi::CString;

pub fn run(command: ReExecCommands) -> nix::Result<Pid> {
    let (cmd, args) = match command {
        ReExecCommands::Start { container_id } => {
            ("start".to_string(), vec![container_id])
        },
    };

    let flags = CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWUTS
        | CloneFlags::CLONE_NEWIPC;

    const STACK_SIZE: usize = 1024 * 1024;
    let mut stack = vec![0u8; STACK_SIZE];

    unsafe {
        clone(
            Box::new(|| child_main(cmd.clone(), args.clone())),
            &mut stack,
            flags,
            Some(libc::SIGCHLD),
        )
    }
}

unsafe fn child_main(reexec_command: String, args: Vec<String>) -> isize {
    let exe = CString::from(c"/proc/self/exe");

    let mut cargs = vec![exe.clone()];
    cargs.push(CString::new("re-exec").unwrap());
    cargs.push(CString::new(reexec_command).unwrap());
    cargs.extend(args.iter().map(|arg| CString::new(arg.clone()).unwrap()));

    execv(&exe, &cargs).unwrap();

    unreachable!()
}
