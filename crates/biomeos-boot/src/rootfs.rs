use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

/// Configuration for building a BiomeOS root filesystem
#[derive(Debug, Clone)]
pub struct RootFsConfig {
    /// Size of the root filesystem (e.g., "8G")
    pub size: String,
    
    /// Output path for the root filesystem image
    pub output: PathBuf,
    
    /// Directory containing primal binaries
    pub primals_dir: Option<PathBuf>,
    
    /// Directory containing systemd service files
    pub services_dir: Option<PathBuf>,
    
    /// Mount point for building (temporary, auto-generated if None)
    pub mount_point: Option<PathBuf>,
    
    /// Filesystem type (default: ext4)
    pub fs_type: String,
    
    /// DNS servers (discovered from system if None)
    pub dns_servers: Option<Vec<String>>,
    
    /// NBD device to use (auto-detect if None)
    pub nbd_device: Option<String>,
    
    /// Hostname for the system (default: "biomeos")
    pub hostname: String,
}

impl Default for RootFsConfig {
    fn default() -> Self {
        Self {
            size: "8G".to_string(),
            output: PathBuf::from("biomeos-root.qcow2"),
            primals_dir: None,
            services_dir: None,
            mount_point: None,
            fs_type: "ext4".to_string(),
            dns_servers: None,  // Discover from system
            nbd_device: None,    // Auto-detect
            hostname: "biomeos".to_string(),
        }
    }
}

/// Builder for BiomeOS root filesystems
pub struct RootFsBuilder {
    config: RootFsConfig,
}

impl RootFsBuilder {
    /// Create a new root filesystem builder
    pub fn new(config: RootFsConfig) -> Self {
        Self { config }
    }
    
    /// Build the complete root filesystem
    pub async fn build(&self) -> Result<PathBuf> {
        info!("🏗️  Building BiomeOS root filesystem");
        info!("  Output: {}", self.config.output.display());
        info!("  Size: {}", self.config.size);
        
        // Step 1: Create qcow2 image
        self.create_image()?;
        
        // Step 2: Format with filesystem
        self.format_filesystem()?;
        
        // Step 3: Mount the image
        let mount_point = self.mount_image()?;
        
        // Step 4: Install base system
        self.install_base_system(&mount_point)?;
        
        // Step 5: Install BiomeOS components
        self.install_biomeos(&mount_point).await?;
        
        // Step 6: Install primals (if provided)
        if let Some(primals_dir) = &self.config.primals_dir {
            self.install_primals(&mount_point, primals_dir).await?;
        }
        
        // Step 6.5: Install systemd services (if provided)
        if let Some(services_dir) = &self.config.services_dir {
            self.install_services(&mount_point, services_dir)?;
        }
        
        // Step 7: Configure system
        self.configure_system(&mount_point)?;
        
        // Step 8: Unmount
        self.unmount_image(&mount_point)?;
        
        info!("✅ Root filesystem built: {}", self.config.output.display());
        Ok(self.config.output.clone())
    }
    
    /// Create a qcow2 disk image
    fn create_image(&self) -> Result<()> {
        info!("📦 Creating qcow2 image...");
        
        let status = Command::new("qemu-img")
            .args(["create", "-f", "qcow2"])
            .arg(&self.config.output)
            .arg(&self.config.size)
            .status()
            .context("Failed to execute qemu-img")?;
        
        if !status.success() {
            anyhow::bail!("qemu-img failed to create image");
        }
        
        info!("  ✓ Created {} image", self.config.size);
        Ok(())
    }
    
    /// Format the image with a filesystem
    fn format_filesystem(&self) -> Result<()> {
        info!("💾 Formatting filesystem...");
        
        // Use qemu-nbd to format the image
        let nbd_device = self.attach_nbd()?;
        
        let status = Command::new(&format!("mkfs.{}", self.config.fs_type))
            .arg(&nbd_device)
            .arg("-L").arg("BIOMEOS")  // Label
            .status()
            .context("Failed to format filesystem")?;
        
        if !status.success() {
            self.detach_nbd(&nbd_device)?;
            anyhow::bail!("Failed to format filesystem");
        }
        
        self.detach_nbd(&nbd_device)?;
        
        info!("  ✓ Formatted as {}", self.config.fs_type);
        Ok(())
    }
    
