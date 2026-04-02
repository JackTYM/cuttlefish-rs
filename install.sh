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

# Provider selections (space-separated list of selected provider indices)
SELECTED_PROVIDERS=""

# Provider definitions - using indexed arrays (bash 3.2+ compatible)
# Format: "index:id:name:type:key_prefix:models"
PROVIDERS_DATA="
1:anthropic:Anthropic:api:sk-ant-:claude-opus-4-6, claude-sonnet-4-6, claude-haiku-4-5
2:openai:OpenAI:api:sk-:gpt-5.4, gpt-5-nano, gpt-4o
3:google:Google Gemini:api:AIza:gemini-2.0-flash
4:moonshot:Moonshot/Kimi:api:sk-:kimi-k2.5
5:zhipu:Zhipu/GLM:api::glm-4-flash
6:minimax:MiniMax:api::abab6.5s-chat
7:xai:xAI/Grok:api:xai-:grok-2
8:claude_oauth:Claude OAuth:oauth::claude.ai account - no API key needed
9:ollama:Ollama:local::local models - no API key needed
10:bedrock:AWS Bedrock:cloud::requires AWS credentials
11:azure:Azure OpenAI:cloud::requires Azure subscription
"

# Collected API keys (will be populated during configuration)
ANTHROPIC_API_KEY=""
OPENAI_API_KEY=""
GOOGLE_API_KEY=""
MOONSHOT_API_KEY=""
ZHIPU_API_KEY=""
MINIMAX_API_KEY=""
XAI_API_KEY=""
OLLAMA_MODEL=""
AZURE_ENDPOINT=""
AZURE_API_KEY=""

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

get_provider_field() {
    local idx="$1"
    local field="$2"
    local line
    line=$(echo "$PROVIDERS_DATA" | grep "^$idx:")
    case "$field" in
        id)       echo "$line" | cut -d: -f2 ;;
        name)     echo "$line" | cut -d: -f3 ;;
        type)     echo "$line" | cut -d: -f4 ;;
        prefix)   echo "$line" | cut -d: -f5 ;;
        models)   echo "$line" | cut -d: -f6 ;;
    esac
}

is_provider_selected() {
    local idx="$1"
    echo " $SELECTED_PROVIDERS " | grep -q " $idx "
}

toggle_provider() {
    local idx="$1"
    if is_provider_selected "$idx"; then
        SELECTED_PROVIDERS=$(echo " $SELECTED_PROVIDERS " | sed "s/ $idx / /g" | xargs)
    else
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS $idx"
        SELECTED_PROVIDERS=$(echo "$SELECTED_PROVIDERS" | xargs)
    fi
}

print_provider_menu() {
    clear
    echo -e "${BOLD}=== Model Provider Setup ===${NC}"
    echo ""
    echo "Select which providers to configure (enter number to toggle, 'done' to continue):"
    echo ""
    echo -e "${YELLOW}API Key Providers:${NC}"
    
    local idx marker name models
    for idx in 1 2 3 4 5 6 7; do
        if is_provider_selected "$idx"; then
            marker="${GREEN}[*]${NC}"
        else
            marker="[ ]"
        fi
        name=$(get_provider_field "$idx" name)
        models=$(get_provider_field "$idx" models)
        printf "  %s %2d) %-20s (%s)\n" "$marker" "$idx" "$name" "$models"
    done
    
    echo ""
    echo -e "${YELLOW}OAuth Providers:${NC}"
    for idx in 8; do
        if is_provider_selected "$idx"; then
            marker="${GREEN}[*]${NC}"
        else
            marker="[ ]"
        fi
        name=$(get_provider_field "$idx" name)
        models=$(get_provider_field "$idx" models)
        printf "  %s %2d) %-20s (%s)\n" "$marker" "$idx" "$name" "$models"
    done
    
    echo ""
    echo -e "${YELLOW}Local Providers:${NC}"
    for idx in 9; do
        if is_provider_selected "$idx"; then
            marker="${GREEN}[*]${NC}"
        else
            marker="[ ]"
        fi
        name=$(get_provider_field "$idx" name)
        models=$(get_provider_field "$idx" models)
        printf "  %s %2d) %-20s (%s)\n" "$marker" "$idx" "$name" "$models"
    done
    
    echo ""
    echo -e "${YELLOW}Cloud Providers:${NC}"
    for idx in 10 11; do
        if is_provider_selected "$idx"; then
            marker="${GREEN}[*]${NC}"
        else
            marker="[ ]"
        fi
        name=$(get_provider_field "$idx" name)
        models=$(get_provider_field "$idx" models)
        printf "  %s %2d) %-20s (%s)\n" "$marker" "$idx" "$name" "$models"
    done
    
    echo ""
}

