#!/usr/bin/env bash
#
# Cuttlefish Install Script
# Guides you through setting up Cuttlefish on a fresh system.
#
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Configuration
INSTALL_DIR="${INSTALL_DIR:-/opt/cuttlefish}"
CONFIG_DIR="${CONFIG_DIR:-/etc/cuttlefish}"
DATA_DIR="${DATA_DIR:-/var/lib/cuttlefish}"
LOG_DIR="${LOG_DIR:-/var/log/cuttlefish}"
RUST_VERSION="1.94.0"
SERVICE_USER="cuttlefish"

# State
ENABLE_DISCORD=false
ENABLE_WEBUI=true
AWS_CONFIGURED=false

#######################################
# Utility Functions
#######################################

print_banner() {
    echo -e "${CYAN}"
    echo '   ____      _   _   _       __ _     _     '
    echo '  / ___|   _| |_| |_| | ___ / _(_)___| |__  '
    echo ' | |  | | | | __| __| |/ _ \ |_| / __| `_ \ '
    echo ' | |__| |_| | |_| |_| |  __/  _| \__ \ | | |'
    echo '  \____\__,_|\__|\__|_|\___|_| |_|___/_| |_|'
    echo -e "${NC}"
    echo -e "${BOLD}Multi-Agent, Multi-Model AI Coding Platform${NC}"
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

prompt() {
    local varname="$1"
    local message="$2"
    local default="${3:-}"
    
    if [[ -n "$default" ]]; then
        echo -en "${CYAN}$message${NC} [${default}]: "
    else
        echo -en "${CYAN}$message${NC}: "
    fi
    
    read -r input
    if [[ -z "$input" && -n "$default" ]]; then
        eval "$varname=\"$default\""
    else
        eval "$varname=\"$input\""
    fi
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

prompt_secret() {
    local varname="$1"
    local message="$2"
    
    echo -en "${CYAN}$message${NC}: "
    read -rs input
    echo ""
    eval "$varname=\"$input\""
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root (or with sudo)"
        exit 1
    fi
}

#######################################
# Dependency Checks
#######################################

check_dependencies() {
    echo ""
    echo -e "${BOLD}Checking dependencies...${NC}"
    echo ""
    
    local missing=()
    
    # Check for required commands
    for cmd in curl git docker; do
        if command -v "$cmd" &> /dev/null; then
            success "$cmd is installed"
        else
            missing+=("$cmd")
            error "$cmd is not installed"
        fi
    done
    
    # Check Docker is running
    if command -v docker &> /dev/null; then
        if docker info &> /dev/null; then
            success "Docker daemon is running"
        else
            error "Docker is installed but not running"
            missing+=("docker-running")
        fi
    fi
    
    # Check for Rust
    if command -v rustc &> /dev/null; then
        local rust_ver
        rust_ver=$(rustc --version | awk '{print $2}')
        success "Rust $rust_ver is installed"
    else
        warn "Rust is not installed (will install)"
    fi
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        echo ""
        error "Missing dependencies: ${missing[*]}"
        echo ""
        if prompt_yn "Would you like to install missing dependencies?"; then
            install_dependencies "${missing[@]}"
        else
            error "Cannot continue without dependencies"
            exit 1
        fi
    fi
}

install_dependencies() {
    local deps=("$@")
    
    # Detect package manager
    if command -v apt-get &> /dev/null; then
        PKG_MANAGER="apt-get"
        PKG_UPDATE="apt-get update"
        PKG_INSTALL="apt-get install -y"
    elif command -v dnf &> /dev/null; then
        PKG_MANAGER="dnf"
        PKG_UPDATE="dnf check-update || true"
        PKG_INSTALL="dnf install -y"
    elif command -v pacman &> /dev/null; then
        PKG_MANAGER="pacman"
        PKG_UPDATE="pacman -Sy"
        PKG_INSTALL="pacman -S --noconfirm"
    else
        error "No supported package manager found (apt, dnf, pacman)"
        exit 1
    fi
    
    info "Updating package lists..."
    $PKG_UPDATE
    
    for dep in "${deps[@]}"; do
        case "$dep" in
            curl|git)
                info "Installing $dep..."
                $PKG_INSTALL "$dep"
                ;;
            docker)
                info "Installing Docker..."
                if [[ "$PKG_MANAGER" == "apt-get" ]]; then
                    $PKG_INSTALL docker.io
                else
                    $PKG_INSTALL docker
                fi
                systemctl enable docker
                systemctl start docker
                ;;
            docker-running)
                info "Starting Docker..."
                systemctl start docker
                ;;
        esac
    done
}

