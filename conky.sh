#!/bin/bash

# Rust Conky - Shell Script GUI with smooth refresh
# Rust collects data, shell displays it

# Colors for beautiful output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Progress bar width
BAR_WIDTH=20

# Save cursor position and hide cursor
tput sc
tput civis

# Trap to show cursor on exit
cleanup() {
    tput cnorm
    tput rc
    clear
    exit 0
}
trap cleanup EXIT INT TERM

# Function to move cursor to top and clear from cursor down
refresh_screen() {
    tput cup 0 0
}

# Function to show header (only once at start)
show_header() {
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║           RUST CONKY - SYSTEM MONITOR (SHELL GUI)        ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo
}

# Function to format bytes to human readable
format_bytes() {
    local bytes=$1
    if [ $bytes -lt 1024 ]; then
        echo "${bytes}B"
    elif [ $bytes -lt 1048576 ]; then
        echo "$(($bytes / 1024))K"
    elif [ $bytes -lt 1073741824 ]; then
        printf "%.1fM" $(echo "scale=1; $bytes / 1048576" | bc)
    else
        printf "%.1fG" $(echo "scale=1; $bytes / 1073741824" | bc)
    fi
}

# Function to draw progress bar - FIXED for floating point
draw_bar() {
    local percent=$1
    local color=$2
    local label=$3
    
    # Use bc for floating point calculation
    local filled=$(echo "scale=0; ($percent * $BAR_WIDTH) / 100" | bc)
    local empty=$((BAR_WIDTH - filled))
    
    printf "  ${color}${label}: ["
    for ((i=0; i<filled; i++)); do printf "█"; done
    for ((i=0; i<empty; i++)); do printf "░"; done
    printf "] %5.1f%%${NC}" "$percent"
}

# Function to format uptime
format_uptime() {
    local seconds=$1
    local hours=$(($seconds / 3600))
    local minutes=$((($seconds % 3600) / 60))
    printf "%dh %dm" "$hours" "$minutes"
}

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq is required but not installed.${NC}"
    echo "Install with: sudo apt install jq (Ubuntu/Debian) or brew install jq (macOS)"
    tput cnorm
    exit 1
fi

# Check if bc is installed (needed for calculations)
if ! command -v bc &> /dev/null; then
    echo -e "${RED}Error: bc is required but not installed.${NC}"
    echo "Install with: sudo apt install bc (Ubuntu/Debian) or brew install bc (macOS)"
    tput cnorm
    exit 1
fi

# Check if Rust binary exists, build if not
if [ ! -f "./target/release/rust-conky" ]; then
    echo -e "${YELLOW}Building Rust backend...${NC}"
    cargo build --release
    if [ $? -ne 0 ]; then
        echo -e "${RED}Failed to build Rust backend${NC}"
        tput cnorm
        exit 1
    fi
    echo -e "${GREEN}Rust backend built successfully!${NC}"
    sleep 2
fi

# Clear screen once at start
clear
show_header

# Save starting line position
HEADER_LINES=5
tput cup $HEADER_LINES 0

