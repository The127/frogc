use nix::errno::Errno;
use nix::fcntl::AtFlags;
use nix::libc::{c_int, c_ulong, EBADF};
use nix::{NixPath, libc};
use std::os::fd::{AsRawFd, BorrowedFd};
use nix::mount::MsFlags;

macro_rules! libc_bitflags {
    (
        $(#[$outer:meta])*
        pub struct $BitFlags:ident: $T:ty {
            $(
                $(#[$inner:ident $($args:tt)*])*
                $Flag:ident $(as $cast:ty)*;
            )+
        }
    ) => {
        ::bitflags::bitflags! {
            #[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
            #[repr(transparent)]
            $(#[$outer])*
            pub struct $BitFlags: $T {
                $(
                    $(#[$inner $($args)*])*
                    const $Flag = libc::$Flag $(as $cast)*;
                )+
            }
        }
    };
}

libc_bitflags! {
    pub struct MountAttrFlags: u64 {
        MOUNT_ATTR_RDONLY;
        MOUNT_ATTR_NOSUID;
        MOUNT_ATTR_NODEV;
        MOUNT_ATTR_NOEXEC;
        MOUNT_ATTR_RELATIME;
        MOUNT_ATTR_NOATIME;
        MOUNT_ATTR_STRICTATIME;
        MOUNT_ATTR__ATIME;
        MOUNT_ATTR_NODIRATIME;
        MOUNT_ATTR_NOSYMFOLLOW;
        MOUNT_ATTR_IDMAP;
    }
}

pub struct MountAttr<'a> {
    pub attr_set: MountAttrFlags,
    pub attr_clr: MountAttrFlags,
    pub propagation: MsFlags,
    pub userns_fd: BorrowedFd<'a>,
}

struct MountAttrInternal {
    attr_set: c_ulong,
    attr_clr: c_ulong,
    propagation: c_ulong,
    userns_fd: c_ulong,
}

impl From<&MountAttr<'_>> for MountAttrInternal {
    fn from(mount_attr: &MountAttr) -> Self {
        Self {
            attr_set: mount_attr.attr_set.bits(),
            attr_clr: mount_attr.attr_clr.bits(),
            propagation: mount_attr.propagation.bits(),
            userns_fd: mount_attr.userns_fd.as_raw_fd() as c_ulong,
        }
    }
}

pub fn mount_setattr<P1: ?Sized + NixPath>(
    dirfd: Option<BorrowedFd>,
    path: &P1,
    flags: AtFlags,
    mount_attr: &MountAttr,
) -> nix::Result<()> {
    let res = unsafe {
        libc::syscall(
            libc::SYS_mount_setattr,
            dirfd.map(|x| x.as_raw_fd() as c_int).unwrap_or(-EBADF),
            path,
            flags,
            &MountAttrInternal::from(mount_attr) as *const MountAttrInternal,
            size_of::<MountAttrInternal>(),
        )
    };

    Errno::result(res).map(drop)
}
