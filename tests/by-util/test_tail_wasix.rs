// This file is part of the uutils coreutils package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Runs inside the same WASIX filesystem as tail. Renaming host files outside
// the VM does not update Wasmer's guest inode table.
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

fn wait_for(child: &mut Child, text: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if fs::read_to_string("output").unwrap().contains(text) {
            // Let the poller finish recording metadata before the next mutation.
            std::thread::sleep(Duration::from_millis(150));
            return;
        }
        assert!(child.try_wait().unwrap().is_none(), "tail exited early");
        assert!(
            Instant::now() < deadline,
            "waiting for {text:?}; output: {:?}; errors: {:?}",
            fs::read_to_string("output"),
            fs::read_to_string("errors")
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn main() {
    fs::write("follow.log", "initial\n").unwrap();
    let mut child = Command::new("tail")
        .args(["-F", "-n", "1", "-s", "0.05", "follow.log"])
        .stdout(File::create("output").unwrap())
        .stderr(File::create("errors").unwrap())
        .spawn()
        .unwrap();
    wait_for(&mut child, "initial\n");
    OpenOptions::new()
        .append(true)
        .open("follow.log")
        .unwrap()
        .write_all(b"appended\n")
        .unwrap();
    wait_for(&mut child, "appended\n");
    fs::write("follow.log", "short\n").unwrap();
    wait_for(&mut child, "short\n");
    fs::rename("follow.log", "rotated.log").unwrap();
    fs::write("follow.log", "replacement\n").unwrap();
    wait_for(&mut child, "replacement\n");
    assert_eq!(
        fs::read_to_string("output").unwrap(),
        "initial\nappended\nshort\nreplacement\n"
    );
    child.kill().unwrap();
    child.wait().unwrap();
    println!("PASS tail -F append, truncate and file replacement");
}
