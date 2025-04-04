use std::process::{Command, Stdio};

pub fn setup(distro: String) {
    if !check_command("docker") {
        println!("docker not found");
        println!("proceeding to install docker and docker compose");
        install_docker(&distro);
    } else {
        println!("docker found");
        ensure_docker_setup()
    }

    println!("checking for docker compose");

    if !check_docker_compose() {
        println!("docker compose not found");
        println!("proceeding to install docker compose");
        install_docker_compose();
    } else {
        println!("docker compose found");
    }
    println!("setup complete");
}

fn check_command(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", cmd))
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn check_docker_compose() -> bool {
    Command::new("sh")
        .arg("-c")
        .arg("docker help | grep 'compose'")
        .stderr(Stdio::null())
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn install_docker(distro: &str) {
    println!("Updating package repositories...");
    let update_command = match distro {
        "arch" => "sudo pacman -Syy --noconfirm",
        "ubuntu" | "debian" => "sudo apt-get update",
        "fedora" | "rhel" | "centos" => "sudo dnf check-update || true",
        "amzn" | "ol" => "sudo yum update -y",
        _ => {
            eprintln!("Unsupported distribution");
            return;
        }
    };
    // Update repositories
    let update_result = Command::new("sh").arg("-c").arg(update_command).status();
    match update_result {
        Ok(status)
            if status.success()
                || (distro == "fedora" || distro == "rhel" || distro == "centos") =>
        {
            // dnf check-update returns exit code 100 when there are updates available
            println!("Package repositories updated successfully");
        }
        _ => {
            eprintln!("Failed to update package repositories");
            return;
        }
    }

    println!("Installing Docker...");

    // For Amazon Linux, we need to detect the version first
    let install_command = if distro == "amzn" {
        // Check Amazon Linux version
        let version_result = Command::new("sh")
            .arg("-c")
            .arg("cat /etc/system-release")
            .output();

        match version_result {
            Ok(output) if output.status.success() => {
                let release_info = String::from_utf8_lossy(&output.stdout);
                if release_info.contains("Amazon Linux release 2023") {
                    "sudo yum install -y docker"
                } else if release_info.contains("Amazon Linux release 2") {
                    "sudo amazon-linux-extras install -y docker"
                } else {
                    eprintln!("Unsupported Amazon Linux version");
                    return;
                }
            }
            _ => {
                eprintln!("Failed to determine Amazon Linux version");
                return;
            }
        }
    } else {
        match distro {
            "arch" => "sudo pacman -S --noconfirm docker",
            "ubuntu" | "debian" => "sudo apt-get install -y docker.io",
            "fedora" => "sudo dnf install -y docker",
            "rhel" | "ol" => {
                "sudo dnf -y install dnf-plugins-core && sudo dnf config-manager --add-repo https://download.docker.com/linux/rhel/docker-ce.repo && sudo dnf install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin"
            }
            _ => {
                eprintln!("Unsupported distribution");
                return;
            }
        }
    };

    // Install Docker
    let install_result = Command::new("sh").arg("-c").arg(install_command).status();
    match install_result {
        Ok(status) if status.success() => println!("Docker installed successfully"),
        _ => {
            eprintln!("Docker installation failed");
            return;
        }
    }
    // Start Docker service
    println!("Starting Docker service...");
    let start_cmd = "sudo systemctl enable --now docker";
    let start_result = Command::new("sh").arg("-c").arg(start_cmd).output();
    match start_result {
        Ok(output) if output.status.success() => println!("Docker service started successfully"),
        _ => {
            eprintln!("Failed to start Docker service");
            return;
        }
    }
    // Add current user to the docker group
    println!("Adding current user to docker group...");
    let current_user = match Command::new("whoami").output() {
        Ok(output) if output.status.success() => match String::from_utf8(output.stdout) {
            Ok(user) => user.trim().to_string(),
            Err(_) => {
                eprintln!("Failed to get current username");
                return;
            }
        },
        _ => {
            eprintln!("Failed to get current username");
            return;
        }
    };
    let group_cmd = format!("sudo usermod -aG docker {}", current_user);
    let group_result = Command::new("sh").arg("-c").arg(group_cmd).output();
    match group_result {
        Ok(output) if output.status.success() => {
            println!("User '{}' added to docker group", current_user);
            println!("NOTE: You may need to log out and back in for group changes to take effect");
        }
        _ => eprintln!("Failed to add user to docker group"),
    }

    // Restart docker service
    let restart_cmd = "sudo systemctl restart docker";
    let restart_result = Command::new("sh").arg("-c").arg(restart_cmd).output();
    match restart_result {
        Ok(output) if output.status.success() => {
            println!("Docker service restarted");
        }
        _ => eprintln!("Failed to restart docker service"),
    }
}

fn install_docker_compose() {
    let arch = match Command::new("uname").arg("-m").output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => {
            eprintln!("failed to detect system architecture");
            return;
        }
    };

    let url = format!(
        "https://github.com/docker/compose/releases/latest/download/docker-compose-linux-{}",
        arch
    );

    println!("starting download");

    let output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "sudo mkdir -p /usr/local/lib/docker/cli-plugins && \
             sudo curl -L '{}' -o /usr/local/lib/docker/cli-plugins/docker-compose && \
             sudo chmod +x /usr/local/lib/docker/cli-plugins/docker-compose",
            url
        ))
        .output()
        .expect("failed to start installation process");

    if output.status.success() {
        println!("download complete");

        if check_docker_compose() {
            println!("docker compose installed successfully");
        } else {
            eprintln!("docker compose installation failed");
        }
    } else {
        eprintln!("installation command failed");
    }
}

fn ensure_docker_setup() {
    // Check if Docker service is running
    println!("Checking Docker service status...");
    let service_check = Command::new("sh")
        .arg("-c")
        .arg("systemctl is-active docker")
        .output();

    let service_running = match service_check {
        Ok(output) => {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            status == "active"
        }
        Err(_) => false,
    };

    if !service_running {
        println!("Docker service is not running. Starting it now...");
        let _ = Command::new("sh")
            .arg("-c")
            .arg("sudo systemctl enable --now docker")
            .output();
    }

    // Get current user
    let current_user = match Command::new("whoami").output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => {
            eprintln!("Failed to get current username");
            return;
        }
    };

    // Check if user is in docker group
    let group_check = Command::new("sh")
        .arg("-c")
        .arg(format!("groups {} | grep -q docker", current_user))
        .status();

    let in_docker_group = match group_check {
        Ok(status) => status.success(),
        Err(_) => false,
    };

    if !in_docker_group {
        println!("Adding user '{}' to docker group...", current_user);
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("sudo usermod -aG docker {}", current_user))
            .output();

        println!("NOTE: You may need to log out and back in for group changes to take effect");
    }
}