install_rust() {
    if ! command -v rustc &> /dev/null; then
        info "Installing Rust ${RUST_VERSION}..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain "$RUST_VERSION"
        source "$HOME/.cargo/env"
        success "Rust installed"
    else
        local current_ver
        current_ver=$(rustc --version | awk '{print $2}')
        if [[ "$current_ver" != "$RUST_VERSION"* ]]; then
            info "Updating Rust to ${RUST_VERSION}..."
            rustup install "$RUST_VERSION"
            rustup default "$RUST_VERSION"
        fi
    fi
}

#######################################
# Configuration
#######################################

configure_server() {
    echo ""
    echo -e "${BOLD}=== Server Configuration ===${NC}"
    echo ""
    
    prompt SERVER_HOST "Server bind address" "0.0.0.0"
    prompt SERVER_PORT "Server port" "8080"
    
    # Generate API key if not provided
    if prompt_yn "Generate a random API key?" "y"; then
        API_KEY=$(openssl rand -hex 32)
        success "Generated API key: ${API_KEY:0:8}...${API_KEY: -8}"
    else
        prompt_secret API_KEY "Enter API key"
    fi
}

configure_database() {
    echo ""
    echo -e "${BOLD}=== Database Configuration ===${NC}"
    echo ""
    
    prompt DB_PATH "Database file path" "${DATA_DIR}/cuttlefish.db"
}

configure_sandbox() {
    echo ""
    echo -e "${BOLD}=== Docker Sandbox Configuration ===${NC}"
    echo ""
    
    prompt DOCKER_SOCKET "Docker socket path" "unix:///var/run/docker.sock"
    prompt MEMORY_LIMIT "Memory limit per sandbox (MB)" "2048"
    prompt CPU_LIMIT "CPU limit per sandbox" "2.0"
    prompt DISK_LIMIT "Disk limit per sandbox (GB)" "10"
    prompt MAX_CONCURRENT "Maximum concurrent sandboxes" "5"
}

configure_discord() {
    echo ""
    echo -e "${BOLD}=== Discord Bot Configuration ===${NC}"
    echo ""
    
    if prompt_yn "Enable Discord bot?" "n"; then
        ENABLE_DISCORD=true
        echo ""
        echo "To get a Discord bot token:"
        echo "  1. Go to https://discord.com/developers/applications"
        echo "  2. Create a new application"
        echo "  3. Go to Bot section, create bot, copy token"
        echo ""
        prompt_secret DISCORD_TOKEN "Discord bot token"
        prompt DISCORD_GUILD_IDS "Guild IDs (comma-separated, or empty for global)" ""
    fi
}

configure_aws() {
    echo ""
    echo -e "${BOLD}=== AWS Bedrock Configuration ===${NC}"
    echo ""
    
    echo "Cuttlefish uses AWS Bedrock for AI models."
    echo "You need AWS credentials with Bedrock access."
    echo ""
    
    if prompt_yn "Configure AWS credentials now?" "y"; then
        AWS_CONFIGURED=true
        prompt AWS_REGION "AWS region" "us-east-1"
        prompt_secret AWS_ACCESS_KEY_ID "AWS Access Key ID"
        prompt_secret AWS_SECRET_ACCESS_KEY "AWS Secret Access Key"
        
        # Verify Bedrock access
        echo ""
        info "Testing AWS Bedrock access..."
        if AWS_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID" \
           AWS_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY" \
           AWS_DEFAULT_REGION="$AWS_REGION" \
           aws bedrock list-foundation-models --max-results 1 &> /dev/null 2>&1; then
            success "AWS Bedrock access verified"
        else
            warn "Could not verify Bedrock access (continuing anyway)"
        fi
    else
        warn "Skipping AWS configuration — you'll need to set credentials manually"
        AWS_REGION="us-east-1"
    fi
    
    # Model selection
    echo ""
    echo "Available model presets:"
    echo "  1) Claude 3.5 Sonnet (recommended)"
    echo "  2) Claude 3 Opus (most capable, expensive)"
    echo "  3) Claude 3 Haiku (fast, cheap)"
    echo ""
    prompt MODEL_CHOICE "Select default model [1-3]" "1"
    
    case "$MODEL_CHOICE" in
        1) MODEL_ID="anthropic.claude-3-5-sonnet-20241022-v2:0" ;;
        2) MODEL_ID="anthropic.claude-3-opus-20240229-v1:0" ;;
        3) MODEL_ID="anthropic.claude-3-haiku-20240307-v1:0" ;;
        *) MODEL_ID="anthropic.claude-3-5-sonnet-20241022-v2:0" ;;
    esac
}

