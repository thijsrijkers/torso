use std::process::Command;
use std::io;

pub fn ssh_run(ssh_user: &str, remote_host: &str, command: &str) -> io::Result<()> {
    let status = Command::new("ssh")
        .args([
            "-o", "BatchMode=yes",
            "-o", "ConnectTimeout=10",
            &format!("{ssh_user}@{remote_host}"),
            command,
        ])
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("ssh command failed: {command}"),
        ));
    }
    Ok(())
}

pub fn scp_copy(
    local_file: &str,
    ssh_user: &str,
    remote_host: &str,
    destination_path: &str
) -> io::Result<()> {
    let destination = format!("{ssh_user}@{remote_host}:{destination_path}");

    let status = Command::new("scp")
        .args([local_file, &destination])
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("scp failed for {remote_host}"),
        ));
    }

    Ok(())
}
