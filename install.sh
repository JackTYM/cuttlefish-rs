#!/usr/bin/env bash
#
# Cuttlefish Install Script
# Guides you through setting up Cuttlefish on a fresh system.
#
set -uo pipefail

# Global error handler - show useful info on failure
trap 'handle_error $? $LINENO "$BASH_COMMAND"' ERR

handle_error() {
    local exit_code=$1
    local line_no=$2
    local command=$3
    echo ""
    echo -e "${RED}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                    INSTALLATION FAILED                         ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${RED}[ERROR]${NC} Command failed with exit code $exit_code"
    echo -e "${RED}[ERROR]${NC} Line $line_no: $command"
    echo ""
    echo -e "${YELLOW}Please report this issue at:${NC}"
    echo -e "${CYAN}https://github.com/JackTYM/cuttlefish-rs/issues${NC}"
    echo ""
    echo "Include the following information:"
    echo "  - Platform: ${PLATFORM:-unknown}"
    echo "  - Distro: ${DISTRO:-unknown}"
    echo "  - Error line: $line_no"
    echo "  - Failed command: $command"
    echo ""
    exit "$exit_code"
}

# Ensure we can read from terminal even when piped
if [[ -t 0 ]]; then
    : # Already have a TTY
elif [[ -e /dev/tty ]]; then
    exec < /dev/tty
else
    echo -e "${RED}[ERROR]${NC} No TTY available for interactive input."
    echo "Please run this script in an interactive terminal."
    exit 1
fi

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

# Bedrock model selections
BEDROCK_PRIMARY_MODEL="anthropic.claude-sonnet-4-6"
BEDROCK_FAST_MODEL="anthropic.claude-haiku-4-5-20251001-v1:0"

# Platform detection state
PLATFORM=""
DISTRO=""
HAS_SYSTEMD=false
DEPLOYMENT_MODE="systemd"

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
9:chatgpt_oauth:ChatGPT OAuth:oauth::chatgpt.com account - no API key needed
10:ollama:Ollama:local::llama3.1, codellama, mistral, etc.
11:bedrock:AWS Bedrock:cloud::requires AWS credentials
12:azure:Azure OpenAI:cloud::requires Azure subscription
13:custom:Custom Endpoint:custom::provide your own API endpoint
"

# Upgrade mode (detected if existing config found)
UPGRADE_MODE=false
EXISTING_CONFIG=""

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

# Custom model/endpoint overrides (set during advanced configuration)
ANTHROPIC_CUSTOM_MODEL=""
ANTHROPIC_BASE_URL=""
OPENAI_CUSTOM_MODEL=""
OPENAI_BASE_URL=""
GOOGLE_CUSTOM_MODEL=""
MOONSHOT_CUSTOM_MODEL=""
MOONSHOT_BASE_URL=""
ZHIPU_CUSTOM_MODEL=""
ZHIPU_BASE_URL=""
MINIMAX_CUSTOM_MODEL=""
MINIMAX_BASE_URL=""
XAI_CUSTOM_MODEL=""
XAI_BASE_URL=""
OLLAMA_CUSTOM_MODEL=""
OLLAMA_BASE_URL="http://localhost:11434"

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

parse_toml_value() {
    local file="$1"
    local section="$2"
    local key="$3"
    local in_section=false
    local value=""
    
    while IFS= read -r line; do
        if [[ "$line" =~ ^\[([^\]]+)\] ]]; then
            if [[ "${BASH_REMATCH[1]}" == "$section" ]]; then
                in_section=true
            else
                in_section=false
            fi
            continue
        fi
        
        if $in_section && [[ "$line" =~ ^[[:space:]]*${key}[[:space:]]*=[[:space:]]*(.+) ]]; then
            value="${BASH_REMATCH[1]}"
            value="${value#\"}"
            value="${value%\"}"
            value="${value#\'}"
            value="${value%\'}"
            echo "$value"
            return 0
        fi
    done < "$file"
    
    echo ""
    return 1
}

detect_existing_config() {
    local config_file="$CONFIG_DIR/cuttlefish.toml"
    
    if [[ -f "$config_file" ]]; then
        EXISTING_CONFIG="$config_file"
        UPGRADE_MODE=true
        return 0
    fi
    
    return 1
}

load_existing_config() {
    if [[ -z "$EXISTING_CONFIG" || ! -f "$EXISTING_CONFIG" ]]; then
        return 1
    fi
    
    info "Loading existing configuration from $EXISTING_CONFIG"
    
    local val
    
    val=$(parse_toml_value "$EXISTING_CONFIG" "server" "host")
    [[ -n "$val" ]] && SERVER_HOST="$val"
    
    val=$(parse_toml_value "$EXISTING_CONFIG" "server" "port")
    [[ -n "$val" ]] && SERVER_PORT="$val"
    
    val=$(parse_toml_value "$EXISTING_CONFIG" "database" "path")
    [[ -n "$val" ]] && DB_PATH="$val"
    
    val=$(parse_toml_value "$EXISTING_CONFIG" "sandbox" "docker_socket")
    [[ -n "$val" ]] && DOCKER_SOCKET="$val"
    
    val=$(parse_toml_value "$EXISTING_CONFIG" "sandbox" "memory_limit_mb")
    [[ -n "$val" ]] && MEMORY_LIMIT="$val"
    
    val=$(parse_toml_value "$EXISTING_CONFIG" "sandbox" "cpu_limit")
    [[ -n "$val" ]] && CPU_LIMIT="$val"
    
    val=$(parse_toml_value "$EXISTING_CONFIG" "sandbox" "disk_limit_gb")
    [[ -n "$val" ]] && DISK_LIMIT="$val"
    
    val=$(parse_toml_value "$EXISTING_CONFIG" "sandbox" "max_concurrent")
    [[ -n "$val" ]] && MAX_CONCURRENT="$val"
    
    if grep -q "^\[discord\]" "$EXISTING_CONFIG" 2>/dev/null; then
        ENABLE_DISCORD=true
    fi
    
    success "Loaded existing configuration"
    return 0
}

load_existing_env() {
    local env_file="$CONFIG_DIR/cuttlefish.env"
    
    if [[ ! -f "$env_file" ]]; then
        return 1
    fi
    
    info "Loading existing environment from $env_file"
    
    while IFS='=' read -r key value; do
        [[ "$key" =~ ^#.*$ || -z "$key" ]] && continue
        value="${value#\"}"
        value="${value%\"}"
        
        case "$key" in
            CUTTLEFISH_API_KEY)  API_KEY="$value" ;;
            ANTHROPIC_API_KEY)   ANTHROPIC_API_KEY="$value" ;;
            OPENAI_API_KEY)      OPENAI_API_KEY="$value" ;;
            GOOGLE_API_KEY)      GOOGLE_API_KEY="$value" ;;
            MOONSHOT_API_KEY)    MOONSHOT_API_KEY="$value" ;;
            ZHIPU_API_KEY)       ZHIPU_API_KEY="$value" ;;
            MINIMAX_API_KEY)     MINIMAX_API_KEY="$value" ;;
            XAI_API_KEY)         XAI_API_KEY="$value" ;;
            AZURE_API_KEY)       AZURE_API_KEY="$value" ;;
            AWS_ACCESS_KEY_ID)   AWS_ACCESS_KEY_ID="$value"; AWS_CONFIGURED=true ;;
            AWS_SECRET_ACCESS_KEY) AWS_SECRET_ACCESS_KEY="$value" ;;
            AWS_DEFAULT_REGION)  AWS_REGION="$value" ;;
            DISCORD_BOT_TOKEN)   DISCORD_TOKEN="$value" ;;
        esac
    done < "$env_file"
    
    success "Loaded existing environment variables"
    return 0
}

detect_configured_providers() {
    if [[ -z "$EXISTING_CONFIG" || ! -f "$EXISTING_CONFIG" ]]; then
        return 1
    fi
    
    SELECTED_PROVIDERS=""
    
    if grep -q "^\[providers\.anthropic\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 1"
    fi
    if grep -q "^\[providers\.openai\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 2"
    fi
    if grep -q "^\[providers\.google\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 3"
    fi
    if grep -q "^\[providers\.moonshot\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 4"
    fi
    if grep -q "^\[providers\.zhipu\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 5"
    fi
    if grep -q "^\[providers\.minimax\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 6"
    fi
    if grep -q "^\[providers\.xai\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 7"
    fi
    if grep -q "^\[providers\.claude-oauth\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 8"
    fi
    if grep -q "^\[providers\.chatgpt-oauth\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 9"
    fi
    if grep -q "^\[providers\.ollama\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 10"
    fi
    if grep -q "^\[providers\.bedrock\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 11"
    fi
    if grep -q "^\[providers\.azure\]" "$EXISTING_CONFIG" 2>/dev/null; then
        SELECTED_PROVIDERS="$SELECTED_PROVIDERS 12"
    fi
    
    SELECTED_PROVIDERS=$(echo "$SELECTED_PROVIDERS" | xargs)
    
    if [[ -n "$SELECTED_PROVIDERS" ]]; then
        return 0
    fi
    return 1
}