    /// Attach image via NBD and return device path
    fn attach_nbd(&self) -> Result<String> {
        // Determine NBD device to use
        let nbd_device = if let Some(ref device) = self.config.nbd_device {
            device.clone()
        } else {
            // Auto-detect available NBD device
            self.find_available_nbd()?
        };
        
        info!("Using NBD device: {}", nbd_device);
        
        // Load nbd module
        let _ = Command::new("modprobe").arg("nbd").arg("max_part=8").status();
        
        // Clean up any existing NBD connections first
        let _ = Command::new("qemu-nbd").args(["-d", &nbd_device]).status();
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // Connect to NBD
        let status = Command::new("qemu-nbd")
            .args(["-c", &nbd_device])
            .arg(&self.config.output)
            .status()
            .context("Failed to attach NBD device")?;
        
        if !status.success() {
            anyhow::bail!("Failed to attach NBD device: {}", nbd_device);
        }
        
        // Wait for device to be ready
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        Ok(nbd_device)
    }
    
    /// Find an available NBD device
    fn find_available_nbd(&self) -> Result<String> {
        // Try nbd0 through nbd15
        for i in 0..16 {
            let device = format!("/dev/nbd{}", i);
            let device_path = Path::new(&device);
            
            if !device_path.exists() {
                continue;
            }
            
            // Check if device is in use
            let output = Command::new("lsblk")
                .arg(&device)
                .output();
                
            if let Ok(out) = output {
                let out_str = String::from_utf8_lossy(&out.stdout);
                // If lsblk shows no partitions, device is available
                if out_str.lines().count() <= 1 {
                    return Ok(device);
                }
            }
        }
        
        // Default to nbd0 if none found
        Ok("/dev/nbd0".to_string())
    }
    
    /// Detach NBD device
    fn detach_nbd(&self, device: &str) -> Result<()> {
        let status = Command::new("qemu-nbd")
            .args(["-d", device])
            .status()
            .context("Failed to detach NBD device")?;
        
        if !status.success() {
            warn!("Failed to detach NBD device cleanly");
        }
        
        Ok(())
    }
    
    /// Mount the filesystem image
    fn mount_image(&self) -> Result<PathBuf> {
        info!("📂 Mounting filesystem...");
        
        let nbd_device = self.attach_nbd()?;
        
        // Use configured mount point or create temporary
        let mount_point = if let Some(ref mp) = self.config.mount_point {
            mp.clone()
        } else {
            // Use tempfile for safer temporary directory
            let temp_dir = tempfile::Builder::new()
                .prefix("biomeos-build-")
                .tempdir()
                .context("Failed to create temporary mount point")?;
            temp_dir.into_path()
        };
        
        std::fs::create_dir_all(&mount_point)?;
        
        let status = Command::new("mount")
            .arg(&nbd_device)
            .arg(&mount_point)
            .status()
            .context("Failed to mount filesystem")?;
        
        if !status.success() {
            self.detach_nbd(&nbd_device)?;
            anyhow::bail!("Failed to mount filesystem at {}", mount_point.display());
        }
        
        info!("  ✓ Mounted at {}", mount_point.display());
        Ok(mount_point)
    }
    
    /// Unmount the filesystem
    fn unmount_image(&self, mount_point: &Path) -> Result<()> {
        info!("📤 Unmounting filesystem...");
        
        let status = Command::new("umount")
            .arg(mount_point)
            .status()
            .context("Failed to unmount filesystem")?;
        
        if !status.success() {
            warn!("Failed to unmount cleanly");
        }
        
        // Detach NBD
        self.detach_nbd("/dev/nbd0")?;
        
        info!("  ✓ Unmounted");
        Ok(())
    }
    
