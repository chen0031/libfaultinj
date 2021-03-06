extern crate libc;
extern crate errno;
extern crate rand;
extern crate dylib;

#[macro_use]
extern crate lazy_static;


pub use libc::{c_char, c_int, c_ulong, c_void, off_t, size_t, mode_t, ssize_t};

#[macro_use]
mod errors;
use errors::{OpenFunc, ReadFunc, WriteFunc, SeekFunc, CloseFunc, MmapFunc, Dup2Func, Dup3Func,
             IoctlFunc, BindFunc, StatFunc, FstatFunc, SocketFunc, ConnectFunc, SendRecvFunc,
             ERR_FDS, DELAY_FDS};
use self::errors::matches_addr;
use errors::{remove_fd_if_present, add_fd_if_old_present};

// These functions are designed to conform to their
//  libc counterparts, but may instead inject errors
//  depending on conditions defined in various environment
//  variables.


#[no_mangle]
pub extern "C" fn open64(filename_: *const c_char, flags: c_int, mode: mode_t) -> c_int {
    do_open!(filename_, flags, mode)
}

#[no_mangle]
pub extern "C" fn open(filename_: *const c_char, flags: c_int, mode: mode_t) -> c_int {
    do_open!(filename_, flags, mode)
}

#[no_mangle]
pub extern "C" fn creat(filename_: *const c_char, mode: mode_t) -> c_int {
    const FLAGS: c_int = libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC;

    // TODO: manpage says this is equivalent to creat() but
    //       should we just call creat() instead?
    do_open!(filename_, FLAGS, mode)
}

const SSIZE_ERR: ssize_t = -1isize;

#[no_mangle]
pub extern "C" fn read(fd: c_int, buf: *mut c_void, nbytes: c_int) -> ssize_t {
    lazy_static! {
        static ref READ_FUNC: ReadFunc = get_libc_func!(ReadFunc, "read");
    }

    injectFaults!(fd, "read", SSIZE_ERR);


    READ_FUNC(fd, buf, nbytes)
}

#[no_mangle]
pub extern "C" fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t {
    lazy_static! {
        static ref SEEK_FUNC: SeekFunc = get_libc_func!(SeekFunc, "lseek");
    }

    injectFaults!(fd, "lseek", -1 as i64);

    SEEK_FUNC(fd, offset, whence)
}

#[no_mangle]
pub extern "C" fn lseek64(fd: c_int, offset: off_t, whence: c_int) -> off_t {
    lazy_static! {
        static ref SEEK_FUNC: SeekFunc = get_libc_func!(SeekFunc, "lseek64");
    }

    injectFaults!(fd, "lseek64", -1 as i64);

    SEEK_FUNC(fd, offset, whence)
}


#[no_mangle]
pub extern "C" fn write(fd: c_int, buf: *mut c_void, nbytes: c_int) -> ssize_t {
    lazy_static! {
        static ref WRITE_FUNC: WriteFunc = get_libc_func!(WriteFunc, "write");
    }

    injectFaults!(fd, "write", SSIZE_ERR);

    WRITE_FUNC(fd, buf, nbytes)
}


#[no_mangle]
pub extern "C" fn close(fd: c_int) -> c_int {
    lazy_static! {
        static ref CLOSE_FUNC: CloseFunc = get_libc_func!(CloseFunc, "close");
    }

    remove_fd_if_present(fd);

    CLOSE_FUNC(fd)
}

// For now we don't intercept this call for error injection,
//   only for fd tracking.
#[no_mangle]
pub extern "C" fn dup2(oldfd: c_int, newfd: c_int) -> c_int {
    lazy_static! {
        static ref DUP2_FUNC: Dup2Func = get_libc_func!(Dup2Func, "dup2");
    }

    add_fd_if_old_present(oldfd, newfd);

    DUP2_FUNC(oldfd, newfd)
}

// For now we don't intercept this call for error injection,
//   only for fd tracking.
#[no_mangle]
pub extern "C" fn dup3(oldfd: c_int, newfd: c_int, flags: c_int) -> c_int {
    lazy_static! {
        static ref DUP3_FUNC: Dup3Func = get_libc_func!(Dup3Func, "dup3");
    }

    add_fd_if_old_present(oldfd, newfd);

    DUP3_FUNC(oldfd, newfd, flags)
}

#[no_mangle]
pub extern "C" fn ioctl(fd: c_int, req: c_ulong, argp: *mut c_char) -> c_int {
    lazy_static! {
        static ref IOCTL_FUNC: IoctlFunc = get_libc_func!(IoctlFunc, "ioctl");
    }

    injectFaults!(fd, "ioctl", -1 as c_int);

    IOCTL_FUNC(fd, req, argp)
}

