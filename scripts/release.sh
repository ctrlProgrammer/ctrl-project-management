#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.1.0}"
NAME="ctrl-project-management"
ARCH="amd64"

echo "==> Building release binary..."
cargo build --release

BIN="target/release/$NAME"
if [ ! -f "$BIN" ]; then
    echo "Error: Binary not found at $BIN"
    exit 1
fi

echo "==> Creating tar.gz..."
TARBALL="${NAME}-v${VERSION}-linux-x86_64.tar.gz"
cp "target/release/$NAME" "target/release/$NAME-script"
cp scripts/install.sh "target/release/"
cp scripts/ctrl-project-management.desktop "target/release/"
tar czf "$TARBALL" -C target/release "$NAME" "install.sh" "ctrl-project-management.desktop"
rm -f "target/release/$NAME-script" "target/release/install.sh" "target/release/ctrl-project-management.desktop"
echo "    Created: $TARBALL"

echo "==> Creating .deb..."
DEB_DIR="${NAME}_${VERSION}_${ARCH}"
mkdir -p "$DEB_DIR/DEBIAN"
mkdir -p "$DEB_DIR/usr/local/bin"
mkdir -p "$DEB_DIR/usr/local/share/applications"

cat > "$DEB_DIR/DEBIAN/control" <<EOF
Package: ${NAME}
Version: ${VERSION}
Section: office
Priority: optional
Architecture: ${ARCH}
Maintainer: Ctrl Projects
Description: Kanban project manager with AI agent MCP integration
 A GTK4 kanban board for project management with an integrated
 MCP (Model Context Protocol) server for AI agent access.
EOF

install -m 755 "$BIN" "$DEB_DIR/usr/local/bin/"
install -m 644 scripts/ctrl-project-management.desktop "$DEB_DIR/usr/local/share/applications/"

dpkg-deb --build "$DEB_DIR" > /dev/null
rm -rf "$DEB_DIR"
echo "    Created: ${DEB_DIR}.deb"

echo ""
echo "==> Done!"
echo "    tar.gz: $TARBALL"
echo "    .deb:   ${DEB_DIR}.deb"
echo ""
echo "Install:"
echo "  sudo dpkg -i ${DEB_DIR}.deb"
echo "  or"
echo "  tar xzf $TARBALL && ./install.sh"