is_provider_selected() {
    local idx="$1"
    echo " $SELECTED_PROVIDERS " | grep -q " $idx "
}

detect_platform() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        PLATFORM="macos"
        DISTRO=""
        HAS_SYSTEMD=false
    elif grep -qi microsoft /proc/version 2>/dev/null; then
        PLATFORM="wsl"
        DISTRO=""
        if command -v systemctl &>/dev/null && systemctl is-system-running &>/dev/null 2>&1; then
            HAS_SYSTEMD=true
        else
            HAS_SYSTEMD=false
        fi
    elif [[ -f /etc/os-release ]]; then
        PLATFORM="linux"
        # shellcheck source=/dev/null
        source /etc/os-release
        DISTRO="${ID:-unknown}"
        if command -v systemctl &>/dev/null && systemctl is-system-running &>/dev/null 2>&1; then
            HAS_SYSTEMD=true
        else
            HAS_SYSTEMD=false
        fi
    else
        error "Unsupported platform. Supported: Linux, macOS, WSL"
        exit 1
    fi
    
    info "Detected platform: $PLATFORM${DISTRO:+ ($DISTRO)}, systemd: $HAS_SYSTEMD"
}

show_docker_mode_warning() {
    echo ""
    echo -e "${RED}${BOLD}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}${BOLD}║  ⚠️  SECURITY WARNING: Docker Container Mode Selected         ║${NC}"
    echo -e "${RED}${BOLD}╠═══════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${RED}${BOLD}║${NC}  In Docker mode, ALL projects run in the SAME container.      ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}║${NC}  There is NO isolation between projects.                      ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}║${NC}                                                               ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}║${NC}  A malicious or buggy project could:                          ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}║${NC}  • Access files from other projects                           ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}║${NC}  • Interfere with other running builds                        ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}║${NC}  • Consume all container resources                            ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}║${NC}                                                               ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}║${NC}  Only use Docker mode for:                                    ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}║${NC}  • Single-user development                                    ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}║${NC}  • Testing purposes                                           ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}║${NC}  • Environments where systemd is unavailable (macOS, WSL)     ${RED}${BOLD}║${NC}"
    echo -e "${RED}${BOLD}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    if ! prompt_yn "I understand and accept these limitations" "n"; then
        info "Returning to deployment mode selection..."
        select_deployment_mode
        return
    fi
    
    warn "Docker Container mode confirmed. Proceeding with no sandbox isolation."
    echo ""
}

select_deployment_mode() {
    echo ""
    echo -e "${BOLD}=== Deployment Mode ===${NC}"
    echo ""
    
    if [[ "$HAS_SYSTEMD" == false ]]; then
        warn "systemd is not available on this system (platform: $PLATFORM)"
        info "Automatically selecting Docker Container mode"
        DEPLOYMENT_MODE="docker"
        show_docker_mode_warning
        return
    fi
    
    echo "How should Cuttlefish be deployed?"
    echo ""
    echo -e "  ${GREEN}1) Systemd Service${NC} ${BOLD}(Recommended)${NC}"
    echo "     - Cuttlefish runs as a persistent system service"
    echo "     - Each project gets an ISOLATED Docker sandbox"
    echo "     - Automatic restart on failure"
    echo "     - Requires: Linux with systemd + Docker"
    echo ""
    echo -e "  ${YELLOW}2) Docker Container Mode${NC}"
    echo "     - Cuttlefish itself runs inside a Docker container"
    echo "     - Simpler setup, no systemd required"
    echo -e "     - ${RED}⚠️  WARNING: NO project sandbox isolation${NC}"
    echo ""
    
    local mode_choice
    prompt mode_choice "Select deployment mode [1-2]" "1"
    
    case "$mode_choice" in
        2)
            DEPLOYMENT_MODE="docker"
            show_docker_mode_warning
            ;;
        *)
            DEPLOYMENT_MODE="systemd"
            success "Systemd service mode selected"
            ;;
    esac
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
    local cursor_pos="$1"
    local line=0
    
    clear
    echo -e "${BOLD}=== Model Provider Setup ===${NC}"
    echo ""
    echo "Use ↑/↓ to navigate, SPACE to toggle, ENTER when done"
    echo ""
    echo -e "${YELLOW}API Key Providers:${NC}"
    
    local idx marker name models cursor
    for idx in 1 2 3 4 5 6 7; do
        line=$((line + 1))
        if is_provider_selected "$idx"; then
            marker="${GREEN}[✓]${NC}"
        else
            marker="[ ]"
        fi
        if [[ "$line" -eq "$cursor_pos" ]]; then
            cursor="${CYAN}▶${NC} "
        else
            cursor="  "
        fi
        name=$(get_provider_field "$idx" name)
        models=$(get_provider_field "$idx" models)
        echo -e "${cursor}${marker} ${idx}) ${name}  ${BLUE}(${models})${NC}"
    done
    
    echo ""
    echo -e "${YELLOW}OAuth Providers:${NC}"
    for idx in 8 9; do
        line=$((line + 1))
        if is_provider_selected "$idx"; then
            marker="${GREEN}[✓]${NC}"
        else
            marker="[ ]"
        fi
        if [[ "$line" -eq "$cursor_pos" ]]; then
            cursor="${CYAN}▶${NC} "
        else
            cursor="  "
        fi
        name=$(get_provider_field "$idx" name)
        models=$(get_provider_field "$idx" models)
        echo -e "${cursor}${marker} ${idx}) ${name}  ${BLUE}(${models})${NC}"
    done
    
    echo ""
    echo -e "${YELLOW}Local Providers:${NC}"
    for idx in 10; do
        line=$((line + 1))
        if is_provider_selected "$idx"; then
            marker="${GREEN}[✓]${NC}"
        else
            marker="[ ]"
        fi
        if [[ "$line" -eq "$cursor_pos" ]]; then
            cursor="${CYAN}▶${NC} "
        else
            cursor="  "
        fi
        name=$(get_provider_field "$idx" name)
        models=$(get_provider_field "$idx" models)
        echo -e "${cursor}${marker} ${idx}) ${name}  ${BLUE}(${models})${NC}"
    done
    
    echo ""
    echo -e "${YELLOW}Cloud Providers:${NC}"
    for idx in 11 12; do
        line=$((line + 1))
        if is_provider_selected "$idx"; then
            marker="${GREEN}[✓]${NC}"
        else
            marker="[ ]"
        fi
        if [[ "$line" -eq "$cursor_pos" ]]; then
            cursor="${CYAN}▶${NC} "
        else
            cursor="  "
        fi
        name=$(get_provider_field "$idx" name)
        models=$(get_provider_field "$idx" models)
        echo -e "${cursor}${marker} ${idx}) ${name}  ${BLUE}(${models})${NC}"
    done
    
    echo ""
    echo -e "${YELLOW}Custom:${NC}"
    for idx in 13; do
        line=$((line + 1))
        if is_provider_selected "$idx"; then
            marker="${GREEN}[✓]${NC}"
        else
            marker="[ ]"
        fi
        if [[ "$line" -eq "$cursor_pos" ]]; then
            cursor="${CYAN}▶${NC} "
        else
            cursor="  "
        fi
        name=$(get_provider_field "$idx" name)
        models=$(get_provider_field "$idx" models)
        echo -e "${cursor}${marker} ${idx}) ${name}  ${BLUE}(${models})${NC}"
    done
    
    echo ""
}

