#!/bin/bash
#═══════════════════════════════════════════════════════════════════════════════
# Install biomeOS Systemd User Units
#
# Installs:
#   biomeos-sovereign-tower.service  — BearDog + Songbird (sovereign mode)
#   biomeos-beacon-dns.service       — DNS beacon updater daemon
#
# Usage:
#   ./scripts/install_systemd_units.sh [install|uninstall|status]
#
#═══════════════════════════════════════════════════════════════════════════════

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

install() {
    echo "═══════════════════════════════════════════════════════════════"
    echo "🏰 Installing biomeOS Sovereign Systemd Units"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""

    mkdir -p "$SYSTEMD_USER_DIR"

    # Copy service files
    cp "$PROJECT_ROOT/config/systemd/biomeos-sovereign-tower.service" "$SYSTEMD_USER_DIR/"
    cp "$PROJECT_ROOT/config/systemd/biomeos-beacon-dns.service" "$SYSTEMD_USER_DIR/"

    # Reload systemd
    systemctl --user daemon-reload

    echo "✅ Service files installed to $SYSTEMD_USER_DIR/"
    echo ""
    echo "To enable and start:"
    echo "  systemctl --user enable --now biomeos-sovereign-tower"
    echo "  systemctl --user enable --now biomeos-beacon-dns"
    echo ""
    echo "To check status:"
    echo "  systemctl --user status biomeos-sovereign-tower"
    echo "  systemctl --user status biomeos-beacon-dns"
    echo ""
    echo "To view logs:"
    echo "  journalctl --user -u biomeos-sovereign-tower -f"
    echo "  journalctl --user -u biomeos-beacon-dns -f"
}

uninstall() {
    echo "Stopping and disabling services..."
    systemctl --user stop biomeos-sovereign-tower 2>/dev/null || true
    systemctl --user stop biomeos-beacon-dns 2>/dev/null || true
    systemctl --user disable biomeos-sovereign-tower 2>/dev/null || true
    systemctl --user disable biomeos-beacon-dns 2>/dev/null || true

    rm -f "$SYSTEMD_USER_DIR/biomeos-sovereign-tower.service"
    rm -f "$SYSTEMD_USER_DIR/biomeos-beacon-dns.service"

    systemctl --user daemon-reload
    echo "✅ Services removed"
}

status() {
    echo "═══════════════════════════════════════════════════════════════"
    echo "🏰 biomeOS Service Status"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    systemctl --user status biomeos-sovereign-tower 2>/dev/null || echo "  biomeos-sovereign-tower: not installed"
    echo ""
    systemctl --user status biomeos-beacon-dns 2>/dev/null || echo "  biomeos-beacon-dns: not installed"
}

case "${1:-install}" in
    install)   install ;;
    uninstall) uninstall ;;
    status)    status ;;
    *)
        echo "Usage: $0 {install|uninstall|status}"
        ;;
esac