#######################################
# Installation
#######################################

create_user() {
    if ! id "$SERVICE_USER" &> /dev/null; then
        info "Creating service user: $SERVICE_USER"
        useradd --system --shell /usr/sbin/nologin --home-dir "$INSTALL_DIR" "$SERVICE_USER"
        usermod -aG docker "$SERVICE_USER"
        success "Created user $SERVICE_USER"
    else
        success "User $SERVICE_USER already exists"
    fi
}

create_directories() {
    info "Creating directories..."
    
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$DATA_DIR"
    mkdir -p "$LOG_DIR"
    
    chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$LOG_DIR"
    
    success "Created directories"
}

build_cuttlefish() {
    echo ""
    echo -e "${BOLD}=== Building Cuttlefish ===${NC}"
    echo ""
    
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    # Check if we're in the source directory
    if [[ -f "$script_dir/Cargo.toml" ]]; then
        info "Building from source directory: $script_dir"
        cd "$script_dir"
    else
        # Clone from GitHub
        prompt REPO_URL "Git repository URL" "https://github.com/YOUR_USER/cuttlefish-rs.git"
        info "Cloning repository..."
        git clone "$REPO_URL" /tmp/cuttlefish-build
        cd /tmp/cuttlefish-build
    fi
    
    # Ensure Rust is available
    if [[ -f "$HOME/.cargo/env" ]]; then
        source "$HOME/.cargo/env"
    fi
    
    info "Building release binary (this may take a few minutes)..."
    cargo build --release
    
    # Copy binary
    cp target/release/cuttlefish-rs "$INSTALL_DIR/"
    chown "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/cuttlefish-rs"
    chmod +x "$INSTALL_DIR/cuttlefish-rs"
    
    # Copy TUI binary if it exists
    if [[ -f target/release/cuttlefish-tui ]]; then
        cp target/release/cuttlefish-tui "$INSTALL_DIR/"
        chown "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/cuttlefish-tui"
        chmod +x "$INSTALL_DIR/cuttlefish-tui"
    fi
    
    success "Built and installed binaries to $INSTALL_DIR"
}

write_config() {
    info "Writing configuration..."
    
    # Parse guild IDs
    local guild_ids_toml="[]"
    if [[ -n "${DISCORD_GUILD_IDS:-}" ]]; then
        guild_ids_toml="[$(echo "$DISCORD_GUILD_IDS" | tr ',' ', ')]"
    fi
    
    cat > "$CONFIG_DIR/cuttlefish.toml" << EOF
# Cuttlefish Configuration
# Generated by install.sh on $(date)

[server]
host = "$SERVER_HOST"
port = $SERVER_PORT

[database]
path = "$DB_PATH"

[sandbox]
docker_socket = "$DOCKER_SOCKET"
memory_limit_mb = $MEMORY_LIMIT
cpu_limit = $CPU_LIMIT
disk_limit_gb = $DISK_LIMIT
max_concurrent = $MAX_CONCURRENT

[providers.claude]
provider_type = "bedrock"
model = "$MODEL_ID"
region = "$AWS_REGION"

[providers.claude-fast]
provider_type = "bedrock"
model = "anthropic.claude-3-haiku-20240307-v1:0"
region = "$AWS_REGION"

[agents.orchestrator]
category = "deep"

[agents.coder]
category = "deep"

[agents.critic]
category = "unspecified-high"
EOF

    if [[ "$ENABLE_DISCORD" == true ]]; then
        cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[discord]
token_env_var = "DISCORD_BOT_TOKEN"
guild_ids = $guild_ids_toml
EOF
    fi
    
    chmod 640 "$CONFIG_DIR/cuttlefish.toml"
    chown root:"$SERVICE_USER" "$CONFIG_DIR/cuttlefish.toml"
    
    success "Configuration written to $CONFIG_DIR/cuttlefish.toml"
}