# Main display loop
while true; do
    # Get JSON data from Rust backend - filter non-JSON lines
    JSON_DATA=$(timeout 2 ./target/release/rust-conky --json 2>&1 | grep '^{' | head -1)
    
    if [ -z "$JSON_DATA" ] || ! echo "$JSON_DATA" | jq -e . >/dev/null 2>&1; then
        # Move to error position
        tput cup $((HEADER_LINES + 1)) 0
        echo -e "${RED}Error: Could not get data${NC}"
        echo -e "${YELLOW}Retrying...${NC}"
        sleep 2
        # Clear error lines
        tput cup $((HEADER_LINES + 1)) 0
        tput el
        tput cup $((HEADER_LINES + 2)) 0
        tput el
        continue
    fi
    
    # Move to content start position
    tput cup $HEADER_LINES 0
    
    # Parse JSON using jq
    CPU_USAGE=$(echo "$JSON_DATA" | jq -r '.cpu.usage')
    CPU_COUNT=$(echo "$JSON_DATA" | jq -r '.cpu.count')
    LOAD_ONE=$(echo "$JSON_DATA" | jq -r '.cpu.load_average.one')
    LOAD_FIVE=$(echo "$JSON_DATA" | jq -r '.cpu.load_average.five')
    LOAD_FIFTEEN=$(echo "$JSON_DATA" | jq -r '.cpu.load_average.fifteen')
    
    MEM_USED=$(echo "$JSON_DATA" | jq -r '.memory.used')
    MEM_TOTAL=$(echo "$JSON_DATA" | jq -r '.memory.total')
    MEM_PERCENT=$(echo "scale=1; $MEM_USED * 100 / $MEM_TOTAL" | bc)
    
    SWAP_USED=$(echo "$JSON_DATA" | jq -r '.memory.used_swap')
    SWAP_TOTAL=$(echo "$JSON_DATA" | jq -r '.memory.total_swap')
    SWAP_PERCENT=0
    if [ "$SWAP_TOTAL" -gt 0 ]; then
        SWAP_PERCENT=$(echo "scale=1; $SWAP_USED * 100 / $SWAP_TOTAL" | bc)
    fi
    
    UPTIME=$(echo "$JSON_DATA" | jq -r '.system.uptime')
    
    # Display CPU section
    echo -e "${GREEN}┌──────────────── CPU ────────────────┐${NC}"
    printf "  Usage:   ${GREEN}%5.1f%%${NC} (%d cores)\n" "$CPU_USAGE" "$CPU_COUNT"
    printf "  Load:    %.2f, %.2f, %.2f\n" "$LOAD_ONE" "$LOAD_FIVE" "$LOAD_FIFTEEN"
    draw_bar "$CPU_USAGE" "$GREEN" "CPU"
    echo -e "\n"
    
    # Display Memory section
    echo -e "${CYAN}┌─────────────── MEMORY ───────────────┐${NC}"
    echo -e "  RAM:     $(format_bytes $MEM_USED)/$(format_bytes $MEM_TOTAL)"
    draw_bar "$MEM_PERCENT" "$CYAN" "RAM"
    echo
    
    if [ "$SWAP_TOTAL" -gt 0 ]; then
        echo -e "  SWAP:    $(format_bytes $SWAP_USED)/$(format_bytes $SWAP_TOTAL)"
        draw_bar "$SWAP_PERCENT" "$BLUE" "SWP"
        echo
    else
        echo
    fi
    
    # Display Disk section (first 2 disks)
    echo -e "${YELLOW}┌──────────────── DISKS ────────────────┐${NC}"
    DISK_COUNT=$(echo "$JSON_DATA" | jq '.disks | length')
    if [ "$DISK_COUNT" -gt 0 ]; then
        for ((i=0; i<DISK_COUNT && i<2; i++)); do
            DISK_NAME=$(echo "$JSON_DATA" | jq -r ".disks[$i].name")
            DISK_TOTAL=$(echo "$JSON_DATA" | jq -r ".disks[$i].total")
            DISK_AVAIL=$(echo "$JSON_DATA" | jq -r ".disks[$i].available")
            DISK_MOUNT=$(echo "$JSON_DATA" | jq -r ".disks[$i].mount_point")
            DISK_USED=$((DISK_TOTAL - DISK_AVAIL))
            DISK_PERCENT=$(echo "scale=1; $DISK_USED * 100 / $DISK_TOTAL" | bc)
            
            # Shorten mount point for display
            MOUNT_NAME=$(basename "$DISK_MOUNT")
            if [ "$MOUNT_NAME" = "/" ]; then
                MOUNT_NAME="root"
            fi
            
            echo -e "  ${MOUNT_NAME}: $(format_bytes $DISK_USED)/$(format_bytes $DISK_TOTAL)"
            draw_bar "$DISK_PERCENT" "$YELLOW" "USE"
            echo
        done
    else
        echo -e "  No disks found\n"
    fi
    
    # Display Network section
    echo -e "${MAGENTA}┌─────────────── NETWORK ───────────────┐${NC}"
    NET_COUNT=$(echo "$JSON_DATA" | jq '.network | length')
    if [ "$NET_COUNT" -gt 0 ]; then
        for ((i=0; i<NET_COUNT && i<2; i++)); do
            IFACE=$(echo "$JSON_DATA" | jq -r ".network[$i].interface")
            RX=$(echo "$JSON_DATA" | jq -r ".network[$i].received")
            TX=$(echo "$JSON_DATA" | jq -r ".network[$i].transmitted")
            
            echo -e "  ${IFACE}: ↓$(format_bytes $RX) ↑$(format_bytes $TX)"
        done
    else
        echo -e "  No network interfaces found"
    fi
    echo -e "\n"
    
    # Display Top Processes
    echo -e "${RED}┌───────────── TOP PROCESSES ─────────────┐${NC}"
    PROC_COUNT=$(echo "$JSON_DATA" | jq '.processes | length')
    if [ "$PROC_COUNT" -gt 0 ]; then
        for ((i=0; i<PROC_COUNT && i<3; i++)); do
            PROC_NAME=$(echo "$JSON_DATA" | jq -r ".processes[$i].name")
            PROC_PID=$(echo "$JSON_DATA" | jq -r ".processes[$i].pid")
            PROC_CPU=$(echo "$JSON_DATA" | jq -r ".processes[$i].cpu_usage")
            PROC_MEM=$(echo "$JSON_DATA" | jq -r ".processes[$i].memory")
            
            # Shorten long process names
            if [ ${#PROC_NAME} -gt 20 ]; then
                PROC_NAME="${PROC_NAME:0:17}..."
            fi
            
            printf "  %5s ${RED}%4.0f%%${NC} %6s ${WHITE}%s${NC}\n" \
                   "$PROC_PID" "$PROC_CPU" "$(format_bytes $PROC_MEM)" "$PROC_NAME"
        done
    else
        echo -e "  No processes data"
    fi
    echo -e "\n"
    
    # Display System Info
    echo -e "${WHITE}┌─────────────── SYSTEM ────────────────┐${NC}"
    echo -e "  Uptime:  $(format_uptime $UPTIME)"
    echo -e "  Update:  Every 1 second"
    echo -e "  Status:  ${GREEN}Running${NC}"
    echo -e "${YELLOW}  Press Ctrl+C to exit${NC}"
    
    # Clear any remaining lines from previous update
    tput ed
    
    # Wait 1 second before refreshing
    sleep 1
done

# Show cursor when done
tput cnorm
