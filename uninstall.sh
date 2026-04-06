#!/usr/bin/env bash
#
# Cuttlefish Uninstall Script
# Removes all files and configurations created by install.sh
#
set -uo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Default paths (same as install.sh)
INSTALL_DIR="${INSTALL_DIR:-/opt/cuttlefish}"
CONFIG_DIR="${CONFIG_DIR:-/etc/cuttlefish}"
DATA_DIR="${DATA_DIR:-/var/lib/cuttlefish}"
LOG_DIR="${LOG_DIR:-/var/log/cuttlefish}"
CACHE_DIR="${CACHE_DIR:-/var/cache/cuttlefish}"
SERVICE_USER="cuttlefish"

# Track what was removed
REMOVED_ITEMS=()
KEPT_ITEMS=()

#######################################
# Utility Functions
#######################################

print_banner() {
    echo -e "${RED}"
    echo '   ____      _   _   _       __ _     _     '
    echo '  / ___|   _| |_| |_| | ___ / _(_)___| |__  '
    echo ' | |  | | | | __| __| |/ _ \ |_| / __| `_ \ '
    echo ' | |__| |_| | |_| |_| |  __/  _| \__ \ | | |'
    echo '  \____\__,_|\__|\__|_|\___|_| |_|___/_| |_|'
    echo -e "${NC}"
    echo -e "${BOLD}Uninstaller${NC}"
    echo ""
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

prompt_yn() {
    local message="$1"
    local default="${2:-n}"

    if [[ "$default" == "y" ]]; then
        echo -en "${CYAN}$message${NC} [Y/n]: "
    else
        echo -en "${CYAN}$message${NC} [y/N]: "
    fi

    read -r yn
    yn="${yn:-$default}"
    [[ "$yn" =~ ^[Yy] ]]
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root (or with sudo)"
        exit 1
    fi
}

#######################################
# Stop Services
#######################################

stop_services() {
    echo ""
    echo -e "${BOLD}=== Stopping Services ===${NC}"
    echo ""

    # Stop systemd service if it exists
    if systemctl is-active --quiet cuttlefish 2>/dev/null; then
        info "Stopping Cuttlefish systemd service..."
        systemctl stop cuttlefish
        success "Service stopped"
        REMOVED_ITEMS+=("systemd service (stopped)")
    fi

    # Disable systemd service
    if systemctl is-enabled --quiet cuttlefish 2>/dev/null; then
        info "Disabling Cuttlefish systemd service..."
        systemctl disable cuttlefish
        success "Service disabled"
    fi

    # Kill any running cuttlefish processes
    local pids_to_kill=()
    while IFS= read -r pid; do
        [[ -z "$pid" ]] && continue
        local exe_path
        exe_path=$(readlink -f "/proc/$pid/exe" 2>/dev/null || true)
        if [[ "$exe_path" == *"/cuttlefish-rs" || "$exe_path" == *"/cuttlefish-tui" ]]; then
            pids_to_kill+=("$pid")
        fi
    done < <(pgrep -f "cuttlefish" 2>/dev/null || true)

    if [[ ${#pids_to_kill[@]} -gt 0 ]]; then
        info "Killing running Cuttlefish processes (PIDs: ${pids_to_kill[*]})..."
        for pid in "${pids_to_kill[@]}"; do
            kill "$pid" 2>/dev/null || true
        done
        sleep 1
        success "Processes terminated"
        REMOVED_ITEMS+=("running processes")
    fi
}

#######################################
# Remove Docker Resources
#######################################

remove_docker_resources() {
    echo ""
    echo -e "${BOLD}=== Docker Resources ===${NC}"
    echo ""

    if ! command -v docker &>/dev/null; then
        info "Docker not installed, skipping Docker cleanup"
        return
    fi

    # Stop and remove cuttlefish container if running
    if docker ps -a --format '{{.Names}}' | grep -q '^cuttlefish$'; then
        info "Removing Cuttlefish Docker container..."
        docker stop cuttlefish 2>/dev/null || true
        docker rm cuttlefish 2>/dev/null || true
        success "Container removed"
        REMOVED_ITEMS+=("docker container 'cuttlefish'")
    fi

    # Find and optionally remove sandbox containers created by Cuttlefish
    local sandbox_containers
    sandbox_containers=$(docker ps -a --filter "label=cuttlefish.sandbox=true" --format '{{.Names}}' 2>/dev/null || true)

    if [[ -n "$sandbox_containers" ]]; then
        echo ""
        warn "Found Cuttlefish sandbox containers:"
        echo "$sandbox_containers" | while read -r name; do
            echo "  - $name"
        done
        echo ""

        if prompt_yn "Remove all sandbox containers?" "y"; then
            docker ps -a --filter "label=cuttlefish.sandbox=true" -q | xargs -r docker rm -f 2>/dev/null || true
            success "Sandbox containers removed"
            REMOVED_ITEMS+=("sandbox containers")
        else
            KEPT_ITEMS+=("sandbox containers")
        fi
    fi

    # Optionally remove cuttlefish Docker image
    if docker images --format '{{.Repository}}' | grep -q '^cuttlefish$'; then
        if prompt_yn "Remove Cuttlefish Docker image?" "n"; then
            docker rmi cuttlefish 2>/dev/null || true
            success "Docker image removed"
            REMOVED_ITEMS+=("docker image 'cuttlefish'")
        else
            KEPT_ITEMS+=("docker image")
        fi
    fi
}

#######################################
# Remove Files
#######################################

remove_systemd_service() {
    echo ""
    echo -e "${BOLD}=== Systemd Service ===${NC}"
    echo ""

    if [[ -f /etc/systemd/system/cuttlefish.service ]]; then
        info "Removing systemd service file..."
        rm -f /etc/systemd/system/cuttlefish.service
        systemctl daemon-reload
        success "Removed /etc/systemd/system/cuttlefish.service"
        REMOVED_ITEMS+=("systemd service file")
    else
        info "No systemd service file found"
    fi
}

remove_symlinks() {
    echo ""
    echo -e "${BOLD}=== Command Symlinks ===${NC}"
    echo ""

    local links=("/usr/local/bin/cuttlefish" "/usr/local/bin/cuttlefish-rs" "/usr/local/bin/cuttlefish-tui")

    for link in "${links[@]}"; do
        if [[ -L "$link" ]]; then
            info "Removing symlink: $link"
            rm -f "$link"
            success "Removed $link"
            REMOVED_ITEMS+=("symlink $link")
        elif [[ -f "$link" ]]; then
            warn "$link exists but is not a symlink"
            if prompt_yn "Remove it anyway?" "n"; then
                rm -f "$link"
                success "Removed $link"
                REMOVED_ITEMS+=("file $link")
            else
                KEPT_ITEMS+=("$link")
            fi
        fi
    done
}

remove_install_dir() {
    echo ""
    echo -e "${BOLD}=== Installation Directory ===${NC}"
    echo ""

    if [[ -d "$INSTALL_DIR" ]]; then
        echo "Contents of $INSTALL_DIR:"
        ls -la "$INSTALL_DIR" 2>/dev/null | head -20
        echo ""

        if prompt_yn "Remove installation directory ($INSTALL_DIR)?" "y"; then
            rm -rf "$INSTALL_DIR"
            success "Removed $INSTALL_DIR"
            REMOVED_ITEMS+=("$INSTALL_DIR")
        else
            KEPT_ITEMS+=("$INSTALL_DIR")
        fi
    else
        info "Installation directory not found: $INSTALL_DIR"
    fi
}

remove_config_dir() {
    echo ""
    echo -e "${BOLD}=== Configuration Directory ===${NC}"
    echo ""

    if [[ -d "$CONFIG_DIR" ]]; then
        echo "Contents of $CONFIG_DIR:"
        ls -la "$CONFIG_DIR" 2>/dev/null
        echo ""

        warn "This contains your configuration and API keys!"
        if prompt_yn "Remove configuration directory ($CONFIG_DIR)?" "n"; then
            rm -rf "$CONFIG_DIR"
            success "Removed $CONFIG_DIR"
            REMOVED_ITEMS+=("$CONFIG_DIR")
        else
            KEPT_ITEMS+=("$CONFIG_DIR (contains config/API keys)")
        fi
    else
        info "Configuration directory not found: $CONFIG_DIR"
    fi
}

remove_data_dir() {
    echo ""
    echo -e "${BOLD}=== Data Directory ===${NC}"
    echo ""

    if [[ -d "$DATA_DIR" ]]; then
        local db_size
        db_size=$(du -sh "$DATA_DIR" 2>/dev/null | cut -f1)
        echo "Data directory: $DATA_DIR ($db_size)"

        if [[ -f "$DATA_DIR/cuttlefish.db" ]]; then
            local db_file_size
            db_file_size=$(du -sh "$DATA_DIR/cuttlefish.db" 2>/dev/null | cut -f1)
            echo "  Database: cuttlefish.db ($db_file_size)"
        fi
        echo ""

        warn "This contains your database with projects, conversations, and history!"
        if prompt_yn "Remove data directory ($DATA_DIR)?" "n"; then
            rm -rf "$DATA_DIR"
            success "Removed $DATA_DIR"
            REMOVED_ITEMS+=("$DATA_DIR")
        else
            KEPT_ITEMS+=("$DATA_DIR (contains database)")
        fi
    else
        info "Data directory not found: $DATA_DIR"
    fi
}

remove_log_dir() {
    echo ""
    echo -e "${BOLD}=== Log Directory ===${NC}"
    echo ""

    if [[ -d "$LOG_DIR" ]]; then
        local log_size
        log_size=$(du -sh "$LOG_DIR" 2>/dev/null | cut -f1)
        echo "Log directory: $LOG_DIR ($log_size)"
        echo ""

        if prompt_yn "Remove log directory ($LOG_DIR)?" "y"; then
            rm -rf "$LOG_DIR"
            success "Removed $LOG_DIR"
            REMOVED_ITEMS+=("$LOG_DIR")
        else
            KEPT_ITEMS+=("$LOG_DIR")
        fi
    else
        info "Log directory not found: $LOG_DIR"
    fi
}

remove_cache_dir() {
    echo ""
    echo -e "${BOLD}=== Cache Directory ===${NC}"
    echo ""

    if [[ -d "$CACHE_DIR" ]]; then
        local cache_size
        cache_size=$(du -sh "$CACHE_DIR" 2>/dev/null | cut -f1)
        echo "Cache directory: $CACHE_DIR ($cache_size)"
        echo ""

        if prompt_yn "Remove cache directory ($CACHE_DIR)?" "y"; then
            rm -rf "$CACHE_DIR"
            success "Removed $CACHE_DIR"
            REMOVED_ITEMS+=("$CACHE_DIR")
        else
            KEPT_ITEMS+=("$CACHE_DIR")
        fi
    else
        info "Cache directory not found: $CACHE_DIR"
    fi
}

remove_user() {
    echo ""
    echo -e "${BOLD}=== Service User ===${NC}"
    echo ""

    if id "$SERVICE_USER" &>/dev/null; then
        info "Found service user: $SERVICE_USER"

        if prompt_yn "Remove service user ($SERVICE_USER)?" "y"; then
            userdel "$SERVICE_USER" 2>/dev/null || true
            success "Removed user $SERVICE_USER"
            REMOVED_ITEMS+=("user $SERVICE_USER")
        else
            KEPT_ITEMS+=("user $SERVICE_USER")
        fi
    else
        info "Service user not found: $SERVICE_USER"
    fi
}

#######################################
# Summary
#######################################

print_summary() {
    echo ""
    echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║                    Uninstall Summary                         ║${NC}"
    echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    if [[ ${#REMOVED_ITEMS[@]} -gt 0 ]]; then
        echo -e "${GREEN}Removed:${NC}"
        for item in "${REMOVED_ITEMS[@]}"; do
            echo "  - $item"
        done
        echo ""
    fi

    if [[ ${#KEPT_ITEMS[@]} -gt 0 ]]; then
        echo -e "${YELLOW}Kept (not removed):${NC}"
        for item in "${KEPT_ITEMS[@]}"; do
            echo "  - $item"
        done
        echo ""
        echo "To remove kept items later, delete them manually:"
        for item in "${KEPT_ITEMS[@]}"; do
            case "$item" in
                *"$CONFIG_DIR"*) echo "  sudo rm -rf $CONFIG_DIR" ;;
                *"$DATA_DIR"*) echo "  sudo rm -rf $DATA_DIR" ;;
                *"$LOG_DIR"*) echo "  sudo rm -rf $LOG_DIR" ;;
                *"$CACHE_DIR"*) echo "  sudo rm -rf $CACHE_DIR" ;;
                *"$INSTALL_DIR"*) echo "  sudo rm -rf $INSTALL_DIR" ;;
                *"user"*) echo "  sudo userdel $SERVICE_USER" ;;
            esac
        done
        echo ""
    fi

    if [[ ${#REMOVED_ITEMS[@]} -gt 0 ]]; then
        success "Cuttlefish has been uninstalled!"
    else
        info "Nothing was removed."
    fi
}

#######################################
# Main
#######################################

main() {
    print_banner

    echo "This script will remove Cuttlefish and its associated files."
    echo "You will be prompted before removing each component."
    echo ""

    check_root

    echo -e "${RED}${BOLD}WARNING: This will remove Cuttlefish from your system.${NC}"
    echo ""

    if ! prompt_yn "Continue with uninstallation?" "n"; then
        info "Uninstallation cancelled."
        exit 0
    fi

    # Quick uninstall option
    echo ""
    echo "Uninstall options:"
    echo "  1) Interactive - Prompt for each component"
    echo "  2) Complete - Remove everything (keeps data/config by default)"
    echo "  3) Purge - Remove absolutely everything including data"
    echo ""

    local mode
    read -rp "$(echo -e "${CYAN}Select option [1-3]${NC} [1]: ")" mode
    mode="${mode:-1}"

    case "$mode" in
        2)
            # Complete - remove all but prompt for data/config
            REMOVE_ALL=true
            PURGE=false
            ;;
        3)
            # Purge - remove everything without prompts
            REMOVE_ALL=true
            PURGE=true
            warn "PURGE MODE: All data will be permanently deleted!"
            if ! prompt_yn "Are you absolutely sure?" "n"; then
                info "Uninstallation cancelled."
                exit 0
            fi
            ;;
        *)
            # Interactive
            REMOVE_ALL=false
            PURGE=false
            ;;
    esac

    stop_services
    remove_docker_resources
    remove_systemd_service
    remove_symlinks
    remove_install_dir

    if [[ "$PURGE" == true ]]; then
        # Purge mode - remove everything without prompting
        echo ""
        echo -e "${BOLD}=== Purging All Data ===${NC}"
        echo ""

        for dir in "$CONFIG_DIR" "$DATA_DIR" "$LOG_DIR" "$CACHE_DIR"; do
            if [[ -d "$dir" ]]; then
                rm -rf "$dir"
                success "Removed $dir"
                REMOVED_ITEMS+=("$dir")
            fi
        done

        if id "$SERVICE_USER" &>/dev/null; then
            userdel "$SERVICE_USER" 2>/dev/null || true
            success "Removed user $SERVICE_USER"
            REMOVED_ITEMS+=("user $SERVICE_USER")
        fi
    else
        remove_config_dir
        remove_data_dir
        remove_log_dir
        remove_cache_dir
        remove_user
    fi

    print_summary
}

# Run main function
main "$@"