write_env_file() {
    info "Writing environment file..."
    
    cat > "$CONFIG_DIR/cuttlefish.env" << EOF
# Cuttlefish Environment Variables
# Generated by install.sh on $(date)

# API Key for WebUI/TUI authentication
CUTTLEFISH_API_KEY=$API_KEY

# Logging level
RUST_LOG=info
EOF

    if [[ "$AWS_CONFIGURED" == true ]]; then
        cat >> "$CONFIG_DIR/cuttlefish.env" << EOF

# AWS Credentials for Bedrock
AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY
AWS_DEFAULT_REGION=$AWS_REGION
EOF
    fi
    
    if [[ "$ENABLE_DISCORD" == true ]]; then
        cat >> "$CONFIG_DIR/cuttlefish.env" << EOF

# Discord Bot Token
DISCORD_BOT_TOKEN=$DISCORD_TOKEN
EOF
    fi
    
    chmod 600 "$CONFIG_DIR/cuttlefish.env"
    chown root:"$SERVICE_USER" "$CONFIG_DIR/cuttlefish.env"
    
    success "Environment file written to $CONFIG_DIR/cuttlefish.env"
}

write_systemd_service() {
    info "Creating systemd service..."
    
    cat > /etc/systemd/system/cuttlefish.service << EOF
[Unit]
Description=Cuttlefish Multi-Agent AI Coding Platform
Documentation=https://github.com/YOUR_USER/cuttlefish-rs
After=network.target docker.service
Requires=docker.service

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_USER
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$CONFIG_DIR/cuttlefish.env
ExecStart=$INSTALL_DIR/cuttlefish-rs --config $CONFIG_DIR/cuttlefish.toml
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
ReadWritePaths=$DATA_DIR $LOG_DIR

# Resource limits
LimitNOFILE=65535
MemoryMax=4G

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    success "Systemd service created"
}

#######################################
# Finalization
#######################################

print_summary() {
    echo ""
    echo -e "${GREEN}${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}${BOLD}║            Cuttlefish Installation Complete!                 ║${NC}"
    echo -e "${GREEN}${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BOLD}Installation Summary:${NC}"
    echo "  Binary:        $INSTALL_DIR/cuttlefish-rs"
    echo "  Config:        $CONFIG_DIR/cuttlefish.toml"
    echo "  Environment:   $CONFIG_DIR/cuttlefish.env"
    echo "  Database:      $DB_PATH"
    echo "  Logs:          $LOG_DIR/"
    echo ""
    echo -e "${BOLD}Service Commands:${NC}"
    echo "  Start:         sudo systemctl start cuttlefish"
    echo "  Stop:          sudo systemctl stop cuttlefish"
    echo "  Status:        sudo systemctl status cuttlefish"
    echo "  Logs:          sudo journalctl -u cuttlefish -f"
    echo "  Enable:        sudo systemctl enable cuttlefish"
    echo ""
    echo -e "${BOLD}Access:${NC}"
    echo "  WebUI:         http://${SERVER_HOST}:${SERVER_PORT}"
    echo "  API Key:       ${API_KEY:0:8}...${API_KEY: -8}"
    if [[ "$ENABLE_DISCORD" == true ]]; then
        echo "  Discord:       Bot enabled"
    fi
    echo ""
    echo -e "${BOLD}TUI Client:${NC}"
    echo "  $INSTALL_DIR/cuttlefish-tui --server ws://${SERVER_HOST}:${SERVER_PORT} --api-key \$API_KEY"
    echo ""
    
    if prompt_yn "Start Cuttlefish now?" "y"; then
        systemctl enable cuttlefish
        systemctl start cuttlefish
        sleep 2
        if systemctl is-active --quiet cuttlefish; then
            success "Cuttlefish is running!"
        else
            error "Failed to start. Check: journalctl -u cuttlefish -n 50"
        fi
    else
        info "Run 'sudo systemctl start cuttlefish' when ready"
    fi
}

#######################################
# Main
#######################################

main() {
    print_banner
    
    echo "This script will guide you through setting up Cuttlefish."
    echo "You can press Ctrl+C at any time to abort."
    echo ""
    
    if [[ "${1:-}" != "--no-root-check" ]]; then
        check_root
    fi
    
    check_dependencies
    install_rust
    
    configure_server
    configure_database
    configure_sandbox
    configure_discord
    configure_aws
    
    echo ""
    echo -e "${BOLD}=== Installation ===${NC}"
    echo ""
    
    create_user
    create_directories
    build_cuttlefish
    write_config
    write_env_file
    write_systemd_service
    
    print_summary
}

# Run main function
main "$@"
