# Copyright (c) 2025 - Cowboy AI, LLC.
#
# CIM Keys Air-Gapped Live Image Configuration
#
# Uses nixos-generators for image creation.
# Build with: nix build .#sdImage
#
# Supported formats:
#   - sd-aarch64: Raspberry Pi 4/5 SD card
#   - iso: Bootable USB/CD ISO
#   - raw: Raw disk image

{ pkgs, lib, ... }:

{
  # System identification
  networking.hostName = "cim-keys";

  # AIR-GAPPED: Disable ALL networking
  networking.useDHCP = false;
  networking.networkmanager.enable = false;
  networking.wireless.enable = false;
  networking.firewall.enable = true;
  networking.firewall.allowedTCPPorts = [ ];
  networking.firewall.allowedUDPPorts = [ ];

  # Disable network services
  services.openssh.enable = false;
  services.avahi.enable = false;

  # Time zone (no NTP - air-gapped)
  time.timeZone = "UTC";
  services.timesyncd.enable = false;

  # Console
  console = {
    font = "Lat2-Terminus16";
    keyMap = "us";
  };
  i18n.defaultLocale = "en_US.UTF-8";

  # Smart card daemon for YubiKey
  services.pcscd.enable = true;

  # Udev rules for YubiKey
  services.udev.packages = [ pkgs.yubikey-personalization ];

  # Auto-login user
  users.users.cimkeys = {
    isNormalUser = true;
    description = "CIM Keys Operator";
    extraGroups = [ "wheel" "video" "audio" "plugdev" ];
    initialPassword = "cimkeys";
  };

  security.sudo.wheelNeedsPassword = false;

  # Graphics
  hardware.graphics.enable = true;

  # Cage kiosk for single-app display (Wayland)
  services.cage = {
    enable = true;
    user = "cimkeys";
    program = "${pkgs.writeShellScript "cim-keys-launcher" ''
      OUTPUT_DIR="/home/cimkeys/cim-keys-output"
      mkdir -p "$OUTPUT_DIR"

      # Wait for smart card daemon
      sleep 2

      # Try to run the GUI, fall back to terminal if not available
      if command -v cim-keys-gui >/dev/null 2>&1; then
        exec cim-keys-gui "$OUTPUT_DIR"
      else
        # Fallback: start terminal with instructions
        exec ${pkgs.foot}/bin/foot -e ${pkgs.writeShellScript "cim-keys-session" ''
          echo "========================================"
          echo "CIM Keys - Air-Gapped PKI System"
          echo "========================================"
          echo ""
          echo "YubiKey Status:"
          ykman list 2>/dev/null || echo "  No YubiKey detected"
          echo ""
          echo "Output directory: /home/cimkeys/cim-keys-output/"
          echo ""
          echo "CLI Usage:"
          echo "  cim-keys --help"
          echo ""
          exec $SHELL
        ''}
      fi
    ''}";
  };

  # System packages
  environment.systemPackages = with pkgs; [
    # YubiKey tools
    yubikey-manager
    yubikey-personalization
    yubico-piv-tool
    pcsc-tools

    # Cryptography
    openssl
    gnupg

    # Rust toolchain for building cim-keys
    rustc
    cargo

    # Build dependencies
    pkg-config
    cmake

    # NATS tools
    natscli

    # Utilities
    vim
    htop
    tree
    file
    git
    jq

    # Terminal
    foot

    # Encryption
    cryptsetup
  ];

  # Fonts
  fonts.packages = with pkgs; [
    noto-fonts
    noto-fonts-emoji
    fira-code
  ];

  # Clone cim-keys on first boot
  system.activationScripts.cloneCimKeys = ''
    if [ ! -d /home/cimkeys/cim-keys ]; then
      echo "Note: cim-keys source should be copied to /home/cimkeys/cim-keys"
    fi
  '';

  users.motd = ''
    ============================================
    CIM Keys - Air-Gapped PKI Management System
    ============================================

    This system is COMPLETELY OFFLINE.
    All network interfaces are disabled.

    Connect your YubiKey(s) and proceed.
    ============================================
  '';

  zramSwap.enable = true;
  system.stateVersion = "24.11";
}