select_providers() {
    SELECTED_PROVIDERS="1"
    local cursor_pos=1
    local max_pos=13
    local key
    
    # Hide cursor and set up cleanup
    tput civis 2>/dev/null || true
    trap 'tput cnorm 2>/dev/null || true' EXIT
    
    while true; do
        print_provider_menu "$cursor_pos"
        
        # Read single keypress
        IFS= read -rsn1 key
        
        case "$key" in
            $'\x1b')  # Escape sequence (arrow keys)
                read -rsn2 -t 0.1 key
                case "$key" in
                    '[A')  # Up arrow
                        ((cursor_pos > 1)) && ((cursor_pos--))
                        ;;
                    '[B')  # Down arrow
                        ((cursor_pos < max_pos)) && ((cursor_pos++))
                        ;;
                esac
                ;;
            ' ')  # Space - toggle selection
                toggle_provider "$cursor_pos"
                ;;
            ''|$'\n')  # Enter - done
                if [[ -z "$SELECTED_PROVIDERS" ]]; then
                    echo -e "\n${RED}You must select at least one provider${NC}"
                    sleep 1
                    continue
                fi
                break
                ;;
            q|Q)  # Quit
                echo -e "\n${RED}Installation cancelled${NC}"
                tput cnorm 2>/dev/null || true
                exit 1
                ;;
        esac
    done
    
    # Show cursor again
    tput cnorm 2>/dev/null || true
    
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
                configure_provider_advanced "$pid" "$pname"
                echo ""
                ;;
            
            oauth)
                info "$pname uses OAuth - no API key needed."
                info "You'll authenticate via browser when first connecting."
                echo ""
                ;;
            
            local)
                configure_ollama
                configure_provider_advanced "$pid" "$pname"
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

configure_provider_advanced() {
    local provider_id="$1"
    local provider_name="$2"
    
    echo ""
    if ! prompt_yn "Configure advanced options for $provider_name? (custom model/endpoint)" "n"; then
        return
    fi
    
    echo ""
    echo -e "${BOLD}Advanced Configuration for $provider_name${NC}"
    echo ""
    
    case "$provider_id" in
        anthropic|openai|google|moonshot|zhipu|minimax|xai|ollama)
            local default_model
            case "$provider_id" in
                anthropic)  default_model="claude-sonnet-4-6" ;;
                openai)     default_model="gpt-5.4" ;;
                google)     default_model="gemini-2.0-flash" ;;
                moonshot)   default_model="kimi-k2.5" ;;
                zhipu)      default_model="glm-4-flash" ;;
                minimax)    default_model="abab6.5s-chat" ;;
                xai)        default_model="grok-2" ;;
                ollama)     default_model="${OLLAMA_MODEL:-llama3.1}" ;;
            esac
            
            if prompt_yn "Use a custom model ID? (default: $default_model)" "n"; then
                local custom_model
                prompt custom_model "Enter model ID" "$default_model"
                case "$provider_id" in
                    anthropic)  ANTHROPIC_CUSTOM_MODEL="$custom_model" ;;
                    openai)     OPENAI_CUSTOM_MODEL="$custom_model" ;;
                    google)     GOOGLE_CUSTOM_MODEL="$custom_model" ;;
                    moonshot)   MOONSHOT_CUSTOM_MODEL="$custom_model" ;;
                    zhipu)      ZHIPU_CUSTOM_MODEL="$custom_model" ;;
                    minimax)    MINIMAX_CUSTOM_MODEL="$custom_model" ;;
                    xai)        XAI_CUSTOM_MODEL="$custom_model" ;;
                    ollama)     OLLAMA_CUSTOM_MODEL="$custom_model" ;;
                esac
            fi
            ;;
    esac
    
    case "$provider_id" in
        anthropic|openai|moonshot|zhipu|minimax|xai|ollama)
            local default_endpoint
            case "$provider_id" in
                anthropic)  default_endpoint="https://api.anthropic.com" ;;
                openai)     default_endpoint="https://api.openai.com/v1" ;;
                moonshot)   default_endpoint="https://api.moonshot.cn/v1" ;;
                zhipu)      default_endpoint="https://open.bigmodel.cn/api/paas/v4" ;;
                minimax)    default_endpoint="https://api.minimax.chat/v1" ;;
                xai)        default_endpoint="https://api.x.ai/v1" ;;
                ollama)     default_endpoint="http://localhost:11434" ;;
            esac
            
            if prompt_yn "Use a custom API endpoint? (default: $default_endpoint)" "n"; then
                local custom_url
                prompt custom_url "Enter API base URL" "$default_endpoint"
                case "$provider_id" in
                    anthropic)  ANTHROPIC_BASE_URL="$custom_url" ;;
                    openai)     OPENAI_BASE_URL="$custom_url" ;;
                    moonshot)   MOONSHOT_BASE_URL="$custom_url" ;;
                    zhipu)      ZHIPU_BASE_URL="$custom_url" ;;
                    minimax)    MINIMAX_BASE_URL="$custom_url" ;;
                    xai)        XAI_BASE_URL="$custom_url" ;;
                    ollama)     OLLAMA_BASE_URL="$custom_url" ;;
                esac
            fi
            ;;
    esac
    
    echo ""
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
        
        configure_bedrock_models
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

