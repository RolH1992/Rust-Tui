# Rust TUI

A system monitor written in Rust, inspired by the original Conky.

![Rust](https://img.shields.io/badge/rust-stable-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- 🖥️ **CPU Monitoring**: Usage percentage, core count, load average
- 💾 **Memory Monitoring**: RAM and swap usage
- 💽 **Disk Usage**: All mounted filesystems
- 🌐 **Network Statistics**: Traffic for all interfaces
- 🔥 **Process Monitoring**: Top processes by CPU usage
- ⏱️ **System Uptime**: How long the system has been running
- ⚡ **Real-time Updates**: Configurable refresh intervals

## Installation

### Prerequisites
- Rust and Cargo installed

### From Source
- git clone https://github.com/RolH1992/Rust-conky.git
- cd Rust-conky
- cargo build --release
- chmod +x conky.sh
- ./conky.sh

## ✨ Latest Improvements

### Flicker-Free GUI
The shell script GUI now uses advanced terminal control for smooth, flicker-free updates:
- Minimal screen redraw with `tput` commands
- Hidden cursor during display
- Proper cleanup on exit
- Color-coded sections with progress bars

### Performance
- Efficient data collection in Rust
- JSON parsing optimized
- 1-second real-time updates