    /// Install base system (minimal Linux userspace)
    fn install_base_system(&self, root: &Path) -> Result<()> {
        info!("🌱 Installing base system...");
        
        // Create directory structure
        let dirs = [
            "bin", "sbin", "usr/bin", "usr/sbin", "usr/local/bin",
            "etc", "etc/systemd/system",
            "var", "var/log", "var/lib",
            "tmp", "run", "proc", "sys", "dev",
            "home", "root",
            "lib", "lib64", "usr/lib",
        ];
        
        for dir in &dirs {
            std::fs::create_dir_all(root.join(dir))
                .with_context(|| format!("Failed to create {}", dir))?;
        }
        
        // Install BusyBox for basic utilities
        self.install_busybox(root)?;
        
        info!("  ✓ Base system installed");
        Ok(())
    }
    
    /// Install BusyBox
    fn install_busybox(&self, root: &Path) -> Result<()> {
        let busybox_path = which::which("busybox")
            .context("BusyBox not found - please install busybox")?;
        
        let dest = root.join("bin/busybox");
        std::fs::copy(&busybox_path, &dest)
            .context("Failed to copy busybox")?;
        
        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&dest, perms)?;
        }
        
        // Create symlinks for common utilities
        let busybox_links = [
            "sh", "ls", "cat", "echo", "mkdir", "rm", "cp", "mv",
            "grep", "ps", "top", "mount", "umount", "ip", "ping",
        ];
        
        for link in &busybox_links {
            let link_path = root.join("bin").join(link);
            #[cfg(unix)]
            std::os::unix::fs::symlink("busybox", &link_path)
                .ok(); // Ignore if already exists
        }
        
        info!("  ✓ BusyBox installed");
        Ok(())
    }
    
    /// Install BiomeOS components (init, etc.)
    async fn install_biomeos(&self, root: &Path) -> Result<()> {
        info!("🦀 Installing BiomeOS components...");
        
        // Install biomeos-init
        let init_src = PathBuf::from("target/release/biomeos-init");
        if !init_src.exists() {
            warn!("  ⚠️  biomeos-init not found, skipping");
            return Ok(());
        }
        
        let init_dest = root.join("sbin/init");
        std::fs::copy(&init_src, &init_dest)
            .context("Failed to copy biomeos-init")?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&init_dest, perms)?;
        }
        
        // Copy required libraries
        self.copy_required_libraries(root, &init_src)?;
        
        info!("  ✓ BiomeOS init installed");
        Ok(())
    }
    
    /// Install primal binaries
    async fn install_primals(&self, root: &Path, primals_dir: &Path) -> Result<()> {
        info!("🔧 Installing primal binaries...");
        
        if !primals_dir.exists() {
            warn!("  ⚠️  Primals directory not found: {}", primals_dir.display());
            return Ok(());
        }
        
        let primal_dest_dir = root.join("usr/local/bin");
        std::fs::create_dir_all(&primal_dest_dir)?;
        
        // Scan for primal binaries
        let mut count = 0;
        for entry in std::fs::read_dir(primals_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                // Check if executable
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = std::fs::metadata(&path)?;
                    if metadata.permissions().mode() & 0o111 == 0 {
                        continue; // Not executable
                    }
                }
                
                let filename = path.file_name().unwrap();
                let dest = primal_dest_dir.join(filename);
                
                std::fs::copy(&path, &dest)
                    .with_context(|| format!("Failed to copy {}", filename.to_string_lossy()))?;
                
                // Copy required libraries
                self.copy_required_libraries(root, &path)?;
                
                info!("  ✓ Installed {}", filename.to_string_lossy());
                count += 1;
            }
        }
        
        info!("  ✓ Installed {} primal(s)", count);
        Ok(())
    }
    
    /// Copy required dynamic libraries for a binary
    fn copy_required_libraries(&self, root: &Path, binary: &Path) -> Result<()> {
        let output = Command::new("ldd")
            .arg(binary)
            .output()
            .context("Failed to run ldd")?;
        
        if !output.status.success() {
            // Binary might be statically linked
            return Ok(());
        }
        
        let ldd_output = String::from_utf8_lossy(&output.stdout);
        
        for line in ldd_output.lines() {
            if let Some(lib_path_str) = line.split("=>").nth(1) {
                if let Some(path_str) = lib_path_str.trim().split_whitespace().next() {
                    let lib_path = PathBuf::from(path_str);
                    if lib_path.exists() && lib_path.is_absolute() {
                        // Preserve full directory structure
                        if let Ok(stripped) = lib_path.strip_prefix("/") {
                            let dest = root.join(stripped);
                            
                            if let Some(parent) = dest.parent() {
                                std::fs::create_dir_all(parent)?;
                            }
                            
                            if !dest.exists() {
                                std::fs::copy(&lib_path, &dest).ok();
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Configure the system (network, services, etc.)
    fn configure_system(&self, root: &Path) -> Result<()> {
        info!("⚙️  Configuring system...");
        
        // Create /etc/hostname (configurable)
        std::fs::write(
            root.join("etc/hostname"),
            format!("{}\n", self.config.hostname)
        )?;
        
        // Create /etc/hosts (use configured hostname)
        std::fs::write(
            root.join("etc/hosts"),
            format!(
                "127.0.0.1\tlocalhost\n\
                 ::1\t\tlocalhost\n\
                 127.0.1.1\t{}\n",
                self.config.hostname
            )
        )?;
        
        // Create /etc/resolv.conf (use configured DNS or discover)
        let dns_config = self.get_dns_config()?;
        std::fs::write(
            root.join("etc/resolv.conf"),
            dns_config
        )?;
        
        // Create /etc/fstab
        std::fs::write(
            root.join("etc/fstab"),
            "# BiomeOS fstab\n\
             proc\t/proc\tproc\tdefaults\t0\t0\n\
             sysfs\t/sys\tsysfs\tdefaults\t0\t0\n\
             devpts\t/dev/pts\tdevpts\tdefaults\t0\t0\n\
             tmpfs\t/tmp\ttmpfs\tdefaults\t0\t0\n\
             tmpfs\t/run\ttmpfs\tdefaults\t0\t0\n"
        )?;
        
        info!("  ✓ System configured (hostname: {})", self.config.hostname);
        Ok(())
    }
    
    /// Get DNS configuration (from config or discover from system)
    fn get_dns_config(&self) -> Result<String> {
        if let Some(ref servers) = self.config.dns_servers {
            // Use configured DNS servers
            Ok(servers
                .iter()
                .map(|s| format!("nameserver {}", s))
                .collect::<Vec<_>>()
                .join("\n") + "\n")
        } else {
            // Discover from system /etc/resolv.conf
            match std::fs::read_to_string("/etc/resolv.conf") {
                Ok(content) if !content.trim().is_empty() => {
                    info!("  Using system DNS configuration");
                    Ok(content)
                }
                _ => {
                    // Fallback to common public DNS
                    info!("  Using fallback DNS (Cloudflare, Google)");
                    Ok("nameserver 1.1.1.1\nnameserver 8.8.8.8\n".to_string())
                }
            }
        }
    }
}

/// Command-line interface for root filesystem builder
#[derive(Debug, clap::Parser)]
#[command(name = "biomeos-rootfs")]
#[command(about = "Build BiomeOS root filesystem images")]
pub struct RootFsCli {
    /// Size of the root filesystem (e.g., "8G", "10G")
    #[arg(short, long, default_value = "8G")]
    size: String,
    
    /// Output path for the image
    #[arg(short, long, default_value = "biomeos-root.qcow2")]
    output: PathBuf,
    
    /// Directory containing primal binaries
    #[arg(short, long)]
    primals: Option<PathBuf>,
    
    /// Filesystem type
    #[arg(short, long, default_value = "ext4")]
    fs_type: String,
}

impl RootFsCli {
    pub async fn execute(self) -> Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter("info")
            .init();
        
        let config = RootFsConfig {
            size: self.size,
            output: self.output,
            primals_dir: self.primals,
            fs_type: self.fs_type,
            ..Default::default()
        };
        
        let builder = RootFsBuilder::new(config);
        builder.build().await?;
        
        Ok(())
    }
}

