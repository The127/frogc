use nix::unistd::execv;
use std::ffi::CString;

pub unsafe fn run(reexec_command: String, args: Vec<String>) -> core::convert::Infallible {
    let exe = CString::from(c"/proc/self/exe");

    let mut cargs = vec![exe.clone()];
    cargs.push(CString::new("re-exec").unwrap());
    cargs.push(CString::new(reexec_command).unwrap());
    cargs.extend(
        args.iter()
            .map(|arg| CString::new(arg.clone()).unwrap()),
    );

    execv(&exe, &cargs).unwrap()
}