configure_bedrock_models() {
    echo ""
    echo -e "${BOLD}=== AWS Bedrock Model Selection ===${NC}"
    echo ""
    echo "Select your primary Claude model for Bedrock:"
    echo ""
    echo -e "${YELLOW}Claude Sonnet 4.6 (Recommended for coding):${NC}"
    echo "   1) anthropic.claude-sonnet-4-6              (Default - single region)"
    echo "   2) us.anthropic.claude-sonnet-4-6           (US cross-region inference)"
    echo "   3) eu.anthropic.claude-sonnet-4-6           (EU cross-region inference)"
    echo "   4) global.anthropic.claude-sonnet-4-6       (Global cross-region inference)"
    echo ""
    echo -e "${YELLOW}Claude Opus 4.6 (Most capable):${NC}"
    echo "   5) anthropic.claude-opus-4-6-v1             (Default)"
    echo "   6) us.anthropic.claude-opus-4-6-v1          (US cross-region)"
    echo "   7) eu.anthropic.claude-opus-4-6-v1          (EU cross-region)"
    echo "   8) global.anthropic.claude-opus-4-6-v1      (Global cross-region)"
    echo ""
    echo -e "${YELLOW}Claude Haiku 4.5 (Fast and economical):${NC}"
    echo "   9) anthropic.claude-haiku-4-5-20251001-v1:0 (Default)"
    echo "  10) us.anthropic.claude-haiku-4-5-20251001-v1:0 (US cross-region)"
    echo "  11) global.anthropic.claude-haiku-4-5-20251001-v1:0 (Global)"
    echo ""
    echo -e "${YELLOW}Claude Opus 4.5:${NC}"
    echo "  12) anthropic.claude-opus-4-5-20251101-v1:0  (Default)"
    echo "  13) us.anthropic.claude-opus-4-5-20251101-v1:0 (US cross-region)"
    echo "  14) global.anthropic.claude-opus-4-5-20251101-v1:0 (Global)"
    echo ""
    echo -e "${YELLOW}Claude Sonnet 4.5:${NC}"
    echo "  15) anthropic.claude-sonnet-4-5-20250929-v1:0 (Default)"
    echo "  16) global.anthropic.claude-sonnet-4-5-20250929-v1:0 (Global)"
    echo ""
    echo -e "${CYAN}Custom:${NC}"
    echo "  17) Enter a custom model ID"
    echo ""
    
    local model_choice
    prompt model_choice "Select primary model [1-17]" "1"
    
    case "$model_choice" in
        1)  BEDROCK_PRIMARY_MODEL="anthropic.claude-sonnet-4-6" ;;
        2)  BEDROCK_PRIMARY_MODEL="us.anthropic.claude-sonnet-4-6" ;;
        3)  BEDROCK_PRIMARY_MODEL="eu.anthropic.claude-sonnet-4-6" ;;
        4)  BEDROCK_PRIMARY_MODEL="global.anthropic.claude-sonnet-4-6" ;;
        5)  BEDROCK_PRIMARY_MODEL="anthropic.claude-opus-4-6-v1" ;;
        6)  BEDROCK_PRIMARY_MODEL="us.anthropic.claude-opus-4-6-v1" ;;
        7)  BEDROCK_PRIMARY_MODEL="eu.anthropic.claude-opus-4-6-v1" ;;
        8)  BEDROCK_PRIMARY_MODEL="global.anthropic.claude-opus-4-6-v1" ;;
        9)  BEDROCK_PRIMARY_MODEL="anthropic.claude-haiku-4-5-20251001-v1:0" ;;
        10) BEDROCK_PRIMARY_MODEL="us.anthropic.claude-haiku-4-5-20251001-v1:0" ;;
        11) BEDROCK_PRIMARY_MODEL="global.anthropic.claude-haiku-4-5-20251001-v1:0" ;;
        12) BEDROCK_PRIMARY_MODEL="anthropic.claude-opus-4-5-20251101-v1:0" ;;
        13) BEDROCK_PRIMARY_MODEL="us.anthropic.claude-opus-4-5-20251101-v1:0" ;;
        14) BEDROCK_PRIMARY_MODEL="global.anthropic.claude-opus-4-5-20251101-v1:0" ;;
        15) BEDROCK_PRIMARY_MODEL="anthropic.claude-sonnet-4-5-20250929-v1:0" ;;
        16) BEDROCK_PRIMARY_MODEL="global.anthropic.claude-sonnet-4-5-20250929-v1:0" ;;
        17) prompt BEDROCK_PRIMARY_MODEL "Enter custom primary model ID" "anthropic.claude-sonnet-4-6" ;;
        *)  BEDROCK_PRIMARY_MODEL="anthropic.claude-sonnet-4-6" ;;
    esac
    
    success "Primary model: $BEDROCK_PRIMARY_MODEL"
    
    local suggested_fast
    if [[ "$BEDROCK_PRIMARY_MODEL" == us.* ]]; then
        suggested_fast="us.anthropic.claude-haiku-4-5-20251001-v1:0"
    elif [[ "$BEDROCK_PRIMARY_MODEL" == eu.* ]]; then
        suggested_fast="anthropic.claude-haiku-4-5-20251001-v1:0"
    elif [[ "$BEDROCK_PRIMARY_MODEL" == global.* ]]; then
        suggested_fast="global.anthropic.claude-haiku-4-5-20251001-v1:0"
    else
        suggested_fast="anthropic.claude-haiku-4-5-20251001-v1:0"
    fi
    
    echo ""
    echo "Select fast/economical model (for quick tasks):"
    echo "  Suggested: $suggested_fast"
    echo ""
    echo "  1) Use suggested: $suggested_fast"
    echo "  2) anthropic.claude-haiku-4-5-20251001-v1:0 (Default Haiku)"
    echo "  3) us.anthropic.claude-haiku-4-5-20251001-v1:0 (US Haiku)"
    echo "  4) global.anthropic.claude-haiku-4-5-20251001-v1:0 (Global Haiku)"
    echo "  5) Same as primary model"
    echo "  6) Enter custom fast model ID"
    echo ""
    
    local fast_choice
    prompt fast_choice "Select fast model [1-6]" "1"
    
    case "$fast_choice" in
        1)  BEDROCK_FAST_MODEL="$suggested_fast" ;;
        2)  BEDROCK_FAST_MODEL="anthropic.claude-haiku-4-5-20251001-v1:0" ;;
        3)  BEDROCK_FAST_MODEL="us.anthropic.claude-haiku-4-5-20251001-v1:0" ;;
        4)  BEDROCK_FAST_MODEL="global.anthropic.claude-haiku-4-5-20251001-v1:0" ;;
        5)  BEDROCK_FAST_MODEL="$BEDROCK_PRIMARY_MODEL" ;;
        6)  prompt BEDROCK_FAST_MODEL "Enter custom fast model ID" "$suggested_fast" ;;
        *)  BEDROCK_FAST_MODEL="$suggested_fast" ;;
    esac
    
    success "Fast model: $BEDROCK_FAST_MODEL"
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
    local docker_missing=false
    local docker_not_running=false
    local build_tools_missing=false
    local nodejs_missing=false
    
    for cmd in curl git; do
        if command -v "$cmd" &> /dev/null; then
            success "$cmd is installed"
        else
            missing+=("$cmd")
            error "$cmd is not installed"
        fi
    done
    
    if command -v cc &> /dev/null || command -v gcc &> /dev/null; then
        success "C compiler is installed"
    else
        error "C compiler (cc/gcc) is not installed"
        build_tools_missing=true
    fi
    
    if command -v docker &> /dev/null; then
        success "docker is installed"
        if docker info &> /dev/null; then
            success "Docker daemon is running"
        else
            error "Docker is installed but not running"
            docker_not_running=true
        fi
    else
        error "docker is not installed"
        docker_missing=true
    fi
    
    if command -v node &> /dev/null; then
        success "Node.js is installed ($(node --version))"
        if command -v pnpm &> /dev/null; then
            success "pnpm is installed"
        elif command -v npm &> /dev/null; then
            success "npm is installed (pnpm recommended)"
        fi
    else
        warn "Node.js is not installed (required for WebUI)"
        nodejs_missing=true
    fi
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        echo ""
        error "Missing dependencies: ${missing[*]}"
        echo ""
        info "Installing missing dependencies..."
        install_dependencies "${missing[@]}"
    fi
    
    if [[ "$build_tools_missing" == true ]]; then
        echo ""
        info "Installing build tools (required for Rust compilation)..."
        case "$PLATFORM" in
            linux|wsl)
                install_dependencies_linux "build-essential"
                ;;
            macos)
                install_dependencies_macos "build-essential"
                ;;
        esac
    fi
    
    if [[ "$nodejs_missing" == true ]]; then
        echo ""
        if prompt_yn "Install Node.js and pnpm for WebUI support?" "y"; then
            install_nodejs
        else
            warn "Skipping Node.js - WebUI will not be available"
        fi
    fi
    
    if [[ "$docker_missing" == true || "$docker_not_running" == true ]]; then
        echo ""
        case "$PLATFORM" in
            macos)
                if [[ "$docker_missing" == true ]]; then
                    warn "Docker is not installed on macOS"
                    info "Installing Docker..."
                    install_dependencies_macos "docker"
                else
                    warn "Docker is installed but not running"
                    info "Starting Docker..."
                    install_dependencies_macos "docker-running"
                fi
                ;;
            wsl)
                if [[ "$docker_missing" == true ]]; then
                    warn "Docker is not available in WSL"
                    info "Setting up Docker..."
                    install_dependencies_wsl "docker"
                else
                    warn "Docker is installed but not running"
                    info "Starting Docker..."
                    install_dependencies_wsl "docker-running"
                fi
                ;;
            linux)
                if [[ "$docker_missing" == true ]]; then
                    info "Installing Docker..."
                    install_dependencies_linux "docker"
                else
                    info "Starting Docker..."
                    install_dependencies_linux "docker-running"
                fi
                ;;
        esac
        
        if ! docker info &>/dev/null; then
            error "Docker is still not accessible. Please check your Docker installation."
            exit 1
        fi
        success "Docker is now running"
    fi
}

install_nodejs() {
    info "Installing Node.js..."
    
    case "$PLATFORM" in
        linux|wsl)
            if command -v apt-get &>/dev/null; then
                curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
                apt-get install -y nodejs
            elif command -v dnf &>/dev/null; then
                curl -fsSL https://rpm.nodesource.com/setup_22.x | bash -
                dnf install -y nodejs
            elif command -v pacman &>/dev/null; then
                pacman -S --noconfirm nodejs npm
            else
                warn "Could not auto-install Node.js. Please install manually."
                return 1
            fi
            ;;
        macos)
            if command -v brew &>/dev/null; then
                brew install node
            else
                warn "Homebrew not found. Please install Node.js manually."
                return 1
            fi
            ;;
    esac
    
    if command -v node &>/dev/null; then
        success "Node.js installed ($(node --version))"
        
        info "Installing pnpm..."
        npm install -g pnpm
        if command -v pnpm &>/dev/null; then
            success "pnpm installed"
        fi
    else
        warn "Node.js installation may have failed"
        return 1
    fi
}

install_dependencies() {
    local deps=("$@")
    
    case "$PLATFORM" in
        linux)
            install_dependencies_linux "${deps[@]}"
            ;;
        macos)
            install_dependencies_macos "${deps[@]}"
            ;;
        wsl)
            install_dependencies_wsl "${deps[@]}"
            ;;
        *)
            error "Unknown platform: $PLATFORM"
            exit 1
            ;;
    esac
}

