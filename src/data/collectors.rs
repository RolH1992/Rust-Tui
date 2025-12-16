use chrono;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, System};

pub struct SystemInfo {
    system: System,
    networks: Networks,
    disks: Disks,
}

impl SystemInfo {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_specifics(CpuRefreshKind::everything());
        system.refresh_memory_specifics(MemoryRefreshKind::everything());

        Self {
            system,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
        }
    }

    pub fn refresh(&mut self) {
        // Refresh CPU, memory, processes, networks, and disks
        self.system
            .refresh_cpu_specifics(CpuRefreshKind::everything());
        self.system
            .refresh_memory_specifics(MemoryRefreshKind::everything());
        self.system
            .refresh_processes(sysinfo::ProcessesToUpdate::All, true); // Corrected variant
        self.networks.refresh(false);
        self.disks.refresh(false);
    }

    // CPU Information
    pub fn cpu_usage(&self) -> f32 {
        self.system.global_cpu_usage()
    }

    pub fn cpu_count(&self) -> usize {
        self.system.cpus().len()
    }

    // Memory Information
    pub fn memory_usage(&self) -> (u64, u64) {
        (self.system.used_memory(), self.system.total_memory())
    }

    pub fn swap_usage(&self) -> (u64, u64) {
        (self.system.used_swap(), self.system.total_swap())
    }

    // Network Information
    pub fn network_stats(&self) -> Vec<(String, u64, u64)> {
        self.networks
            .iter()
            .map(|(interface, data)| (interface.clone(), data.received(), data.transmitted()))
            .collect()
    }

    // Disk Information
    pub fn disk_stats(&self) -> Vec<(String, u64, u64, String)> {
        self.disks
            .iter()
            .map(|disk| {
                (
                    disk.name().to_string_lossy().to_string(),
                    disk.total_space(),
                    disk.available_space(),
                    disk.mount_point().to_string_lossy().to_string(),
                )
            })
            .collect()
    }

    // Processes Information
    pub fn top_processes(&self, count: usize) -> Vec<(String, u32, f32, u64)> {
        let mut processes: Vec<_> = self
            .system
            .processes()
            .values()
            .map(|process| {
                (
                    process.name().to_string_lossy().to_string(),
                    process.pid().as_u32(),
                    process.cpu_usage(),
                    process.memory(),
                )
            })
            .collect();

        // Sort by CPU usage in descending order
        processes.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Return the top `count` processes
        processes.into_iter().take(count).collect()
    }

    // System Information
    pub fn uptime(&self) -> u64 {
        System::uptime()
    }

    pub fn load_average(&self) -> (f64, f64, f64) {
        let load_avg = System::load_average();
        (load_avg.one, load_avg.five, load_avg.fifteen)
    }
}

// Struct for JSON serialization
#[derive(serde::Serialize)]
pub struct SystemData {
    pub cpu: CpuData,
    pub memory: MemoryData,
    pub disks: Vec<DiskData>,
    pub network: Vec<NetworkData>,
    pub processes: Vec<ProcessData>,
    pub system: SystemInfoData,
    pub timestamp: i64,
}

#[derive(serde::Serialize)]
pub struct CpuData {
    pub usage: f32,
    pub count: usize,
    pub load_average: LoadAverage,
}

#[derive(serde::Serialize)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(serde::Serialize)]
pub struct MemoryData {
    pub used: u64,
    pub total: u64,
    pub used_swap: u64,
    pub total_swap: u64,
}

#[derive(serde::Serialize)]
pub struct DiskData {
    pub name: String,
    pub total: u64,
    pub available: u64,
    pub mount_point: String,
}

#[derive(serde::Serialize)]
pub struct NetworkData {
    pub interface: String,
    pub received: u64,
    pub transmitted: u64,
}

#[derive(serde::Serialize)]
pub struct ProcessData {
    pub name: String,
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory: u64,
}

#[derive(serde::Serialize)]
pub struct SystemInfoData {
    pub uptime: u64,
}

impl<'a> From<&'a SystemInfo> for SystemData {
    fn from(sys_info: &'a SystemInfo) -> Self {
        let (used_mem, total_mem) = sys_info.memory_usage();
        let (used_swap, total_swap) = sys_info.swap_usage();
        let load_avg = sys_info.load_average();

        SystemData {
            cpu: CpuData {
                usage: sys_info.cpu_usage(),
                count: sys_info.cpu_count(),
                load_average: LoadAverage {
                    one: load_avg.0,
                    five: load_avg.1,
                    fifteen: load_avg.2,
                },
            },
            memory: MemoryData {
                used: used_mem,
                total: total_mem,
                used_swap,
                total_swap,
            },
            disks: sys_info
                .disk_stats()
                .iter()
                .map(|(name, total, available, mount_point)| DiskData {
                    name: name.clone(),
                    total: *total,
                    available: *available,
                    mount_point: mount_point.clone(),
                })
                .collect(),
            network: sys_info
                .network_stats()
                .iter()
                .map(|(interface, received, transmitted)| NetworkData {
                    interface: interface.clone(),
                    received: *received,
                    transmitted: *transmitted,
                })
                .collect(),
            processes: sys_info
                .top_processes(5) // Call the top_processes method here
                .iter()
                .map(|(name, pid, cpu, memory)| ProcessData {
                    name: name.clone(),
                    pid: *pid,
                    cpu_usage: *cpu,
                    memory: *memory,
                })
                .collect(),
            system: SystemInfoData {
                uptime: sys_info.uptime(),
            },
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}