use libc::sockaddr;
use self::errors::socklen_t;
#[no_mangle]
pub extern "C" fn connect(sockfd: c_int, addr: *const sockaddr, addrlen: socklen_t) -> c_int {
    lazy_static! {
        static ref CONNECT_FUNC: ConnectFunc = get_libc_func!(ConnectFunc, "connect");
    }

    if unsafe { matches_addr(addr, "LIBFAULTINJ_ERROR_PATH") } {
        ERR_FDS.write().unwrap().insert(sockfd);
    }

    if unsafe { matches_addr(addr, "LIBFAULTINJ_DELAY_PATH") } {
        DELAY_FDS.write().unwrap().insert(sockfd);
    }

    CONNECT_FUNC(sockfd, addr, addrlen)
}

#[no_mangle]
pub extern "C" fn send(sockfd: c_int, buf: *mut c_void, len: size_t, flags: c_int) -> ssize_t {
    lazy_static! {
        static ref SEND_FUNC: SendRecvFunc = get_libc_func!(SendRecvFunc, "send");
    }

    injectFaults!(sockfd, "send", -1);

    SEND_FUNC(sockfd, buf, len, flags)
}

#[no_mangle]
pub extern "C" fn recv(sockfd: c_int, buf: *mut c_void, len: size_t, flags: c_int) -> ssize_t {
    lazy_static! {
        static ref RECV_FUNC: SendRecvFunc = get_libc_func!(SendRecvFunc, "recv");
    }

    injectFaults!(sockfd, "recv", -1);


    RECV_FUNC(sockfd, buf, len, flags)
}


#[no_mangle]
pub extern "C" fn bind(sockfd: c_int, addr: *const sockaddr, addrlen: u8) -> c_int {
    lazy_static! {
        static ref BIND_FUNC: BindFunc = get_libc_func!(BindFunc, "bind");
    }

    if unsafe { matches_addr(addr, "LIBFAULTINJ_ERROR_PATH") } {
        ERR_FDS.write().unwrap().insert(sockfd);
    }

    if unsafe { matches_addr(addr, "LIBFAULTINJ_DELAY_PATH") } {
        DELAY_FDS.write().unwrap().insert(sockfd);
    }

    BIND_FUNC(sockfd, addr, addrlen)
}

#[no_mangle]
pub extern "C" fn fstat(fd: c_int, buf: *const libc::stat) -> c_int {
    lazy_static! {
        static ref FSTAT_FUNC: FstatFunc = get_libc_func!(FstatFunc, "fstat"); // fixme
    }

    injectFaults!(fd, "fstat", -1);

    FSTAT_FUNC(fd, buf)
}



// DISABLED -- these calls are disabled until problems caused can be addressed
#[no_mangle]
#[allow(private_no_mangle_fns)]
#[allow(dead_code)]
#[allow(unused_variables)]
extern "C" fn stat(pathname: *const c_char, buf: *mut libc::stat) -> c_int {
    lazy_static! {
        static ref STAT_FUNC: StatFunc = get_libc_func!(StatFunc, "stat"); // fixme
    }
    // TO BE IMPLEMENTED
    STAT_FUNC(pathname, buf) - 1
}


#[no_mangle]
#[allow(private_no_mangle_fns)]
#[allow(dead_code)]
#[allow(unused_variables)]
extern "C" fn socket(domain: c_int, type_: c_int, protocol: c_int) -> c_int {
    lazy_static! {
        static ref SOCKET_FUNC: SocketFunc = get_libc_func!(SocketFunc, "socket");
    }

    //  injection strategy TBD

    SOCKET_FUNC(domain, type_, protocol)
}

// mmap() interception is disabled for now.  deadlocks on
//   malloc_init_hard()->mmap()->DynamicLibrary::open()->malloc_init_hard(), at least
//   on systems w/jemalloc.
#[no_mangle]
#[allow(private_no_mangle_fns)]
#[allow(dead_code)]
// pub
extern "C" fn mmap(addr: *mut c_void,
                   length_: size_t,
                   prot: c_int,
                   flags: c_int,
                   fd: c_int,
                   offset: off_t)
                   -> *mut c_void {
    lazy_static! {
        static ref MMAP_FUNC: MmapFunc = get_libc_func!(MmapFunc, "mmap");
    }

    injectFaults!(fd, "mmap", libc::MAP_FAILED);

    MMAP_FUNC(addr, length_, prot, flags, fd, offset)
}
