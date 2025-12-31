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

  # Enable firmware for Intel GPU
  hardware.enableRedistributableFirmware = true;
  hardware.cpu.intel.updateMicrocode = true;

  # Graphics - Intel integrated GPU support
  hardware.graphics = {
    enable = true;
    enable32Bit = false;
    extraPackages = with pkgs; [
      # Intel GPU (Iris/UHD)
      intel-media-driver    # iHD driver for newer Intel
      vaapiIntel           # i965 driver for older Intel
      intel-compute-runtime # OpenCL
      # Mesa Vulkan drivers
      mesa.drivers
    ];
  };

  # Environment for GPU/Vulkan - set driver paths correctly
  environment.sessionVariables = {
    # DRI/Mesa driver paths
    LIBGL_DRIVERS_PATH = "/run/opengl-driver/lib/dri";
    __EGL_VENDOR_LIBRARY_DIRS = "/run/opengl-driver/share/glvnd/egl_vendor.d";
    # Vulkan ICD (Intel ANV driver)
    VK_ICD_FILENAMES = "/run/opengl-driver/share/vulkan/icd.d/intel_icd.x86_64.json";
    # Wayland
    XDG_SESSION_TYPE = "wayland";
    XDG_RUNTIME_DIR = "/run/user/1000";
    # Reduce debug noise
    VK_LOADER_DEBUG = "none";
  };

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

  # System packages (cim-keys-gui is added by cimKeysModule in flake.nix)
  environment.systemPackages = with pkgs; [
    # YubiKey tools
    yubikey-manager
    yubikey-personalization
    yubico-piv-tool
    pcsc-tools

    # Cryptography
    openssl
    gnupg

    # NATS tools
    natscli

    # Utilities
    vim
    htop
    tree
    file
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
