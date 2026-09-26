// This file is part of the uutils coreutils package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(unix)]
pub use self::unix::{
    Pid,
    ProcessChecker,
    //stdin_is_bad_fd, stdin_is_pipe_or_fifo, supports_pid_checks, Pid, ProcessChecker,
    supports_pid_checks,
};

#[cfg(windows)]
pub use self::windows::{Pid, ProcessChecker, supports_pid_checks};

// WASI has no process management; provide stubs so tail compiles.
#[cfg(all(target_os = "wasi", not(target_vendor = "wasmer")))]
pub type Pid = u64;

#[cfg(all(target_os = "wasi", not(target_vendor = "wasmer")))]
pub fn supports_pid_checks(_pid: Pid) -> bool {
    false
}

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[cfg(target_vendor = "wasmer")]
mod wasix;
#[cfg(target_vendor = "wasmer")]
pub use wasix::{Pid, ProcessChecker, supports_pid_checks};
