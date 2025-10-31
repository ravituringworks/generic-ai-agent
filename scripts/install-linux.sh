#!/bin/bash
# The Agency - Linux systemd Installation Script
# Run as root or with sudo

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${GREEN}The Agency - Linux systemd Installation${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}Error: This script must be run as root${NC}"
    echo "Please run: sudo $0"
    exit 1
fi

# Detect distribution
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$NAME
    VER=$VERSION_ID
else
    echo -e "${RED}Error: Cannot detect operating system${NC}"
    exit 1
fi

echo -e "${CYAN}Detected OS:${NC} $OS $VER"
echo ""

# Create user
echo -e "${CYAN}Creating system user...${NC}"
if ! id -u agency &>/dev/null; then
    useradd -r -s /bin/false -d /var/lib/the-agency agency
    echo -e "  ${GREEN}✓${NC} User 'agency' created"
else
    echo -e "  ${YELLOW}⚠${NC} User 'agency' already exists"
fi

# Create directories
echo ""
echo -e "${CYAN}Creating directories...${NC}"
directories=(
    "/usr/local/bin"
    "/etc/the-agency"
    "/var/lib/the-agency"
    "/var/log/the-agency"
    "/opt/the-agency"
)

for dir in "${directories[@]}"; do
    mkdir -p "$dir"
    echo -e "  ${GREEN}✓${NC} Created: $dir"
done

# Set permissions
chown -R agency:agency /var/lib/the-agency
chown -R agency:agency /var/log/the-agency
chown -R agency:agency /etc/the-agency
chmod 755 /var/lib/the-agency
chmod 755 /var/log/the-agency
chmod 755 /etc/the-agency

# Copy binary
echo ""
echo -e "${CYAN}Installing binary...${NC}"
if [ -f "./target/release/agency-daemon" ]; then
    cp "./target/release/agency-daemon" /usr/local/bin/
    chmod 755 /usr/local/bin/agency-daemon
    echo -e "  ${GREEN}✓${NC} Binary installed to /usr/local/bin/agency-daemon"
else
    echo -e "${RED}Error: Binary not found${NC}"
    echo "Please build first: cargo build --release --bin agency-daemon"
    exit 1
fi

# Copy configuration
echo ""
echo -e "${CYAN}Installing configuration...${NC}"
if [ -f "./config.example.toml" ]; then
    if [ ! -f "/etc/the-agency/config.toml" ]; then
        cp "./config.example.toml" /etc/the-agency/config.toml
        chown agency:agency /etc/the-agency/config.toml
        chmod 644 /etc/the-agency/config.toml
        echo -e "  ${GREEN}✓${NC} Config installed to /etc/the-agency/config.toml"
    else
        echo -e "  ${YELLOW}⚠${NC} Config already exists at /etc/the-agency/config.toml"
    fi
fi

# Install systemd service
echo ""
echo -e "${CYAN}Installing systemd service...${NC}"
if [ -f "./the-agency.service" ]; then
    cp "./the-agency.service" /etc/systemd/system/
    chmod 644 /etc/systemd/system/the-agency.service
    echo -e "  ${GREEN}✓${NC} Service file installed"
else
    echo -e "${RED}Error: Service file not found${NC}"
    exit 1
fi

# Reload systemd
echo ""
echo -e "${CYAN}Reloading systemd...${NC}"
systemctl daemon-reload
echo -e "  ${GREEN}✓${NC} Systemd reloaded"

# Enable service
echo ""
echo -e "${CYAN}Enabling service...${NC}"
systemctl enable the-agency
echo -e "  ${GREEN}✓${NC} Service enabled to start on boot"

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo -e "${CYAN}Service commands:${NC}"
echo -e "  Start service:    ${NC}sudo systemctl start the-agency"
echo -e "  Stop service:     ${NC}sudo systemctl stop the-agency"
echo -e "  Restart service:  ${NC}sudo systemctl restart the-agency"
echo -e "  Service status:   ${NC}sudo systemctl status the-agency"
echo -e "  View logs:        ${NC}sudo journalctl -u the-agency -f"
echo ""
echo -e "${CYAN}Configuration file:${NC}"
echo -e "  /etc/the-agency/config.toml"
echo ""

# Prompt to start service
read -p "Do you want to start the service now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo -e "${CYAN}Starting service...${NC}"
    systemctl start the-agency
    sleep 2
    
    if systemctl is-active --quiet the-agency; then
        echo -e "${GREEN}✓ Service started successfully!${NC}"
        echo ""
        echo -e "${CYAN}API available at:${NC} http://localhost:8080"
        echo -e "${CYAN}Health check:${NC} http://localhost:8080/health"
    else
        echo -e "${RED}✗ Service failed to start${NC}"
        echo "Check logs with: sudo journalctl -u the-agency -n 50"
    fi
fi

echo ""
echo -e "${GREEN}Setup complete!${NC}"