select_providers() {
    SELECTED_PROVIDERS="1"
    
    while true; do
        print_provider_menu
        echo -en "${CYAN}Enter provider number to toggle (or 'done' to continue): ${NC}"
        read -r choice
        
        case "$choice" in
            done|d|D|DONE|Done)
                if [[ -z "$SELECTED_PROVIDERS" ]]; then
                    error "You must select at least one provider"
                    sleep 1
                    continue
                fi
                break
                ;;
            [1-9]|1[01])
                if [[ "$choice" -ge 1 && "$choice" -le 11 ]]; then
                    toggle_provider "$choice"
                else
                    warn "Invalid choice. Enter 1-11 or 'done'"
                    sleep 1
                fi
                ;;
            *)
                warn "Invalid input. Enter a provider number (1-11) or 'done'"
                sleep 1
                ;;
        esac
    done
    
    echo ""
    success "Selected providers: $(for idx in $SELECTED_PROVIDERS; do get_provider_field "$idx" name; done | tr '\n' ', ' | sed 's/,$//')"
}

validate_api_key() {
    local provider_id="$1"
    local key="$2"
    local prefix
    
    if [[ -z "$key" ]]; then
        return 1
    fi
    
    if [[ ${#key} -lt 10 ]]; then
        return 1
    fi
    
    prefix=$(get_provider_field "$provider_id" prefix 2>/dev/null || echo "")
    if [[ -n "$prefix" && ! "$key" == "$prefix"* ]]; then
        warn "Key doesn't match expected prefix ($prefix...). Continuing anyway."
    fi
    
    return 0
}

collect_provider_credentials() {
    echo ""
    echo -e "${BOLD}=== Provider API Keys ===${NC}"
    echo ""
    
    for idx in $SELECTED_PROVIDERS; do
        local ptype pname pid
        ptype=$(get_provider_field "$idx" type)
        pname=$(get_provider_field "$idx" name)
        pid=$(get_provider_field "$idx" id)
        
        case "$ptype" in
            api)
                echo -e "${CYAN}$pname requires an API key.${NC}"
                local key_valid=false
                while ! $key_valid; do
                    prompt_secret API_KEY_TMP "Enter your $pname API key"
                    if validate_api_key "$idx" "$API_KEY_TMP"; then
                        key_valid=true
                    else
                        error "Invalid API key. Please try again."
                    fi
                done
                
                case "$pid" in
                    anthropic)  ANTHROPIC_API_KEY="$API_KEY_TMP" ;;
                    openai)     OPENAI_API_KEY="$API_KEY_TMP" ;;
                    google)     GOOGLE_API_KEY="$API_KEY_TMP" ;;
                    moonshot)   MOONSHOT_API_KEY="$API_KEY_TMP" ;;
                    zhipu)      ZHIPU_API_KEY="$API_KEY_TMP" ;;
                    minimax)    MINIMAX_API_KEY="$API_KEY_TMP" ;;
                    xai)        XAI_API_KEY="$API_KEY_TMP" ;;
                esac
                success "$pname API key saved"
                echo ""
                ;;
            
            oauth)
                info "$pname uses OAuth - no API key needed."
                info "You'll authenticate via browser when first connecting."
                echo ""
                ;;
            
            local)
                configure_ollama
                ;;
            
            cloud)
                case "$pid" in
                    bedrock)
                        configure_aws_provider
                        ;;
                    azure)
                        configure_azure_provider
                        ;;
                esac
                ;;
        esac
    done
}

configure_ollama() {
    echo ""
    echo -e "${BOLD}=== Ollama Configuration ===${NC}"
    echo ""
    
    if command -v ollama &> /dev/null; then
        success "Ollama is already installed"
    else
        warn "Ollama is not installed"
        if prompt_yn "Would you like to install Ollama now?" "y"; then
            info "Installing Ollama..."
            curl -fsSL https://ollama.com/install.sh | sh
            if command -v ollama &> /dev/null; then
                success "Ollama installed successfully"
            else
                error "Failed to install Ollama"
                return
            fi
        else
            warn "Skipping Ollama installation"
            return
        fi
    fi
    
    echo ""
    echo "Popular models:"
    echo "  1) llama3.1 (8B - general purpose)"
    echo "  2) llama3.1:70b (70B - powerful)"
    echo "  3) codellama (code-optimized)"
    echo "  4) mistral (7B - fast)"
    echo "  5) deepseek-coder-v2 (coding)"
    echo "  6) custom (enter model name)"
    echo ""
    
    prompt MODEL_CHOICE "Select model to pull [1-6]" "1"
    
    case "$MODEL_CHOICE" in
        1) OLLAMA_MODEL="llama3.1" ;;
        2) OLLAMA_MODEL="llama3.1:70b" ;;
        3) OLLAMA_MODEL="codellama" ;;
        4) OLLAMA_MODEL="mistral" ;;
        5) OLLAMA_MODEL="deepseek-coder-v2" ;;
        6) prompt OLLAMA_MODEL "Enter model name" "llama3.1" ;;
        *) OLLAMA_MODEL="llama3.1" ;;
    esac
    
    if prompt_yn "Pull $OLLAMA_MODEL now? (can take a while)" "y"; then
        info "Pulling $OLLAMA_MODEL..."
        if ollama pull "$OLLAMA_MODEL"; then
            success "Model $OLLAMA_MODEL pulled successfully"
        else
            warn "Failed to pull model. You can run 'ollama pull $OLLAMA_MODEL' later."
        fi
    else
        info "You can pull the model later with: ollama pull $OLLAMA_MODEL"
    fi
    echo ""
}

