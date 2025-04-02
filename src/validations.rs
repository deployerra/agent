use std::fs;

pub fn check_distro() -> Result<String, String> {
    let supported_distros = ["ubuntu", "debian", "fedora", "rhel", "amzn", "ol", "arch"];

    let distro_id = fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|contents| {
            contents
                .lines()
                .find(|line| line.starts_with("ID="))
                .map(|line| line.trim_start_matches("ID=").trim().replace('"', ""))
        });

    match distro_id {
        Some(distro) if supported_distros.contains(&distro.as_str()) => Ok(distro),
        Some(distro) => Err(format!("unsupported distro: {}. exiting", distro)),
        None => Err("Could not determine the distro. Exiting...".to_string()),
    }
}

use std::process::{Command, Stdio};

pub fn has_sudo_access() -> bool {
    let output = Command::new("sudo")
        .arg("-n")
        .arg("true")
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("sudo:") && stderr.contains("password") {
                false
            } else {
                output.status.success()
            }
        }
        Err(_) => false,
    }
}