install_dependencies_linux() {
    local deps=("$@")
    
    # Detect package manager from DISTRO or available commands
    local pkg_install pkg_update
    case "$DISTRO" in
        ubuntu|debian|linuxmint|pop)
            pkg_update="apt-get update"
            pkg_install="apt-get install -y"
            ;;
        fedora|rhel|centos|rocky|almalinux)
            pkg_update="dnf check-update || true"
            pkg_install="dnf install -y"
            ;;
        arch|manjaro|endeavouros)
            pkg_update="pacman -Sy --noconfirm"
            pkg_install="pacman -S --noconfirm"
            ;;
        *)
            # Fallback: try to detect from available commands
            if command -v apt-get &>/dev/null; then
                pkg_update="apt-get update"
                pkg_install="apt-get install -y"
            elif command -v dnf &>/dev/null; then
                pkg_update="dnf check-update || true"
                pkg_install="dnf install -y"
            elif command -v pacman &>/dev/null; then
                pkg_update="pacman -Sy --noconfirm"
                pkg_install="pacman -S --noconfirm"
            else
                error "No supported package manager found (apt-get, dnf, pacman)"
                exit 1
            fi
            ;;
    esac
    
    info "Updating package lists..."
    eval "$pkg_update"
    
    for dep in "${deps[@]}"; do
        case "$dep" in
            curl|git)
                info "Installing $dep..."
                eval "$pkg_install $dep"
                ;;
            build-essential)
                info "Installing build tools..."
                case "$DISTRO" in
                    ubuntu|debian|linuxmint|pop)
                        eval "$pkg_install build-essential pkg-config libssl-dev"
                        ;;
                    fedora|rhel|centos|rocky|almalinux)
                        eval "$pkg_install gcc make openssl-devel pkgconfig"
                        ;;
                    arch|manjaro|endeavouros)
                        eval "$pkg_install base-devel openssl"
                        ;;
                    *)
                        # Fallback based on package manager
                        if command -v apt-get &>/dev/null; then
                            eval "$pkg_install build-essential pkg-config libssl-dev"
                        elif command -v dnf &>/dev/null; then
                            eval "$pkg_install gcc make openssl-devel pkgconfig"
                        elif command -v pacman &>/dev/null; then
                            eval "$pkg_install base-devel openssl"
                        else
                            warn "Unknown distro, skipping build tools"
                        fi
                        ;;
                esac
                ;;
            docker)
                info "Installing Docker..."
                if command -v apt-get &>/dev/null; then
                    # Use official Docker install script for Debian/Ubuntu
                    curl -fsSL https://get.docker.com | sh
                else
                    eval "$pkg_install docker"
                fi
                if [[ "$HAS_SYSTEMD" == true ]]; then
                    systemctl enable docker
                    systemctl start docker
                else
                    service docker start 2>/dev/null || true
                fi
                usermod -aG docker "$SERVICE_USER" 2>/dev/null || true
                ;;
            docker-running)
                info "Starting Docker daemon..."
                if [[ "$HAS_SYSTEMD" == true ]]; then
                    systemctl start docker
                else
                    service docker start 2>/dev/null || true
                fi
                ;;
        esac
    done
}

install_dependencies_macos() {
    local deps=("$@")
    
    # Ensure Homebrew is installed
    if ! command -v brew &>/dev/null; then
        info "Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        # Add brew to PATH for Apple Silicon
        if [[ -f /opt/homebrew/bin/brew ]]; then
            eval "$(/opt/homebrew/bin/brew shellenv)"
        fi
    fi
    
    for dep in "${deps[@]}"; do
        case "$dep" in
            curl|git)
                if ! command -v "$dep" &>/dev/null; then
                    info "Installing $dep via Homebrew..."
                    brew install "$dep"
                fi
                ;;
            build-essential)
                info "Installing Xcode Command Line Tools..."
                xcode-select --install 2>/dev/null || true
                brew install openssl pkg-config
                ;;
            docker|docker-running)
                if ! command -v docker &>/dev/null; then
                    echo ""
                    echo "Docker is not installed. Choose an option:"
                    echo "  1) Docker Desktop (GUI, requires license for commercial use)"
                    echo "  2) Colima (CLI, free and open source - recommended)"
                    echo ""
                    local docker_choice
                    prompt docker_choice "Select Docker option [1-2]" "2"
                    
                    case "$docker_choice" in
                        1)
                            info "Installing Docker Desktop..."
                            brew install --cask docker
                            echo ""
                            warn "Please launch Docker Desktop from Applications and wait for it to start."
                            echo -en "${CYAN}Press Enter when Docker Desktop is running...${NC}"
                            read -r
                            ;;
                        *)
                            info "Installing Colima + Docker CLI..."
                            brew install colima docker
                            info "Starting Colima..."
                            colima start
                            ;;
                    esac
                elif [[ "$dep" == "docker-running" ]]; then
                    # Docker is installed but not running
                    if command -v colima &>/dev/null; then
                        info "Starting Colima..."
                        colima start
                    else
                        echo ""
                        warn "Docker is installed but not running."
                        echo "Please start Docker Desktop from Applications."
                        echo -en "${CYAN}Press Enter when Docker is running...${NC}"
                        read -r
                    fi
                fi
                ;;
        esac
    done
}

