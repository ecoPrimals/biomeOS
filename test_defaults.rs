use biomeos_system::SystemConfig;

fn main() {
    let config = SystemConfig::default();
    println!("Boot timeout: {}", config.boot.timeout_seconds);
    println!("Boot sequence: {}", config.boot.sequence.len());
    println!("Services timeout: {}", config.services.startup_timeout_seconds);
    println!("Services count: {}", config.services.services.len());
    println!("Devices config_dir: {:?}", config.devices.config_dir);
    println!("Packages cache_dir: {:?}", config.packages.cache_dir);
}
