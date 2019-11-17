use libc::ioctl;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use nix::fcntl::OFlag;
use nix::{fcntl};
use nix::errno::Errno;
use nix::sys::stat::Mode;
use nix::unistd::close;

const VT_ACTIVATE: u64 = 0x5606;
const VT_WAITACTIVE: u64 = 0x5607;

const KDGKBTYPE: u64 = 0x4B33;


const KB_101: u8 = 0x02;
const KB_84: u8 = 0x01;

#[derive(Debug)]
pub enum ErrorKind {
    ActivateError(i32),
    WaitActiveError(i32),
    CloseError,
    OpenConsoleError,
    ENOTCONN,
    GetFDError,
    PermissionDeniedError
}
impl Error for ErrorKind {}
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <dyn Debug>::fmt(self, f)
    }
}

#[derive(Debug)]
pub enum TryOpenError {
    EACCESS,
    OTHER
}
impl Error for TryOpenError {}
impl fmt::Display for TryOpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <dyn Debug>::fmt(self, f)
    }
}

fn is_a_console(fd: i32) -> bool {
    unsafe {
        let mut arg = 0;
        return ioctl(fd, KDGKBTYPE, &mut arg) == 0 && ((arg == KB_101) || (arg == KB_84));
    }
}

fn try_open(filename: &str, oflag: OFlag) -> Result<i32, TryOpenError> {
    match fcntl::open(filename, oflag, Mode::empty()) {
        Ok(fd) => return Ok(fd),
        Err(err) => {
            match err.as_errno() {
                Some(errno) => {
                    if errno == Errno::EACCES {
                        return Err(TryOpenError::EACCESS)
                    }
                }
                None => ()
            }
        }
    }
    Err(TryOpenError::OTHER)
}

fn open_a_console(filename: &str) -> Result<i32, ErrorKind> {

    //TODO: Can this be done cleaner?
    let fd = match try_open(filename, OFlag::O_RDWR) {
        Err(TryOpenError::EACCESS) => {
            match try_open(filename, OFlag::O_WRONLY) {
                Err( TryOpenError::EACCESS) => {
                    match try_open(filename, OFlag::O_RDONLY) {
                        Err(_) => return Err(ErrorKind::OpenConsoleError),
                        Ok(fd) => fd
                    }
                },
                Err(_) => return Err(ErrorKind::OpenConsoleError),
                Ok(fd) => fd
            }
        },
        Err(_) => return Err(ErrorKind::OpenConsoleError),
        Ok(fd) => fd
    };

    if !is_a_console(fd) {
        close(fd).map_err(|_| ErrorKind::CloseError)?;
        return Err(ErrorKind::ENOTCONN);
    }

    Ok(fd)
}

fn get_fd() -> Result<i32, ErrorKind> {

    match open_a_console("/dev/tty") {
        Ok(fd) => return Ok(fd),
        Err(_) => ()
    }

    match open_a_console("/dev/tty") {
        Ok(fd) => return Ok(fd),
        Err(_) => ()
    }

    match open_a_console("/dev/tty0") {
        Ok(fd) => return Ok(fd),
        Err(_) => ()
    }

    match open_a_console("/dev/vc/0") {
        Ok(fd) => return Ok(fd),
        Err(_) => ()
    }

    match open_a_console("/dev/console") {
        Ok(fd) => return Ok(fd),
        Err(_) => ()
    }

    for fd in 0..3 {
        if is_a_console(fd) {
            return Ok(fd);
        }
    }

    // If all attempts fail Error
    Err(ErrorKind::GetFDError)
}

pub fn chvt(ttynum: i32) -> Result<(), ErrorKind> {

    let fd = get_fd()?;

    unsafe {
        let activate = ioctl(fd, VT_ACTIVATE, ttynum);

        if activate > 0 {
            return Err(ErrorKind::ActivateError(activate));
        }
        let wait = ioctl(fd, VT_WAITACTIVE, ttynum);
        if wait > 0 {
            return Err(ErrorKind::WaitActiveError(wait));
        }
    }

    close(fd).map_err(|_| ErrorKind::CloseError)?;

    Ok(())
}