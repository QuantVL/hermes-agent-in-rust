//! Private control-socket transport for native session tickets.
//!
//! On unix the socket lives at `<control_home>/control.sock` and the peer must
//! be the same OS user. On Windows this returns a clear unsupported error: the
//! CI gate is ubuntu and the native control channel is unix-only.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{GatewayError, Result};

pub const CONTROL_SOCKET_NAME: &str = "control.sock";
pub const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);

/// Resolve the control socket path for a profile's control home.
///
/// Kept as a small abstraction so tests can inject a temporary directory.
pub fn control_socket_path(control_home: &Path) -> PathBuf {
    control_home.join(CONTROL_SOCKET_NAME)
}

/// Send one newline-terminated request and read one newline-terminated reply.
#[cfg(unix)]
pub fn query(control_home: &Path, request_line: &str) -> Result<String> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;

    let path = control_socket_path(control_home);
    let stream = UnixStream::connect(&path).map_err(|e| {
        GatewayError::Discovery(format!(
            "cannot connect to control socket {} (is this the same OS user?): {e}",
            path.display()
        ))
    })?;
    ensure_same_user(&stream)?;
    stream.set_read_timeout(Some(CONTROL_TIMEOUT))?;
    stream.set_write_timeout(Some(CONTROL_TIMEOUT))?;

    let mut writer = stream.try_clone()?;
    writer.write_all(request_line.as_bytes())?;
    writer.flush()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Err(GatewayError::InvalidState(
            "control socket closed without sending a response line".into(),
        ));
    }
    Ok(line.trim_end().to_string())
}

#[cfg(target_os = "linux")]
fn ensure_same_user(stream: &std::os::unix::net::UnixStream) -> Result<()> {
    use std::os::unix::io::AsRawFd;

    let mut cred: libc::ucred = unsafe { std::mem::zeroed() };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let rc = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut cred as *mut _ as *mut libc::c_void,
            &mut len,
        )
    };
    if rc != 0 {
        return Err(GatewayError::InvalidState(
            "SO_PEERCRED failed on the control socket".into(),
        ));
    }
    let euid = unsafe { libc::geteuid() };
    if cred.uid != euid {
        return Err(GatewayError::InvalidState(format!(
            "control socket peer uid {} does not match effective uid {euid}",
            cred.uid
        )));
    }
    Ok(())
}

#[cfg(all(unix, not(target_os = "linux")))]
fn ensure_same_user(_stream: &std::os::unix::net::UnixStream) -> Result<()> {
    // Local-socket permissions enforce same-user access on these platforms.
    Ok(())
}

#[cfg(not(unix))]
pub fn query(_control_home: &Path, _request_line: &str) -> Result<String> {
    Err(GatewayError::Unsupported(
        "native control-socket tickets are unix-only and unsupported in this build/CI".into(),
    ))
}
