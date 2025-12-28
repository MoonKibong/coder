//! System Resource Monitor
//!
//! Provides real-time system metrics for the admin dashboard.

use serde::Serialize;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};

/// System resource metrics
#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disks: Vec<DiskMetrics>,
    pub network: NetworkMetrics,
    pub system_info: SystemInfo,
}

#[derive(Debug, Serialize)]
pub struct CpuMetrics {
    /// Overall CPU usage percentage (0-100)
    pub usage_percent: f32,
    /// Number of physical cores
    pub physical_cores: usize,
    /// Number of logical cores (threads)
    pub logical_cores: usize,
    /// CPU brand/name
    pub brand: String,
}

#[derive(Debug, Serialize)]
pub struct MemoryMetrics {
    /// Total memory in bytes
    pub total_bytes: u64,
    /// Used memory in bytes
    pub used_bytes: u64,
    /// Available memory in bytes
    pub available_bytes: u64,
    /// Memory usage percentage (0-100)
    pub usage_percent: f32,
    /// Total swap in bytes
    pub swap_total_bytes: u64,
    /// Used swap in bytes
    pub swap_used_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct DiskMetrics {
    /// Disk name/mount point
    pub name: String,
    /// Total space in bytes
    pub total_bytes: u64,
    /// Available space in bytes
    pub available_bytes: u64,
    /// Used space in bytes
    pub used_bytes: u64,
    /// Usage percentage (0-100)
    pub usage_percent: f32,
    /// Filesystem type
    pub fs_type: String,
}

#[derive(Debug, Serialize)]
pub struct NetworkMetrics {
    /// Total bytes received
    pub received_bytes: u64,
    /// Total bytes transmitted
    pub transmitted_bytes: u64,
    /// Active interfaces
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    /// OS name
    pub os_name: String,
    /// OS version
    pub os_version: String,
    /// Kernel version
    pub kernel_version: String,
    /// Hostname
    pub hostname: String,
    /// System uptime in seconds
    pub uptime_seconds: u64,
}

pub struct SystemMonitor;

impl SystemMonitor {
    /// Get current system metrics
    pub fn get_metrics() -> SystemMetrics {
        // Create system with specific refresh settings
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        // Refresh CPU usage (needs a small delay for accurate readings)
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_usage();

        SystemMetrics {
            cpu: Self::get_cpu_metrics(&sys),
            memory: Self::get_memory_metrics(&sys),
            disks: Self::get_disk_metrics(),
            network: Self::get_network_metrics(),
            system_info: Self::get_system_info(),
        }
    }

    /// Get lightweight metrics (without disk and network which can be slow)
    pub fn get_quick_metrics() -> SystemMetrics {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_usage();

        SystemMetrics {
            cpu: Self::get_cpu_metrics(&sys),
            memory: Self::get_memory_metrics(&sys),
            disks: vec![], // Skip for quick metrics
            network: NetworkMetrics {
                received_bytes: 0,
                transmitted_bytes: 0,
                interfaces: vec![],
            },
            system_info: Self::get_system_info(),
        }
    }

    fn get_cpu_metrics(sys: &System) -> CpuMetrics {
        let cpus = sys.cpus();
        let usage: f32 = if cpus.is_empty() {
            0.0
        } else {
            cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32
        };

        let brand = cpus.first().map(|c| c.brand().to_string()).unwrap_or_default();

        CpuMetrics {
            usage_percent: usage,
            physical_cores: sys.physical_core_count().unwrap_or(0),
            logical_cores: cpus.len(),
            brand,
        }
    }

    fn get_memory_metrics(sys: &System) -> MemoryMetrics {
        let total = sys.total_memory();
        let used = sys.used_memory();
        let available = sys.available_memory();
        let usage_percent = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        MemoryMetrics {
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            usage_percent,
            swap_total_bytes: sys.total_swap(),
            swap_used_bytes: sys.used_swap(),
        }
    }

    fn get_disk_metrics() -> Vec<DiskMetrics> {
        let disks = Disks::new_with_refreshed_list();
        disks
            .iter()
            .filter(|disk| {
                // Filter out virtual/special filesystems
                let fs = disk.file_system().to_string_lossy();
                !fs.starts_with("devfs")
                    && !fs.starts_with("tmpfs")
                    && !fs.starts_with("overlay")
                    && disk.total_space() > 0
            })
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total.saturating_sub(available);
                let usage_percent = if total > 0 {
                    (used as f32 / total as f32) * 100.0
                } else {
                    0.0
                };

                DiskMetrics {
                    name: disk.mount_point().to_string_lossy().to_string(),
                    total_bytes: total,
                    available_bytes: available,
                    used_bytes: used,
                    usage_percent,
                    fs_type: disk.file_system().to_string_lossy().to_string(),
                }
            })
            .collect()
    }

    fn get_network_metrics() -> NetworkMetrics {
        let networks = Networks::new_with_refreshed_list();

        let mut total_received: u64 = 0;
        let mut total_transmitted: u64 = 0;
        let mut interfaces = Vec::new();

        for (name, data) in &networks {
            let received = data.total_received();
            let transmitted = data.total_transmitted();

            // Skip interfaces with no traffic
            if received > 0 || transmitted > 0 {
                total_received += received;
                total_transmitted += transmitted;

                interfaces.push(NetworkInterface {
                    name: name.clone(),
                    received_bytes: received,
                    transmitted_bytes: transmitted,
                });
            }
        }

        NetworkMetrics {
            received_bytes: total_received,
            transmitted_bytes: total_transmitted,
            interfaces,
        }
    }

    fn get_system_info() -> SystemInfo {
        SystemInfo {
            os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            uptime_seconds: System::uptime(),
        }
    }
}

/// Format bytes to human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format uptime to human-readable string
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
