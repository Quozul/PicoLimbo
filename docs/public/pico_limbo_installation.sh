#!/usr/bin/env bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

REPO="Quozul/PicoLimbo"
BINARY_NAME="pico_limbo"

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

detect_arch() {
    local arch=$(uname -m)
    case $arch in
        x86_64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        *)
            print_error "Unsupported architecture: $arch"
            print_error "Supported architectures: x86_64, aarch64"
            exit 1
            ;;
    esac
}

get_install_dir() {
    if [[ $EUID -eq 0 ]]; then
        echo "/usr/local/bin"
    else
        local user_bin="$HOME/.local/bin"
        mkdir -p "$user_bin"
        echo "$user_bin"
    fi
}

check_path() {
    local dir="$1"
    if [[ ":$PATH:" != *":$dir:"* ]]; then
        print_warning "Directory '$dir' is not in your PATH."
        print_warning "Please add the following line to your shell profile (e.g., ~/.bashrc or ~/.zshrc):"
        print_warning "export PATH=\"\$PATH:$dir\""
    fi
}

get_latest_release_url() {
    local arch="$1"
    local api_url="https://api.github.com/repos/$REPO/releases/latest"

    local download_url
    if command -v curl >/dev/null 2>&1; then
        download_url=$(curl -s "$api_url" | grep -o "https://[^\"]*linux-${arch}-musl\.tar\.gz" | head -1)
    elif command -v wget >/dev/null 2>&1; then
        download_url=$(wget -qO- "$api_url" | grep -o "https://[^\"]*linux-${arch}-musl\.tar\.gz" | head -1)
    else
        print_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi

    if [[ -z "$download_url" ]]; then
        print_error "Could not find download URL for architecture: $arch"
        print_error "Please check if a release asset exists for your architecture."
        exit 1
    fi

    echo "$download_url"
}

download_and_install() {
    local url="$1"
    local install_dir="$2"
    local temp_dir=$(mktemp -d)

    local filename=$(basename "$url")
    if command -v curl >/dev/null 2>&1; then
        if ! curl --fail --silent --location -o "$temp_dir/$filename" "$url"; then
            print_error "Failed to download the file using curl."
            rm -rf "$temp_dir"
            exit 1
        fi
    else
        if ! wget --quiet -O "$temp_dir/$filename" "$url"; then
            print_error "Failed to download the file using wget."
            rm -rf "$temp_dir"
            exit 1
        fi
    fi

    if ! tar -xzf "$temp_dir/$filename" -C "$temp_dir"; then
        print_error "Failed to extract the archive. It may be corrupted."
        rm -rf "$temp_dir"
        exit 1
    fi

    local binary_path=$(find "$temp_dir" -name "$BINARY_NAME" -type f | head -1)

    if [[ -z "$binary_path" ]]; then
        print_error "Could not find '$BINARY_NAME' in the extracted files."
        print_error "Contents of archive:"
        find "$temp_dir"
        rm -rf "$temp_dir"
        exit 1
    fi

    if ! mv "$binary_path" "$install_dir/$BINARY_NAME"; then
        print_error "Failed to move binary to $install_dir. Check permissions."
        rm -rf "$temp_dir"
        exit 1
    fi
    chmod +x "$install_dir/$BINARY_NAME"

    rm -rf "$temp_dir"
}

main() {
    if [[ "$(uname)" != "Linux" ]]; then
        print_error "This script is designed for Linux systems only."
        exit 1
    fi

    local arch=$(detect_arch)

    local install_dir=$(get_install_dir)

    if [[ ! -w "$install_dir" ]]; then
        print_error "No write permission to $install_dir."
        if [[ $EUID -ne 0 ]]; then
            print_error "Try running with 'sudo' for a global installation."
        fi
        exit 1
    fi

    local download_url=$(get_latest_release_url "$arch")

    download_and_install "$download_url" "$install_dir"

    if [[ $EUID -ne 0 ]]; then
        check_path "$install_dir"
    fi

    local binary_path="$install_dir/$BINARY_NAME"
    print_status "The program has been installed at: $binary_path"

    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        "$BINARY_NAME" --help
    fi
}

main "$@"
