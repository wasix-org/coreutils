// This file is part of the uutils coreutils package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub type Pid = libc::pid_t;

pub struct ProcessChecker(Pid);

impl ProcessChecker {
    pub fn new(pid: Pid) -> Self {
        Self(pid)
    }

    pub fn is_dead(&self) -> bool {
        // SAFETY: signal zero checks existence without delivering a signal.
        (unsafe { libc::kill(self.0, 0) }) == -1
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
    }
}

pub fn supports_pid_checks(pid: Pid) -> bool {
    // SAFETY: signal zero checks existence without delivering a signal.
    pid > 0
        && (unsafe { libc::kill(pid, 0) } == 0
            || std::io::Error::last_os_error().raw_os_error() != Some(libc::ENOSYS))
}
