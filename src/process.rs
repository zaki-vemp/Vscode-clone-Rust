//! Captured helper processes have explicit output budgets and deadlines.
use std::io;
use std::process::{Command, Output};
use std::time::Duration;

#[derive(Clone, Copy)]
pub struct Limits {
    pub timeout: Duration,
    pub stdout: usize,
    pub stderr: usize,
}

enum Chunk {
    Data(bool, Vec<u8>),
    End,
    Error(io::Error),
}

struct ChildGuard {
    child: std::process::Child,
    armed: bool,
}
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if self.armed {
            #[cfg(unix)]
            // SAFETY: capture creates this child's process group and does not
            // reap it before the pipe readers finish. Its PGID cannot be reused
            // while the unreaped child still owns that identity.
            unsafe {
                libc::kill(-(self.child.id() as libc::pid_t), libc::SIGKILL);
            }
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

pub fn capture(command: &mut Command, limits: Limits) -> io::Result<Output> {
    use std::io::Read;
    use std::process::Stdio;
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Instant;
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut guard = ChildGuard {
        child: command.spawn()?,
        armed: true,
    };
    let (tx, rx) = mpsc::sync_channel(8);
    fn read_chunks(
        mut pipe: impl Read + Send + 'static,
        stdout: bool,
        tx: mpsc::SyncSender<Chunk>,
    ) {
        std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match pipe.read(&mut buffer) {
                    Ok(0) => {
                        let _ = tx.send(Chunk::End);
                        return;
                    }
                    Ok(length) => {
                        if tx
                            .send(Chunk::Data(stdout, buffer[..length].to_vec()))
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => {
                        let _ = tx.send(Chunk::Error(error));
                        return;
                    }
                }
            }
        });
    }
    read_chunks(
        guard.child.stdout.take().expect("piped stdout"),
        true,
        tx.clone(),
    );
    read_chunks(guard.child.stderr.take().expect("piped stderr"), false, tx);
    let start = Instant::now();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut finished = 0;
    loop {
        if start.elapsed() >= limits.timeout {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "helper process exceeded its deadline",
            ));
        }
        if finished == 2 {
            if let Some(status) = guard.child.try_wait()? {
                guard.armed = false;
                return Ok(Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }
        match rx.recv_timeout(
            limits
                .timeout
                .saturating_sub(start.elapsed())
                .min(Duration::from_millis(10)),
        ) {
            Ok(Chunk::Data(is_stdout, bytes)) => {
                let (output, limit) = if is_stdout {
                    (&mut stdout, limits.stdout)
                } else {
                    (&mut stderr, limits.stderr)
                };
                if output.len().saturating_add(bytes.len()) > limit {
                    return Err(io::Error::other(
                        "helper process exceeded its output budget",
                    ));
                }
                output.extend_from_slice(&bytes);
            }
            Ok(Chunk::End) => finished += 1,
            Ok(Chunk::Error(error)) => return Err(error),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                return Err(io::Error::other(
                    "helper output readers stopped unexpectedly",
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(mode: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "process::tests::capture_fixture",
                "--ignored",
                "--nocapture",
            ])
            .env("OXIDE_TEST_CHILD", mode);
        command
    }
    #[test]
    #[ignore = "subprocess fixture, invoked only by process tests"]
    fn capture_fixture() {
        match std::env::var("OXIDE_TEST_CHILD")
            .unwrap_or_default()
            .as_str()
        {
            "sleep" => std::thread::sleep(Duration::from_millis(300)),
            "output" => {
                use std::io::Write;
                std::io::stdout()
                    .write_all(&vec![b'x'; 128 * 1024])
                    .unwrap();
            }
            _ => {}
        }
    }
    #[test]
    fn ops_001_hanging_capture_has_deadline() {
        let start = std::time::Instant::now();
        let result = capture(
            &mut fixture("sleep"),
            Limits {
                timeout: Duration::from_millis(50),
                stdout: 16 * 1024,
                stderr: 16 * 1024,
            },
        );
        assert!(
            result.is_err(),
            "Capture must stop a helper that exceeds its deadline"
        );
        assert!(start.elapsed() < Duration::from_millis(250));
    }
    #[test]
    fn ops_001_capture_rejects_excess_output() {
        let result = capture(
            &mut fixture("output"),
            Limits {
                timeout: Duration::from_secs(2),
                stdout: 1024,
                stderr: 1024,
            },
        );
        assert!(
            result.is_err(),
            "Captured stdout must have an enforced byte budget"
        );
    }
}
