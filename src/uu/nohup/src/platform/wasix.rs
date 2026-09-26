// This file is part of the uutils coreutils package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::ffi::CString;
use std::fs::{File, OpenOptions};
use std::io::{Error, IsTerminal};
use std::os::fd::{AsRawFd, FromRawFd};
use std::process::Command;
use uucore::error::{UResult, set_exit_code};

use crate::find_stdout;

unsafe extern "C" {
    fn __SIG_IGN(signal: libc::c_int);
}

pub(crate) fn prepare() -> UResult<()> {
    if std::io::stdin().is_terminal() {
        let file = OpenOptions::new().write(true).open("/dev/null")?;
        redirect(file.as_raw_fd(), libc::STDIN_FILENO)?;
    }
    if std::io::stdout().is_terminal() {
        let file = find_stdout()?;
        redirect(file.as_raw_fd(), libc::STDOUT_FILENO)?;
    }
    if std::io::stderr().is_terminal() {
        redirect(libc::STDOUT_FILENO, libc::STDERR_FILENO)?;
    }
    // SAFETY: an empty signal mask and zero flags are valid for sigaction.
    let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
    // WASIX libc uses a real function for SIG_IGN. The Rust libc constant (1)
    // is a Unix sentinel, not the WASIX handler or its inheritable disposition.
    action.sa_sigaction = __SIG_IGN as *const () as libc::sighandler_t;
    // SAFETY: action is initialized and references libc's ignore handler.
    if unsafe { libc::sigaction(libc::SIGHUP, &action, std::ptr::null_mut()) } == -1 {
        return Err(Error::last_os_error().into());
    }
    Ok(())
}

fn redirect(from: i32, to: i32) -> std::io::Result<()> {
    // SAFETY: the caller retains ownership of both descriptors across dup2.
    if unsafe { libc::dup2(from, to) } == -1 {
        return Err(Error::last_os_error());
    }
    Ok(())
}

pub(crate) fn run(command: &mut Command) -> UResult<Option<Error>> {
    // WASIX exec does not transfer guest signal dispositions, while its
    // posix_spawn implementation (used by Command) preserves ignored signals.
    match command.status() {
        Ok(status) => {
            set_exit_code(status.code().unwrap_or(1));
            Ok(None)
        }
        Err(error) => Ok(Some(error)),
    }
}

pub(crate) fn open_output_file(path: &str) -> std::io::Result<File> {
    let path = CString::new(path)?;
    // SAFETY: path is terminated, flags require a mode argument, and ownership
    // of a successful descriptor is transferred once to File below.
    let fd = unsafe {
        libc::open(
            path.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND,
            0o600 as libc::mode_t,
        )
    };
    if fd == -1 {
        return Err(Error::last_os_error());
    }
    // SAFETY: fd is newly opened and owned by this function.
    Ok(unsafe { File::from_raw_fd(fd) })
}
