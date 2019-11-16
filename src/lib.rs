use libc::ioctl;
use libc::open;
use libc::close;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::ffi::CString;

const VT_ACTIVATE: u64 = 0x5606;
const VT_WAITACTIVE: u64 = 0x5607;

const KDGKBTYPE: u64 = 0x4B33;


const KB_101: u8 = 0x02;
const KB_84: u8 = 0x01;

#[derive(Debug)]
pub enum ErrorKind {
    ActivateError(i32),
    WaitActiveError(i32),
    CantOpenConsoleError,
}
impl Error for ErrorKind {}
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <dyn Debug>::fmt(self, f)
    }
}

unsafe fn is_a_console(fd: i32) -> bool {
    let mut arg = 0;

    return ioctl(fd, KDGKBTYPE, &arg) == 0 && ((arg == KB_101) || (arg == KB_84));
}

unsafe fn open_a_console(filename: &str) -> i32 {
    let c_str = CString::new(filename).unwrap();
    let fnam: *const i8 = c_str.as_ptr() as *const i8;

    let mut fd = open(fnam, libc::O_RDWR);

    let mut errno = *libc::__errno_location();

    if fd < 0 && errno == libc::EACCES {
        fd = open(fnam, libc::O_WRONLY);
    }
    if fd < 0 && errno == libc::EACCES {
        fd = open(fnam, libc::O_RDONLY);
    }
    if (fd < 0){
        return -1;
    }

    if (!is_a_console(fd)) {
        close(fd);
        return -1;
    }
    return fd;
}

unsafe fn get_fd() -> i32{

    let mut fd = open_a_console("/dev/tty");
    if (fd >= 0) {
        return fd;
    }
    fd = open_a_console("/dev/tty0");
    if (fd >= 0){
        return fd;
    }
    fd = open_a_console("/dev/vc/0");
    if (fd >= 0){
        return fd;
    }
    fd = open_a_console("/dev/console");
    if (fd >= 0){
        return fd;
    }

    for fd in 0..3{
        if is_a_console(fd){
            return fd;
        }
    }

    return -1;
}

pub fn chvt(ttynum: i32) -> Result<(), ErrorKind> {
    unsafe {

        let fd = get_fd();

        if fd < 0 {
            return Err(ErrorKind::CantOpenConsoleError);
        }

        let activate = ioctl(fd, VT_ACTIVATE, ttynum);
        if activate > 0 {
            return Err(ErrorKind::ActivateError(activate));
        }
        let wait = ioctl(fd, VT_WAITACTIVE, ttynum);
        if wait > 0 {
            return Err(ErrorKind::WaitActiveError(wait));
        }
    }

    Ok(())
}
