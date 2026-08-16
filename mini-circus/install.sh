#!/bin/sh
# Installs mini-circus by downloading a prebuilt binary from GitHub Releases.
#
#   curl -fsSL https://raw.githubusercontent.com/Nightmare99/circus/main/mini-circus/install.sh | sh
#
# Env vars:
#   MINI_CIRCUS_VERSION      version to install, e.g. "0.1.0" (default: latest)
#   MINI_CIRCUS_INSTALL_DIR  where to put the binary (default: $HOME/.local/bin)
set -eu

REPO="Nightmare99/circus"
INSTALL_DIR="${MINI_CIRCUS_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${MINI_CIRCUS_VERSION:-latest}"

say() { printf '%s\n' "$*"; }
err() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}
need_cmd() {
    command -v "$1" >/dev/null 2>&1 || err "need '$1' but it's not installed"
}

need_cmd curl
need_cmd tar
need_cmd mktemp

detect_target() {
    os=$(uname -s)
    arch=$(uname -m)

    case "$os" in
        Darwin) os_part="apple-darwin" ;;
        Linux) os_part="unknown-linux-gnu" ;;
        *) err "unsupported OS: $os (mini-circus ships prebuilt binaries for macOS and Linux only)

Build from source instead:
  cargo install --git https://github.com/$REPO mini-circus" ;;
    esac

    case "$arch" in
        x86_64 | amd64) arch_part="x86_64" ;;
        arm64 | aarch64) arch_part="aarch64" ;;
        *) err "unsupported architecture: $arch

Build from source instead:
  cargo install --git https://github.com/$REPO mini-circus" ;;
    esac

    printf '%s-%s\n' "$arch_part" "$os_part"
}

TARGET=$(detect_target)

if [ "$VERSION" = "latest" ]; then
    say "Resolving latest mini-circus release..."
    TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases" |
        grep -m1 '"tag_name": *"mini-circus-v' |
        sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
    [ -n "$TAG" ] || err "couldn't find a mini-circus release. Check https://github.com/$REPO/releases"
else
    TAG="mini-circus-v${VERSION#v}"
fi

say "Installing mini-circus ($TAG) for $TARGET..."

ASSET="mini-circus-${TARGET}.tar.gz"
BASE_URL="https://github.com/$REPO/releases/download/$TAG"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

if ! curl -fsSL "$BASE_URL/$ASSET" -o "$TMP_DIR/$ASSET"; then
    err "no release asset for $TARGET at $BASE_URL/$ASSET

Your platform may not have a prebuilt binary. Build from source instead:
  cargo install --git https://github.com/$REPO mini-circus"
fi

if curl -fsSL "$BASE_URL/SHA256SUMS" -o "$TMP_DIR/SHA256SUMS" 2>/dev/null; then
    EXPECTED=$(grep "$ASSET" "$TMP_DIR/SHA256SUMS" | awk '{print $1}')
    if [ -n "$EXPECTED" ]; then
        if command -v sha256sum >/dev/null 2>&1; then
            ACTUAL=$(sha256sum "$TMP_DIR/$ASSET" | awk '{print $1}')
        else
            ACTUAL=$(shasum -a 256 "$TMP_DIR/$ASSET" | awk '{print $1}')
        fi
        [ "$EXPECTED" = "$ACTUAL" ] || err "checksum mismatch for $ASSET - download may be corrupted or tampered with"
        say "Checksum verified."
    fi
else
    say "warning: could not fetch SHA256SUMS, skipping checksum verification"
fi

tar xzf "$TMP_DIR/$ASSET" -C "$TMP_DIR"
[ -f "$TMP_DIR/mini-circus" ] || err "downloaded archive did not contain a mini-circus binary"

mkdir -p "$INSTALL_DIR"
chmod +x "$TMP_DIR/mini-circus"
mv "$TMP_DIR/mini-circus" "$INSTALL_DIR/mini-circus"

say "Installed to $INSTALL_DIR/mini-circus"

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        say ""
        say "$INSTALL_DIR is not on your PATH. Add this to your shell profile:"
        say "  export PATH=\"$INSTALL_DIR:\$PATH\""
        ;;
esac

say ""
"$INSTALL_DIR/mini-circus" --version