configure_aws_provider() {
    echo ""
    echo -e "${BOLD}=== AWS Bedrock Configuration ===${NC}"
    echo ""
    
    echo "AWS Bedrock requires AWS credentials with Bedrock access."
    echo ""
    
    if prompt_yn "Configure AWS credentials now?" "y"; then
        AWS_CONFIGURED=true
        prompt AWS_REGION "AWS region" "us-east-1"
        prompt_secret AWS_ACCESS_KEY_ID "AWS Access Key ID"
        prompt_secret AWS_SECRET_ACCESS_KEY "AWS Secret Access Key"
        
        echo ""
        info "Testing AWS Bedrock access..."
        if command -v aws &> /dev/null; then
            if AWS_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID" \
               AWS_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY" \
               AWS_DEFAULT_REGION="$AWS_REGION" \
               aws bedrock list-foundation-models --max-results 1 &> /dev/null 2>&1; then
                success "AWS Bedrock access verified"
            else
                warn "Could not verify Bedrock access (continuing anyway)"
            fi
        else
            warn "AWS CLI not installed, skipping verification"
        fi
    else
        warn "Skipping AWS configuration - set credentials manually"
        AWS_REGION="us-east-1"
    fi
    echo ""
}

configure_azure_provider() {
    echo ""
    echo -e "${BOLD}=== Azure OpenAI Configuration ===${NC}"
    echo ""
    
    echo "Azure OpenAI requires an Azure subscription and deployed model."
    echo ""
    
    prompt AZURE_ENDPOINT "Azure OpenAI endpoint URL" ""
    prompt_secret AZURE_API_KEY "Azure OpenAI API key"
    
    success "Azure OpenAI configured"
    echo ""
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
EOF

    local first_deep_provider=""
    local first_quick_provider=""
    
    for idx in $SELECTED_PROVIDERS; do
        local pid
        pid=$(get_provider_field "$idx" id)
        
        case "$pid" in
            anthropic)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.anthropic]
provider_type = "anthropic"
model = "claude-sonnet-4-6"
api_key_env = "ANTHROPIC_API_KEY"

[providers.anthropic-fast]
provider_type = "anthropic"
model = "claude-haiku-4-5"
api_key_env = "ANTHROPIC_API_KEY"
EOF
                [[ -z "$first_deep_provider" ]] && first_deep_provider="anthropic"
                [[ -z "$first_quick_provider" ]] && first_quick_provider="anthropic-fast"
                ;;
                
            openai)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.openai]
provider_type = "openai"
model = "gpt-5.4"
api_key_env = "OPENAI_API_KEY"

[providers.openai-fast]
provider_type = "openai"
model = "gpt-5-nano"
api_key_env = "OPENAI_API_KEY"
EOF
                [[ -z "$first_deep_provider" ]] && first_deep_provider="openai"
                [[ -z "$first_quick_provider" ]] && first_quick_provider="openai-fast"
                ;;
                
            google)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.google]
provider_type = "google"
model = "gemini-2.0-flash"
api_key_env = "GOOGLE_API_KEY"
EOF
                [[ -z "$first_quick_provider" ]] && first_quick_provider="google"
                ;;
                
            moonshot)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.moonshot]
provider_type = "moonshot"
model = "kimi-k2.5"
api_key_env = "MOONSHOT_API_KEY"
EOF
                [[ -z "$first_deep_provider" ]] && first_deep_provider="moonshot"
                ;;
                
            zhipu)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.zhipu]
provider_type = "zhipu"
model = "glm-4-flash"
api_key_env = "ZHIPU_API_KEY"
EOF
                [[ -z "$first_quick_provider" ]] && first_quick_provider="zhipu"
                ;;
                
            minimax)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.minimax]
provider_type = "minimax"
model = "abab6.5s-chat"
api_key_env = "MINIMAX_API_KEY"
EOF
                [[ -z "$first_quick_provider" ]] && first_quick_provider="minimax"
                ;;
                
            xai)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.xai]
provider_type = "xai"
model = "grok-2"
api_key_env = "XAI_API_KEY"
EOF
                [[ -z "$first_deep_provider" ]] && first_deep_provider="xai"
                ;;
                
            claude_oauth)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.claude-oauth]
