use std::os::fd::{BorrowedFd, RawFd};
use bitflags::Flags;
use nix::libc::{EBADF, MS_ASYNC};
use nix::mount::MsFlags;
use crate::spec;
use crate::syscall::{MountAttr, MountAttrFlags};

pub struct Mount<'a> {
    pub source: String,
    pub destination: String,
    pub fs_type: String,
    pub flags: MsFlags,
    pub mount_attr: MountAttr<'a>,
    pub options: Option<String>,
    pub tmp_copy_up: bool,
}

impl From<&spec::Mount> for Mount<'_> {
    fn from(mount: &spec::Mount) -> Self {
        let mut tmp_copy_up = false;
        let mut mount_flags = MsFlags::empty();
        let mut propagation_flags = MsFlags::empty();
        let mut recursive_flags = MountAttrFlags::empty();
        let mut recursive_clear_flags = MountAttrFlags::empty();
        let mut options : Vec<String> = Vec::new();

        for option in &mount.options {
            match option.as_str() {
                "async" => mount_flags.set(MsFlags::MS_BIND, false),
                "atime" => mount_flags.set(MsFlags::MS_BIND, false),
                "bind" => mount_flags.set(MsFlags::MS_BIND, true),
                "defaults" => (), // ignored for now
                "dev" => mount_flags.set(MsFlags::MS_NODEV, false),
                "diratime" => mount_flags.set(MsFlags::MS_NODIRATIME, false),
                "dirsync" => mount_flags.set(MsFlags::MS_DIRSYNC, true),
                "exec" => mount_flags.set(MsFlags::MS_NOEXEC, false),
                "iversion" => mount_flags.set(MsFlags::MS_I_VERSION, true),
                "lazytime" => mount_flags.set(MsFlags::MS_LAZYTIME, true),
                "loud" => mount_flags.set(MsFlags::MS_SILENT, false),
                "mand" => mount_flags.set(MsFlags::MS_MANDLOCK, true),
                "noatime" => mount_flags.set(MsFlags::MS_NOATIME, true),
                "nodev" => mount_flags.set(MsFlags::MS_NODEV, true),
                "nodiratime" => mount_flags.set(MsFlags::MS_NODIRATIME, true),
                "noexec" => mount_flags.set(MsFlags::MS_NOEXEC, true),
                "noiversion" => mount_flags.set(MsFlags::MS_I_VERSION, false),
                "nolazytime" => mount_flags.set(MsFlags::MS_LAZYTIME, false),
                "nomand" => mount_flags.set(MsFlags::MS_MANDLOCK, false),
                "norelatime" => mount_flags.set(MsFlags::MS_RELATIME, false),
                "nostrictatime" => mount_flags.set(MsFlags::MS_STRICTATIME, false),
                "nosuid" => mount_flags.set(MsFlags::MS_NOSUID, true),
                "nosymfollow" => (), // SHOULD => we dont support this option for now
                "private" => mount_flags.set(MsFlags::MS_PRIVATE, true),
                "ratime" => recursive_clear_flags.set(MountAttrFlags::MOUNT_ATTR__ATIME, true),
                "rbind" => {
                    mount_flags.set(MsFlags::MS_BIND, true);
                    mount_flags.set(MsFlags::MS_REC, true);
                }
                "rdev" => recursive_clear_flags.set(MountAttrFlags::MOUNT_ATTR_NODEV, true),
                "rdiratime" => recursive_clear_flags.set(MountAttrFlags::MOUNT_ATTR_NODIRATIME, true),
                "relatime" => mount_flags.set(MsFlags::MS_RELATIME, true),
                "remount" => mount_flags.set(MsFlags::MS_REMOUNT, true),
                "rexec" => recursive_clear_flags.set(MountAttrFlags::MOUNT_ATTR_NOEXEC, true),
                "rnoatime" => mount_flags.set(MsFlags::MS_NOATIME, true),
                "rnodiratime" => recursive_flags.set(MountAttrFlags::MOUNT_ATTR_NODIRATIME, true),
                "rnoexec" => recursive_flags.set(MountAttrFlags::MOUNT_ATTR_NOEXEC, true),
                "rnorelatime" => recursive_clear_flags.set(MountAttrFlags::MOUNT_ATTR_RELATIME, true),
                "rnostrictatime" => recursive_clear_flags.set(MountAttrFlags::MOUNT_ATTR_STRICTATIME, true),
                "rnosuid" => recursive_flags.set(MountAttrFlags::MOUNT_ATTR_NOSUID, true),
                "rnosymfollow" => recursive_flags.set(MountAttrFlags::MOUNT_ATTR_NOSYMFOLLOW, true),
                "ro" => mount_flags.set(MsFlags::MS_RDONLY, true),
                "rprivate" => {
                    propagation_flags.set(MsFlags::MS_PRIVATE, true);
                    propagation_flags.set(MsFlags::MS_REC, true);
                }
                "rrelatime" => recursive_flags.set(MountAttrFlags::MOUNT_ATTR_RELATIME, true),
                "rro" => recursive_flags.set(MountAttrFlags::MOUNT_ATTR_RDONLY, true),
                "rrw" => recursive_clear_flags.set(MountAttrFlags::MOUNT_ATTR_RDONLY, true),
                "rshared" => {
                    propagation_flags.set(MsFlags::MS_SHARED, true);
                    propagation_flags.set(MsFlags::MS_REC, true);
                }
                "rslave" => {
                    propagation_flags.set(MsFlags::MS_SLAVE, true);
                    propagation_flags.set(MsFlags::MS_REC, true);
                }
                "rstrictatime" => recursive_flags.set(MountAttrFlags::MOUNT_ATTR_STRICTATIME, true),
                "rsuid" => recursive_clear_flags.set(MountAttrFlags::MOUNT_ATTR_NOSUID, true),
                "rsymfollow" => recursive_clear_flags.set(MountAttrFlags::MOUNT_ATTR_NOSYMFOLLOW, true),
                "runbindable" => propagation_flags.set(MsFlags::MS_UNBINDABLE, true),
                "rw" => mount_flags.set(MsFlags::MS_RDONLY, false),
                "shared" => propagation_flags.set(MsFlags::MS_SHARED, true),
                "silent" => mount_flags.set(MsFlags::MS_SILENT, true),
                "slave" => propagation_flags.set(MsFlags::MS_SLAVE, true),
                "strictatime" => mount_flags.set(MsFlags::MS_STRICTATIME, true),
                "suid" => mount_flags.set(MsFlags::MS_NOSUID, false),
                "symfollow" => (), // ignored for now
                "sync" => mount_flags.set(MsFlags::MS_SYNCHRONOUS, true),
                "tmpcopyup" => tmp_copy_up = true,
                "unbindable" => mount_flags.set(MsFlags::MS_UNBINDABLE, true),
                "idmap" => (), // ignored for now
                "ridmap" => (),
                _ => options.push(option.clone()), // unknown options are supposed to be treated as filesystem specific options
            }
        }

        unsafe {
            Mount {
                source: mount.source.clone(),
                destination: mount.destination.clone(),
                fs_type: mount.fs_type.clone(),
                flags: MsFlags::empty(),
                mount_attr: MountAttr {
                    attr_clr: recursive_clear_flags,
                    attr_set: recursive_flags,
                    propagation: propagation_flags,
                    userns_fd: BorrowedFd::borrow_raw(RawFd::from(-EBADF)),
                },
                options: if options.is_empty() { None } else { Some(options.join(",")) },
                tmp_copy_up,
            }
        }
    }
}