install_dependencies_wsl() {
    local deps=("$@")
    
    # WSL uses Linux package managers but Docker needs special handling
    # First handle non-Docker deps using Linux installer
    local non_docker_deps=()
    local needs_docker=false
    
    for dep in "${deps[@]}"; do
        if [[ "$dep" == "docker" || "$dep" == "docker-running" ]]; then
            needs_docker=true
        else
            non_docker_deps+=("$dep")
        fi
    done
    
    if [[ ${#non_docker_deps[@]} -gt 0 ]]; then
        install_dependencies_linux "${non_docker_deps[@]}"
    fi
    
    if [[ "$needs_docker" == true ]] && ! command -v docker &>/dev/null; then
        echo ""
        echo "Docker is not found in WSL. Choose an option:"
        echo "  1) Use Docker Desktop for Windows (recommended)"
        echo "     - Install Docker Desktop on Windows"
        echo "     - Enable WSL integration in Docker Desktop settings"
        echo "     - Docker will be available in WSL automatically"
        echo "  2) Install Docker directly in WSL"
        echo "     - Standalone Docker daemon in WSL"
        echo "     - May have limitations with systemd"
        echo ""
        local wsl_docker_choice
        prompt wsl_docker_choice "Select Docker option [1-2]" "1"
        
        case "$wsl_docker_choice" in
            2)
                info "Installing Docker in WSL..."
                curl -fsSL https://get.docker.com | sh
                if [[ "$HAS_SYSTEMD" == true ]]; then
                    systemctl enable docker
                    systemctl start docker
                else
                    service docker start 2>/dev/null || true
                    warn "systemd not available. Start Docker manually: sudo service docker start"
                fi
                usermod -aG docker "$SERVICE_USER" 2>/dev/null || true
                ;;
            *)
                echo ""
                warn "Please install Docker Desktop for Windows and enable WSL integration."
                echo "  1. Download: https://www.docker.com/products/docker-desktop/"
                echo "  2. Install and launch Docker Desktop"
                echo "  3. Go to Settings → Resources → WSL Integration"
                echo "  4. Enable integration for your WSL distro"
                echo ""
                echo -en "${CYAN}Press Enter when Docker Desktop is configured...${NC}"
                read -r
                ;;
        esac
    elif [[ "$needs_docker" == true ]]; then
        # Docker command exists but may not be running
        if ! docker info &>/dev/null; then
            echo ""
            warn "Docker is installed but not running."
            if [[ "$HAS_SYSTEMD" == true ]]; then
                info "Starting Docker..."
                systemctl start docker
            else
                echo "If using Docker Desktop for Windows, please start it."
                echo "If using native Docker, run: sudo service docker start"
                echo ""
                echo -en "${CYAN}Press Enter when Docker is running...${NC}"
                read -r
            fi
        fi
    fi
}

install_rust() {
    # Source cargo env if it exists (in case rust is installed but not in PATH)
    [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"
    
    if ! command -v rustc &> /dev/null && [[ ! -x "$HOME/.cargo/bin/rustc" ]]; then
        info "Installing Rust ${RUST_VERSION}..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain "$RUST_VERSION"
        source "$HOME/.cargo/env"
        success "Rust installed"
    else
        local current_ver
        current_ver=$("$HOME/.cargo/bin/rustc" --version 2>/dev/null || rustc --version | awk '{print $2}')
        if [[ "$current_ver" != "$RUST_VERSION"* ]]; then
            info "Updating Rust to ${RUST_VERSION}..."
            "$HOME/.cargo/bin/rustup" install "$RUST_VERSION" 2>/dev/null || rustup install "$RUST_VERSION"
            "$HOME/.cargo/bin/rustup" default "$RUST_VERSION" 2>/dev/null || rustup default "$RUST_VERSION"
        fi
        success "Rust $current_ver is ready"
    fi
}

#######################################
# Configuration
#######################################

configure_server() {
    echo ""
    echo -e "${BOLD}=== Server Configuration ===${NC}"
    echo ""
    
    prompt SERVER_HOST "Server bind address" "${SERVER_HOST:-0.0.0.0}"
    prompt SERVER_PORT "Server port" "${SERVER_PORT:-8080}"
    
    if [[ -n "${API_KEY:-}" ]]; then
        if prompt_yn "Keep existing API key?" "y"; then
            success "Keeping existing API key: ${API_KEY:0:8}...${API_KEY: -8}"
        elif prompt_yn "Generate a new random API key?" "y"; then
            API_KEY=$(openssl rand -hex 32)
            success "Generated API key: ${API_KEY:0:8}...${API_KEY: -8}"
        else
            prompt_secret API_KEY "Enter API key"
        fi
    elif prompt_yn "Generate a random API key?" "y"; then
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
    
    prompt DB_PATH "Database file path" "${DB_PATH:-${DATA_DIR}/cuttlefish.db}"
}

configure_sandbox() {
    echo ""
    echo -e "${BOLD}=== Docker Sandbox Configuration ===${NC}"
    echo ""
    
    prompt DOCKER_SOCKET "Docker socket path" "${DOCKER_SOCKET:-unix:///var/run/docker.sock}"
    prompt MEMORY_LIMIT "Memory limit per sandbox (MB)" "${MEMORY_LIMIT:-2048}"
    prompt CPU_LIMIT "CPU limit per sandbox" "${CPU_LIMIT:-2.0}"
    prompt DISK_LIMIT "Disk limit per sandbox (GB)" "${DISK_LIMIT:-10}"
    prompt MAX_CONCURRENT "Maximum concurrent sandboxes" "${MAX_CONCURRENT:-5}"
}

configure_discord() {
    echo ""
    echo -e "${BOLD}=== Discord Bot Configuration ===${NC}"
    echo ""
    
    local discord_default="n"
    [[ "$ENABLE_DISCORD" == true ]] && discord_default="y"
    
    if prompt_yn "Enable Discord bot?" "$discord_default"; then
        ENABLE_DISCORD=true
        
        if [[ -n "${DISCORD_TOKEN:-}" ]]; then
            if prompt_yn "Keep existing Discord token?" "y"; then
                success "Keeping existing Discord token"
            else
                echo ""
                echo "To get a Discord bot token:"
                echo "  1. Go to https://discord.com/developers/applications"
                echo "  2. Create a new application"
                echo "  3. Go to Bot section, create bot, copy token"
                echo ""
                prompt_secret DISCORD_TOKEN "Discord bot token"
            fi
        else
            echo ""
            echo "To get a Discord bot token:"
            echo "  1. Go to https://discord.com/developers/applications"
            echo "  2. Create a new application"
            echo "  3. Go to Bot section, create bot, copy token"
            echo ""
            prompt_secret DISCORD_TOKEN "Discord bot token"
        fi
        prompt DISCORD_GUILD_IDS "Guild IDs (comma-separated, or empty for global)" "${DISCORD_GUILD_IDS:-}"
    else
        ENABLE_DISCORD=false
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

stop_running_service() {
    if [[ "$HAS_SYSTEMD" == true ]] && systemctl is-active --quiet cuttlefish 2>/dev/null; then
        info "Stopping running Cuttlefish service..."
        systemctl stop cuttlefish
        sleep 1
        success "Service stopped"
        return 0
    fi

    # Find cuttlefish-rs binary by looking at actual binary path, not command line
    # This avoids killing bash scripts that contain "cuttlefish-rs" in their content
    local my_pid=$$
    local my_ppid=$PPID
    local pids_to_kill=()

    while IFS= read -r pid; do
        [[ -z "$pid" ]] && continue
        # Skip our own process tree
        [[ "$pid" == "$my_pid" ]] && continue
        [[ "$pid" == "$my_ppid" ]] && continue

        # Check if this is actually the cuttlefish-rs binary (not a bash script)
        local exe_path
        exe_path=$(readlink -f "/proc/$pid/exe" 2>/dev/null || true)
        if [[ "$exe_path" == *"/cuttlefish-rs" ]]; then
            pids_to_kill+=("$pid")
        fi
    done < <(pgrep -f "cuttlefish-rs" 2>/dev/null || true)

    if [[ ${#pids_to_kill[@]} -gt 0 ]]; then
        info "Stopping running Cuttlefish process (PIDs: ${pids_to_kill[*]})..."
        for pid in "${pids_to_kill[@]}"; do
            kill "$pid" 2>/dev/null || true
        done
        sleep 1
        success "Process stopped"
    fi

    return 0
}

get_latest_release() {
    # First try to get the 'latest' pre-release (built on every push)
    local response
    response=$(curl -sS "https://api.github.com/repos/JackTYM/cuttlefish-rs/releases/tags/latest" 2>&1)
    
    if echo "$response" | grep -q '"tag_name"'; then
        echo "latest"
        return 0
    fi
    
    # Fall back to actual latest release
    response=$(curl -sS "https://api.github.com/repos/JackTYM/cuttlefish-rs/releases/latest" 2>&1) || {
        warn "Failed to fetch releases from GitHub API"
        return 1
    }
    
    local tag
    tag=$(echo "$response" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [[ -z "$tag" ]]; then
        warn "No releases found (repository may not have any releases yet)"
        return 1
    fi
    
    echo "$tag"
}

detect_arch() {
    local arch
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

download_binary() {
    echo ""
    echo -e "${BOLD}=== Installing Cuttlefish ===${NC}"
    echo ""
    
    stop_running_service
    
    local version arch os_name download_url tarball
    
    if ! version=$(get_latest_release); then
        info "Falling back to build from source..."
        build_from_source
        return
    fi
    
    if [[ -z "$version" ]]; then
        warn "Could not fetch latest release, falling back to build from source"
        build_from_source
        return
    fi
    
    arch=$(detect_arch)
    
    case "$PLATFORM" in
        linux|wsl)
            os_name="linux"
            ;;
        macos)
            os_name="darwin"
            ;;
        *)
            warn "Unknown platform, falling back to build from source"
            build_from_source
            return
            ;;
    esac
    
    tarball="cuttlefish-${os_name}-${arch}.tar.gz"
    download_url="https://github.com/JackTYM/cuttlefish-rs/releases/download/${version}/${tarball}"
    
    info "Downloading Cuttlefish $version for ${os_name}-${arch}..."
    
    if ! curl -fSL "$download_url" -o "/tmp/$tarball" 2>/dev/null; then
        warn "Binary not available for this platform, falling back to build from source"
        build_from_source
        return
    fi
    
    info "Extracting binary..."
    tar -xzf "/tmp/$tarball" -C /tmp
    
    mv /tmp/cuttlefish-rs "$INSTALL_DIR/"
    chown "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/cuttlefish-rs"
    chmod +x "$INSTALL_DIR/cuttlefish-rs"
    
    rm -f "/tmp/$tarball"
    
    success "Installed Cuttlefish $version to $INSTALL_DIR"
}

build_from_source() {
    echo ""
    echo -e "${BOLD}=== Building Cuttlefish from Source ===${NC}"
    echo ""
    
    local script_dir build_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    if [[ -f "$script_dir/Cargo.toml" ]]; then
        info "Building from source directory: $script_dir"
        build_dir="$script_dir"
    else
        build_dir="/tmp/cuttlefish-build"
        
        if [[ -d "$build_dir" ]]; then
            warn "Previous build directory exists: $build_dir"
            if prompt_yn "Remove it and clone fresh?" "y"; then
                rm -rf "$build_dir"
            else
                info "Using existing directory..."
                if [[ -d "$build_dir/.git" ]]; then
                    info "Pulling latest changes..."
                    cd "$build_dir"
                    git fetch origin
                    git reset --hard origin/master
                    cd - > /dev/null
                fi
            fi
        fi
        
        if [[ ! -d "$build_dir" ]]; then
            info "Cloning repository..."
            if ! git clone "https://github.com/JackTYM/cuttlefish-rs.git" "$build_dir"; then
                error "Failed to clone repository"
                error "Check your network connection and try again"
                exit 1
            fi
        fi
    fi
    
    cd "$build_dir"
    
    [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"
    
    install_rust
    
    info "Building release binary (this may take several minutes)..."
    local cargo_cmd="${HOME}/.cargo/bin/cargo"
    [[ ! -x "$cargo_cmd" ]] && cargo_cmd="cargo"
    
    if ! "$cargo_cmd" build --release; then
        error "Build failed! See errors above."
        error "Common fixes:"
        error "  - Ensure you have build-essential/gcc installed"
        error "  - Ensure you have at least 4GB free RAM"
        error "  - Check disk space (build needs ~2GB)"
        exit 1
    fi
    
    cp target/release/cuttlefish-rs "$INSTALL_DIR/"
    chown "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/cuttlefish-rs"
    chmod +x "$INSTALL_DIR/cuttlefish-rs"
    
    if [[ -f target/release/cuttlefish-tui ]]; then
        cp target/release/cuttlefish-tui "$INSTALL_DIR/"
        chown "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/cuttlefish-tui"
        chmod +x "$INSTALL_DIR/cuttlefish-tui"
    fi
    
    # Build and install WebUI if Node.js/pnpm is available
    if [[ -d "cuttlefish-web" ]]; then
        info "Building WebUI..."
        if command -v pnpm &> /dev/null; then
            cd cuttlefish-web
            NUXT_TELEMETRY_DISABLED=1 pnpm install --no-frozen-lockfile
            NUXT_TELEMETRY_DISABLED=1 pnpm generate
            cd ..
            
            if [[ -d "cuttlefish-web/.output/public" ]]; then
                mkdir -p "$INSTALL_DIR/webui"
                cp -r cuttlefish-web/.output/public/* "$INSTALL_DIR/webui/"
                chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/webui"
                success "WebUI built and installed to $INSTALL_DIR/webui"
            else
                warn "WebUI build did not produce expected output"
            fi
        elif command -v npm &> /dev/null; then
            cd cuttlefish-web
            NUXT_TELEMETRY_DISABLED=1 npm install
            NUXT_TELEMETRY_DISABLED=1 npm run generate
            cd ..
            
            if [[ -d "cuttlefish-web/.output/public" ]]; then
                mkdir -p "$INSTALL_DIR/webui"
                cp -r cuttlefish-web/.output/public/* "$INSTALL_DIR/webui/"
                chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/webui"
                success "WebUI built and installed to $INSTALL_DIR/webui"
            else
                warn "WebUI build did not produce expected output"
            fi
        else
            warn "Neither pnpm nor npm found - skipping WebUI build"
            info "Install Node.js and pnpm to enable WebUI: https://nodejs.org"
        fi
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

[webui]
enabled = true
static_dir = "$INSTALL_DIR/webui"
EOF

    local first_deep_provider=""
    local first_quick_provider=""
    
    for idx in $SELECTED_PROVIDERS; do
        local pid
        pid=$(get_provider_field "$idx" id)
        
        case "$pid" in
            anthropic)
                local anthropic_model="${ANTHROPIC_CUSTOM_MODEL:-claude-sonnet-4-6}"
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.anthropic]
provider_type = "anthropic"
model = "$anthropic_model"
api_key_env = "ANTHROPIC_API_KEY"

[providers.anthropic-fast]
provider_type = "anthropic"
model = "claude-haiku-4-5"
api_key_env = "ANTHROPIC_API_KEY"
EOF
                if [[ -n "${ANTHROPIC_BASE_URL:-}" ]]; then
                    sed -i '/^\[providers\.anthropic\]$/,/^\[/{s/^api_key_env = "ANTHROPIC_API_KEY"$/api_key_env = "ANTHROPIC_API_KEY"\nbase_url = "'"$ANTHROPIC_BASE_URL"'"/}' "$CONFIG_DIR/cuttlefish.toml"
                    sed -i '/^\[providers\.anthropic-fast\]$/,/^\[/{s/^api_key_env = "ANTHROPIC_API_KEY"$/api_key_env = "ANTHROPIC_API_KEY"\nbase_url = "'"$ANTHROPIC_BASE_URL"'"/}' "$CONFIG_DIR/cuttlefish.toml"
                fi
                [[ -z "$first_deep_provider" ]] && first_deep_provider="anthropic"
                [[ -z "$first_quick_provider" ]] && first_quick_provider="anthropic-fast"
                ;;
                
            openai)
                local openai_model="${OPENAI_CUSTOM_MODEL:-gpt-5.4}"
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.openai]
provider_type = "openai"
model = "$openai_model"
api_key_env = "OPENAI_API_KEY"

[providers.openai-fast]
provider_type = "openai"
model = "gpt-5-nano"
api_key_env = "OPENAI_API_KEY"
EOF
                if [[ -n "${OPENAI_BASE_URL:-}" ]]; then
                    sed -i '/^\[providers\.openai\]$/,/^\[/{s/^api_key_env = "OPENAI_API_KEY"$/api_key_env = "OPENAI_API_KEY"\nbase_url = "'"$OPENAI_BASE_URL"'"/}' "$CONFIG_DIR/cuttlefish.toml"
                    sed -i '/^\[providers\.openai-fast\]$/,/^\[/{s/^api_key_env = "OPENAI_API_KEY"$/api_key_env = "OPENAI_API_KEY"\nbase_url = "'"$OPENAI_BASE_URL"'"/}' "$CONFIG_DIR/cuttlefish.toml"
                fi
                [[ -z "$first_deep_provider" ]] && first_deep_provider="openai"
                [[ -z "$first_quick_provider" ]] && first_quick_provider="openai-fast"
                ;;
                
            google)
                local google_model="${GOOGLE_CUSTOM_MODEL:-gemini-2.0-flash}"
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.google]
provider_type = "google"
model = "$google_model"
api_key_env = "GOOGLE_API_KEY"
EOF
                [[ -z "$first_quick_provider" ]] && first_quick_provider="google"
                ;;
                
            moonshot)
                local moonshot_model="${MOONSHOT_CUSTOM_MODEL:-kimi-k2.5}"
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.moonshot]
provider_type = "moonshot"
model = "$moonshot_model"
api_key_env = "MOONSHOT_API_KEY"
EOF
                if [[ -n "${MOONSHOT_BASE_URL:-}" ]]; then
                    sed -i '/^\[providers\.moonshot\]$/,/^\[/{s/^api_key_env = "MOONSHOT_API_KEY"$/api_key_env = "MOONSHOT_API_KEY"\nbase_url = "'"$MOONSHOT_BASE_URL"'"/}' "$CONFIG_DIR/cuttlefish.toml"
                fi
                [[ -z "$first_deep_provider" ]] && first_deep_provider="moonshot"
                ;;
                
            zhipu)
                local zhipu_model="${ZHIPU_CUSTOM_MODEL:-glm-4-flash}"
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.zhipu]
provider_type = "zhipu"
model = "$zhipu_model"
api_key_env = "ZHIPU_API_KEY"
EOF
                if [[ -n "${ZHIPU_BASE_URL:-}" ]]; then
                    sed -i '/^\[providers\.zhipu\]$/,/^\[/{s/^api_key_env = "ZHIPU_API_KEY"$/api_key_env = "ZHIPU_API_KEY"\nbase_url = "'"$ZHIPU_BASE_URL"'"/}' "$CONFIG_DIR/cuttlefish.toml"
                fi
                [[ -z "$first_quick_provider" ]] && first_quick_provider="zhipu"
                ;;
                
            minimax)
                local minimax_model="${MINIMAX_CUSTOM_MODEL:-abab6.5s-chat}"
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.minimax]
provider_type = "minimax"
model = "$minimax_model"
api_key_env = "MINIMAX_API_KEY"
EOF
                if [[ -n "${MINIMAX_BASE_URL:-}" ]]; then
                    sed -i '/^\[providers\.minimax\]$/,/^\[/{s/^api_key_env = "MINIMAX_API_KEY"$/api_key_env = "MINIMAX_API_KEY"\nbase_url = "'"$MINIMAX_BASE_URL"'"/}' "$CONFIG_DIR/cuttlefish.toml"
                fi
                [[ -z "$first_quick_provider" ]] && first_quick_provider="minimax"
                ;;
                
            xai)
                local xai_model="${XAI_CUSTOM_MODEL:-grok-2}"
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.xai]
provider_type = "xai"
model = "$xai_model"
api_key_env = "XAI_API_KEY"
EOF
                if [[ -n "${XAI_BASE_URL:-}" ]]; then
                    sed -i '/^\[providers\.xai\]$/,/^\[/{s/^api_key_env = "XAI_API_KEY"$/api_key_env = "XAI_API_KEY"\nbase_url = "'"$XAI_BASE_URL"'"/}' "$CONFIG_DIR/cuttlefish.toml"
                fi
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
                local ollama_model="${OLLAMA_CUSTOM_MODEL:-${OLLAMA_MODEL:-llama3.1}}"
                local ollama_url="${OLLAMA_BASE_URL:-http://localhost:11434}"
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.ollama]
provider_type = "ollama"
model = "$ollama_model"
base_url = "$ollama_url"
EOF
                [[ -z "$first_deep_provider" ]] && first_deep_provider="ollama"
                [[ -z "$first_quick_provider" ]] && first_quick_provider="ollama"
                ;;
                
            bedrock)
                cat >> "$CONFIG_DIR/cuttlefish.toml" << EOF

[providers.bedrock]
provider_type = "bedrock"
model = "${BEDROCK_PRIMARY_MODEL}"
region = "${AWS_REGION:-us-east-1}"

[providers.bedrock-fast]
provider_type = "bedrock"
model = "${BEDROCK_FAST_MODEL}"
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
Documentation=https://github.com/JackTYM/cuttlefish-rs
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

create_symlinks() {
    info "Creating command symlinks..."

    # Create symlink for 'cuttlefish' command
    ln -sf "$INSTALL_DIR/cuttlefish-rs" /usr/local/bin/cuttlefish

    # Also create cuttlefish-rs symlink for explicit invocation
    ln -sf "$INSTALL_DIR/cuttlefish-rs" /usr/local/bin/cuttlefish-rs

    # Create symlink for TUI if it exists
    if [[ -f "$INSTALL_DIR/cuttlefish-tui" ]]; then
        ln -sf "$INSTALL_DIR/cuttlefish-tui" /usr/local/bin/cuttlefish-tui
    fi

    success "Created symlinks in /usr/local/bin/"
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
    echo "  Command:       cuttlefish (symlinked to /usr/local/bin/)"
    echo "  Config:        $CONFIG_DIR/cuttlefish.toml"
    echo "  Environment:   $CONFIG_DIR/cuttlefish.env"
    echo "  Database:      $DB_PATH"
    echo "  Logs:          $LOG_DIR/"
    echo "  Mode:          $DEPLOYMENT_MODE"
    echo ""
    echo -e "${BOLD}Configured Providers:${NC}"
    for idx in $SELECTED_PROVIDERS; do
        local pname
        pname=$(get_provider_field "$idx" name)
        echo "  - $pname"
    done
    echo ""
    
    if [[ "$DEPLOYMENT_MODE" == "systemd" ]]; then
        echo -e "${BOLD}Service Commands:${NC}"
        echo "  Start:         sudo systemctl start cuttlefish"
        echo "  Stop:          sudo systemctl stop cuttlefish"
        echo "  Status:        sudo systemctl status cuttlefish"
        echo "  Logs:          sudo journalctl -u cuttlefish -f"
        echo "  Enable:        sudo systemctl enable cuttlefish"
    else
        echo -e "${BOLD}Docker Commands:${NC}"
        echo "  Run:"
        echo "    docker run -d --name cuttlefish \\"
        echo "      -v /var/run/docker.sock:/var/run/docker.sock \\"
        echo "      -v $CONFIG_DIR:/etc/cuttlefish:ro \\"
        echo "      -v $DATA_DIR:/var/lib/cuttlefish \\"
        echo "      -v $LOG_DIR:/var/log/cuttlefish \\"
        echo "      --env-file $CONFIG_DIR/cuttlefish.env \\"
        echo "      -p ${SERVER_PORT}:${SERVER_PORT} \\"
        echo "      cuttlefish:latest"
        echo ""
        echo "  Stop:          docker stop cuttlefish"
        echo "  Logs:          docker logs -f cuttlefish"
        echo "  Restart:       docker restart cuttlefish"
        echo ""
        echo -e "${YELLOW}Note: Build the Docker image first with: docker build -t cuttlefish .${NC}"
    fi
    echo ""
    echo -e "${BOLD}Access:${NC}"
    echo "  WebUI:         http://${SERVER_HOST}:${SERVER_PORT}"
    echo "  API Key:       ${API_KEY:0:8}...${API_KEY: -8}"
    if [[ "$ENABLE_DISCORD" == true ]]; then
        echo "  Discord:       Bot enabled"
    fi
    echo ""
    echo -e "${BOLD}CLI Commands:${NC}"
    echo "  cuttlefish --help           Show all commands"
    echo "  cuttlefish update check     Check for updates"
    echo "  cuttlefish-tui              Launch TUI client"
    echo ""
    
    if [[ "$DEPLOYMENT_MODE" == "systemd" ]]; then
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
    else
        info "Build and run the Docker container when ready (see commands above)"
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
    
    detect_platform
    
    if detect_existing_config; then
        echo ""
        echo -e "${GREEN}${BOLD}Existing installation detected!${NC}"
        echo "  Config: $EXISTING_CONFIG"
        echo ""
        echo "Installation options:"
        echo "  1) Upgrade - Reinstall binary, keep current config as defaults"
        echo "  2) Reconfigure - Go through all options with current values as defaults"
        echo "  3) Fresh install - Ignore existing config entirely"
        echo ""
        
        local install_mode
        prompt install_mode "Select option [1-3]" "1"
        
        case "$install_mode" in
            1)
                info "Upgrade mode: Reinstalling binary with existing configuration"
                load_existing_config
                load_existing_env
                detect_configured_providers

                check_dependencies
                install_rust

                create_user
                create_directories
                download_binary
                create_symlinks

                # Restart service if systemd is available
                if [[ "$HAS_SYSTEMD" == true ]]; then
                    info "Restarting Cuttlefish service..."
                    systemctl start cuttlefish 2>/dev/null || true
                    sleep 1
                    if systemctl is-active --quiet cuttlefish 2>/dev/null; then
                        success "Cuttlefish service restarted!"
                    else
                        warn "Service not started. Run: sudo systemctl start cuttlefish"
                    fi
                fi

                print_summary
                return
                ;;
            2)
                info "Reconfigure mode: Using existing values as defaults"
                load_existing_config
                load_existing_env
                detect_configured_providers
                ;;
            3)
                info "Fresh install: Ignoring existing configuration"
                UPGRADE_MODE=false
                EXISTING_CONFIG=""
                ;;
            *)
                info "Defaulting to upgrade mode"
                load_existing_config
                load_existing_env
                detect_configured_providers

                check_dependencies
                install_rust

                create_user
                create_directories
                download_binary
                create_symlinks

                # Restart service if systemd is available
                if [[ "$HAS_SYSTEMD" == true ]]; then
                    info "Restarting Cuttlefish service..."
                    systemctl start cuttlefish 2>/dev/null || true
                    sleep 1
                    if systemctl is-active --quiet cuttlefish 2>/dev/null; then
                        success "Cuttlefish service restarted!"
                    else
                        warn "Service not started. Run: sudo systemctl start cuttlefish"
                    fi
                fi

                print_summary
                return
                ;;
        esac
    fi
    
    select_deployment_mode
    
    check_dependencies
    install_rust
    
    configure_server
    configure_database
    configure_sandbox
    
    if [[ "$UPGRADE_MODE" == true && -n "$SELECTED_PROVIDERS" ]]; then
        echo ""
        echo -e "${BOLD}=== Provider Configuration ===${NC}"
        echo ""
        echo "Currently configured providers:"
        for idx in $SELECTED_PROVIDERS; do
            local pname
            pname=$(get_provider_field "$idx" name)
            echo "  - $pname"
        done
        echo ""
        
        if prompt_yn "Keep existing provider configuration?" "y"; then
            info "Keeping existing providers"
        else
            select_providers
            collect_provider_credentials
        fi
    else
        select_providers
        collect_provider_credentials
    fi
    
    configure_discord
    
    echo ""
    echo -e "${BOLD}=== Installation ===${NC}"
    echo ""
    
    create_user
    create_directories
    download_binary
    create_symlinks
    write_config
    write_env_file
    if [[ "$DEPLOYMENT_MODE" == "systemd" ]]; then
        write_systemd_service
    fi

    print_summary
}

# Run main function
main "$@"
