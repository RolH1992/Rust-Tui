use crossterm::{
    cursor,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::{self, Write};

pub fn draw(system_info: &crate::data::SystemInfo) -> anyhow::Result<()> {
    let mut stdout = io::stdout();

    // Move cursor to top-left and clear from cursor to end of screen
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.execute(Clear(ClearType::FromCursorDown))?;

    // Header - Cyan
    stdout.execute(SetForegroundColor(Color::Cyan))?;
    println!("=== Rust Conky System Monitor ===");
    stdout.execute(ResetColor)?;
    println!();

    // CPU Information - Green
    stdout.execute(SetForegroundColor(Color::Green))?;
    let cpu_usage = system_info.cpu_usage();
    let cpu_count = system_info.cpu_count();
    let load_avg = system_info.load_average();

    // Color CPU usage based on load
    let cpu_color = match cpu_usage {
        x if x > 80.0 => Color::Red,
        x if x > 60.0 => Color::Yellow,
        _ => Color::Green,
    };

    stdout.execute(SetForegroundColor(cpu_color))?;
    print!("CPU: {:.1}% ", cpu_usage);
    stdout.execute(SetForegroundColor(Color::Green))?;
    println!("({} cores)", cpu_count);

    println!(
        "Load Average: {:.2}, {:.2}, {:.2}",
        load_avg.0, load_avg.1, load_avg.2
    );
    println!();
    stdout.execute(ResetColor)?;

    // Memory Information - Blue
    stdout.execute(SetForegroundColor(Color::Blue))?;
    let (used_mem, total_mem) = system_info.memory_usage();
    let (used_swap, total_swap) = system_info.swap_usage();
    let used_mem_gb = used_mem as f64 / 1024.0 / 1024.0 / 1024.0;
    let total_mem_gb = total_mem as f64 / 1024.0 / 1024.0 / 1024.0;
    let mem_percentage = (used_mem as f64 / total_mem as f64) * 100.0;

    // Color memory usage based on percentage
    let mem_color = match mem_percentage {
        x if x > 90.0 => Color::Red,
        x if x > 75.0 => Color::Yellow,
        _ => Color::Blue,
    };

    stdout.execute(SetForegroundColor(mem_color))?;
    print!(
        "Memory: {:.2}GB / {:.2}GB ({:.1}%)",
        used_mem_gb, total_mem_gb, mem_percentage
    );
    stdout.execute(SetForegroundColor(Color::Blue))?;

    let used_swap_gb = used_swap as f64 / 1024.0 / 1024.0 / 1024.0;
    let total_swap_gb = total_swap as f64 / 1024.0 / 1024.0 / 1024.0;
    let swap_percentage = if total_swap > 0 {
        (used_swap as f64 / total_swap as f64) * 100.0
    } else {
        0.0
    };

    let swap_color = match swap_percentage {
        x if x > 50.0 => Color::Red,
        x if x > 25.0 => Color::Yellow,
        _ => Color::Blue,
    };

    stdout.execute(SetForegroundColor(swap_color))?;
    println!();
    print!(
        "Swap:   {:.2}GB / {:.2}GB ({:.1}%)",
        used_swap_gb, total_swap_gb, swap_percentage
    );
    stdout.execute(ResetColor)?;
    println!();
    println!();

    // Disk Information - Magenta
    let disk_stats = system_info.disk_stats();
    if !disk_stats.is_empty() {
        stdout.execute(SetForegroundColor(Color::Magenta))?;
        println!("Disks:");
        stdout.execute(ResetColor)?;

        for (name, total, available, mount_point) in disk_stats {
            let used = total - available;
            let used_gb = used as f64 / 1024.0 / 1024.0 / 1024.0;
            let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;
            let percentage = (used as f64 / total as f64) * 100.0;

            let disk_color = match percentage {
                x if x > 90.0 => Color::Red,
                x if x > 80.0 => Color::Yellow,
                _ => Color::DarkGrey,
            };

            stdout.execute(SetForegroundColor(disk_color))?;
            println!(
                "  {} ({}) {:.1}GB / {:.1}GB ({:.1}%)",
                name, mount_point, used_gb, total_gb, percentage
            );
        }
        stdout.execute(ResetColor)?;
        println!();
    }

    // Network Information - Yellow
    let network_stats = system_info.network_stats();
    if !network_stats.is_empty() {
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        println!("Network Interfaces:");
        stdout.execute(ResetColor)?;

        for (interface, received, transmitted) in network_stats {
            let received_mb = received as f64 / 1024.0 / 1024.0;
            let transmitted_mb = transmitted as f64 / 1024.0 / 1024.0;

            println!(
                "  {}: ↓ {:.2}MB ↑ {:.2}MB",
                interface, received_mb, transmitted_mb
            );
        }
        println!();
    }

    // Top Processes - Cyan
    let top_processes = system_info.top_processes(5);
    if !top_processes.is_empty() {
        stdout.execute(SetForegroundColor(Color::Cyan))?;
        println!("Top Processes (by CPU):");
        stdout.execute(ResetColor)?;

        for (name, pid, cpu, memory) in top_processes {
            let memory_mb = memory as f64 / 1024.0 / 1024.0;

            // Color process based on CPU usage
            let process_color = match cpu {
                x if x > 50.0 => Color::Red,
                x if x > 20.0 => Color::Yellow,
                _ => Color::White,
            };

            stdout.execute(SetForegroundColor(process_color))?;
            println!("  {:6} {:.1}% {:.1}MB {}", pid, cpu, memory_mb, name);
        }
        stdout.execute(ResetColor)?;
        println!();
    }

    // System Uptime - Green
    stdout.execute(SetForegroundColor(Color::Green))?;
    let uptime = system_info.uptime();
    let hours = uptime / 3600;
    let minutes = (uptime % 3600) / 60;
    println!("Uptime: {} hours, {} minutes", hours, minutes);
    stdout.execute(ResetColor)?;

    stdout.flush()?;
    Ok(())
}

pub fn clear_screen() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.flush()?;
    Ok(())
}
