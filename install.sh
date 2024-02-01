#!/bin/sh

# This script should be run via curl:
# source < "$(curl -fsSL https://raw.githubusercontent.com/FifthTry/clift/main/install.sh)"

# The [ -t 1 ] check only works when the function is not called from
# a subshell (like in `$(...)` or `(...)`, so this hack redefines the
# function at the top level to always return false when stdout is not
# a tty.
if [ -t 1 ]; then
  is_tty() {
    true
  }
else
  is_tty() {
    false
  }
fi

setup_colors() {
    if ! is_tty; then
        FMT_RED=""
        FMT_GREEN=""
        FMT_YELLOW=""
        FMT_BLUE=""
        FMT_BOLD=""
        FMT_ORANGE=""
        FMT_RESET=""
    else
        FMT_RED="$(printf '\033[31m')"
        FMT_GREEN="$(printf '\033[32m')"
        FMT_YELLOW="$(printf '\033[33m')"
        FMT_BLUE="$(printf '\033[34m')"
        FMT_BOLD="$(printf '\033[1m')"
        FMT_ORANGE="$(printf '\033[38;5;208m')"
        FMT_RESET="$(printf '\033[0m')"
    fi
}

print_success_box() {
    log_message "╭────────────────────────────────────────╮"
    log_message "│                                        │"
    log_message "│   clift installation completed.        │"
    log_message "│                                        │"
    log_message "│                                        │"
    log_message "│   Get started with fastn at:           │"
    log_message "│   ${FMT_BLUE}https://fastn.com${FMT_RESET}                    │"
    log_message "│                                        │"
    log_message "╰────────────────────────────────────────╯"
}

# Function for logging informational messages
log_message() {
    echo "${FMT_GREEN}$1${FMT_RESET}"
}

# Function for logging error messages
log_error() {
    echo "${FMT_RED}ERROR:${FMT_RESET} $1"
}

command_exists() {
  command -v "$@" >/dev/null 2>&1
}

update_path() {
    local shell_config_file

    if [ -n "$ZSH_VERSION" ]; then
        shell_config_file="${HOME}/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        shell_config_file="${HOME}/.bashrc"
    else
        shell_config_file="${HOME}/.profile"
    fi

    echo ""

    # Create the shell config file if it doesn't exist
    if [ ! -e "$shell_config_file" ]; then
        touch "$shell_config_file"
    fi

    # Check if the path is already added to the shell config file
    if ! grep -qF "export PATH=\"\$PATH:${DESTINATION_PATH}\"" "$shell_config_file"; then
        if [ -w "$shell_config_file" ]; then
            # Add the destination path to the PATH variable in the shell config file
            echo "export PATH=\"\$PATH:${DESTINATION_PATH}\"" >> "$shell_config_file"
        else
            log_error "Failed to add '${DESTINATION_PATH}' to PATH. Insufficient permissions for '$shell_config_file'."
            log_message "The installer has successfully downloaded the \`clift\` binary in '${DESTINATION_PATH}' but it failed to add it in your \$PATH variable."
            log_message "Configure the \$PATH manually or run \`clift\` binary from '${DESTINATION_PATH}/clift'"
            return 1
        fi
    fi

    export PATH=$PATH:$DESTINATION_PATH
    return 0
}


setup() {
    VERSION=""

    # Parse arguments
    while [ $# -gt 0 ]; do
        case $1 in
            --version=*) VERSION="${1#*=}" ;;
            *) echo "Unknown CLI argument: $1"; exit 1 ;;
        esac
        shift
    done

    if [ -z "$VERSION" ] && [ -f clift-version ]; then
        VERSION=$(cat clift-version | tr -d '\n')
    fi

    DESTINATION_PATH="/usr/local/bin"

    if [ -d "$DESTINATION_PATH" ] && [ -w "$DESTINATION_PATH" ]; then
        DESTINATION_PATH=$DESTINATION_PATH
    else
        DESTINATION_PATH="${HOME}/.clift/bin"
        mkdir -p "$DESTINATION_PATH"
    fi

    if [ -n "$VERSION" ]; then
        URL="https://github.com/FiftTry/clift/releases/download/$VERSION"
        log_message "clift-version file found."
        log_message "Installing clift $VERSION in $DESTINATION_PATH."
    else
        URL="https://github.com/FiftTry/clift/releases/latest/download"
        log_message "clift-version file not found."
        log_message "Downloading the latest release of clift in $DESTINATION_PATH."
    fi

    if [ "$(uname)" = "Darwin" ]; then
        echo "MacOS is not supported yet."
        exit 1
    else
        FILENAME="clift_linux_musl_x86_64"
    fi

    # Download the binary directly using the URL
    curl -# -L -o "${DESTINATION_PATH}/clift" "${URL}/${FILENAME}"
    chmod +x "${DESTINATION_PATH}/clift"

    # Check if the destination files is present and executable before updating the PATH
    if [ -e "${DESTINATION_PATH}/clift" ]; then
        if update_path; then
            print_success_box
        else
            echo "Failed to update PATH settings in your shell."
            echo "Please manually add ${DESTINATION_PATH} to your PATH."
            echo "Or you can run clift using full path:"
            echo "${DESTINATION_PATH}/clift"
        fi
    else
        log_error "Installation failed. Please check if you have sufficient permissions to install in $DESTINATION_PATH."
    fi
}


main() {
    setup_colors

    if ! command_exists curl; then
        log_error "curl not found. Please install curl and execute the script once again"
        exit 1
    fi
    setup "$@"
}

main "$@"