provider_type = "claude_oauth"
model = "claude-sonnet-4-6"
EOF
                [[ -z "$first_deep_provider" ]] && first_deep_provider="claude-oauth"
                ;;
                
            ollama)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.ollama]
provider_type = "ollama"
model = "${OLLAMA_MODEL:-llama3.1}"
base_url = "http://localhost:11434"
EOF
                [[ -z "$first_deep_provider" ]] && first_deep_provider="ollama"
                [[ -z "$first_quick_provider" ]] && first_quick_provider="ollama"
                ;;
                
            bedrock)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.bedrock]
provider_type = "bedrock"
model = "anthropic.claude-3-5-sonnet-20241022-v2:0"
region = "${AWS_REGION:-us-east-1}"

[providers.bedrock-fast]
provider_type = "bedrock"
model = "anthropic.claude-3-haiku-20240307-v1:0"
region = "${AWS_REGION:-us-east-1}"
EOF
                [[ -z "$first_deep_provider" ]] && first_deep_provider="bedrock"
                [[ -z "$first_quick_provider" ]] && first_quick_provider="bedrock-fast"
                ;;
                
            azure)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.azure]
provider_type = "azure_openai"
endpoint = "$AZURE_ENDPOINT"
deployment = "gpt-4"
api_key_env = "AZURE_API_KEY"
EOF
                [[ -z "$first_deep_provider" ]] && first_deep_provider="azure"
                ;;
        esac
    done
    
    first_deep_provider="${first_deep_provider:-ollama}"
    first_quick_provider="${first_quick_provider:-$first_deep_provider}"
    
    cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[agents.orchestrator]
category = "deep"

[agents.coder]
category = "deep"

[agents.critic]
category = "unspecified-high"

[routing]
deep = "$first_deep_provider"
quick = "$first_quick_provider"
ultrabrain = "$first_deep_provider"
visual = "$first_deep_provider"
unspecified_high = "$first_deep_provider"
unspecified_low = "$first_quick_provider"
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

CUTTLEFISH_API_KEY=$API_KEY
RUST_LOG=info
EOF

    for idx in $SELECTED_PROVIDERS; do
        local pid
        pid=$(get_provider_field "$idx" id)
        
        case "$pid" in
            anthropic)
                [[ -n "$ANTHROPIC_API_KEY" ]] && echo "ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY" >> "$CONFIG_DIR/cuttlefish.env"
                ;;
            openai)
                [[ -n "$OPENAI_API_KEY" ]] && echo "OPENAI_API_KEY=$OPENAI_API_KEY" >> "$CONFIG_DIR/cuttlefish.env"
                ;;
            google)
                [[ -n "$GOOGLE_API_KEY" ]] && echo "GOOGLE_API_KEY=$GOOGLE_API_KEY" >> "$CONFIG_DIR/cuttlefish.env"
                ;;
            moonshot)
                [[ -n "$MOONSHOT_API_KEY" ]] && echo "MOONSHOT_API_KEY=$MOONSHOT_API_KEY" >> "$CONFIG_DIR/cuttlefish.env"
                ;;
            zhipu)
                [[ -n "$ZHIPU_API_KEY" ]] && echo "ZHIPU_API_KEY=$ZHIPU_API_KEY" >> "$CONFIG_DIR/cuttlefish.env"
                ;;
            minimax)
                [[ -n "$MINIMAX_API_KEY" ]] && echo "MINIMAX_API_KEY=$MINIMAX_API_KEY" >> "$CONFIG_DIR/cuttlefish.env"
                ;;
            xai)
                [[ -n "$XAI_API_KEY" ]] && echo "XAI_API_KEY=$XAI_API_KEY" >> "$CONFIG_DIR/cuttlefish.env"
                ;;
            bedrock)
                if [[ "$AWS_CONFIGURED" == true ]]; then
                    cat >> "$CONFIG_DIR/cuttlefish.env" << EOF2
AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY
AWS_DEFAULT_REGION=$AWS_REGION
EOF2
                fi
                ;;
            azure)
                [[ -n "$AZURE_API_KEY" ]] && echo "AZURE_API_KEY=$AZURE_API_KEY" >> "$CONFIG_DIR/cuttlefish.env"
                ;;
        esac
    done
    
    if [[ "$ENABLE_DISCORD" == true ]]; then
        echo "DISCORD_BOT_TOKEN=$DISCORD_TOKEN" >> "$CONFIG_DIR/cuttlefish.env"
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
    echo -e "${BOLD}Configured Providers:${NC}"
    for idx in $SELECTED_PROVIDERS; do
        local pname
        pname=$(get_provider_field "$idx" name)
        echo "  - $pname"
    done
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
    
    select_providers
    collect_provider_credentials
    
    configure_discord
    
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
