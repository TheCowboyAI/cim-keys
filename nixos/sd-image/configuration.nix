# Copyright (c) 2025 - Cowboy AI, LLC.
#
# CIM Keys Air-Gapped SD Card Image
#
# This configuration creates a bootable NixOS SD card image for
# secure, air-gapped PKI key generation and YubiKey provisioning.
#
# Build with: nix build .#sdImage
# Write to SD: sudo dd if=result/sd-image/*.img of=/dev/sdX bs=4M status=progress

{ config, pkgs, lib, ... }:

{
  # Import SD card image module
  imports = [
    <nixpkgs/nixos/modules/installer/sd-card/sd-image-aarch64.nix>
  ];

  # System identification
  networking.hostName = "cim-keys";

  # AIR-GAPPED: Disable ALL networking
  networking.useDHCP = false;
  networking.networkmanager.enable = false;
  networking.wireless.enable = false;
  networking.firewall.enable = true;
  networking.firewall.allowedTCPPorts = [ ];
  networking.firewall.allowedUDPPorts = [ ];

  # Disable unnecessary services
  services.openssh.enable = false;
  services.avahi.enable = false;

  # Time zone (no NTP - air-gapped)
  time.timeZone = "UTC";
  services.timesyncd.enable = false;

  # Console and locale
  console = {
    font = "Lat2-Terminus16";
    keyMap = "us";
  };
  i18n.defaultLocale = "en_US.UTF-8";

  # Boot configuration
  boot.loader.grub.enable = false;
  boot.loader.generic-extlinux-compatible.enable = true;

  # Kernel modules for YubiKey/smart cards
  boot.kernelModules = [ "usbhid" ];

  # Smart card daemon (required for YubiKey)
  services.pcscd.enable = true;

  # Udev rules for YubiKey
  services.udev.packages = [ pkgs.yubikey-personalization ];

  # Auto-login user for kiosk mode
  users.users.cimkeys = {
    isNormalUser = true;
    description = "CIM Keys Operator";
    extraGroups = [ "wheel" "video" "audio" "plugdev" ];
    initialPassword = "cimkeys"; # Change on first boot
  };

  # Allow wheel users to use sudo without password (for setup)
  security.sudo.wheelNeedsPassword = false;

  # Enable graphics for GUI
  hardware.graphics.enable = true;

  # X11/Wayland for GUI
  services.xserver = {
    enable = true;

    # Minimal window manager
    windowManager.openbox.enable = true;

    # Auto-login to cimkeys user
    displayManager = {
      lightdm = {
        enable = true;
        autoLogin = {
          enable = true;
          user = "cimkeys";
        };
      };
      defaultSession = "none+openbox";
    };
  };

  # Openbox autostart - launch cim-keys GUI
  environment.etc."xdg/openbox/autostart".text = ''
    # Hide cursor after inactivity
    unclutter -idle 3 &

    # Set background color
    xsetroot -solid "#1a1a2e"

    # Wait for smart card daemon
    sleep 2

    # Launch CIM Keys GUI
    ${pkgs.writeShellScript "start-cim-keys" ''
      OUTPUT_DIR="/home/cimkeys/cim-keys-output"
      mkdir -p "$OUTPUT_DIR"

      # Set up library paths
      export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
        pkgs.wayland
        pkgs.libxkbcommon
        pkgs.vulkan-loader
        pkgs.libGL
        pkgs.mesa
      ]}:$LD_LIBRARY_PATH"

      # Run CIM Keys GUI
      exec cim-keys-gui "$OUTPUT_DIR"
    ''} &
  '';

  # System packages
  environment.systemPackages = with pkgs; [
    # CIM Keys (will be built from source)
    # cim-keys-gui  # Uncomment when package is available

    # YubiKey tools
    yubikey-manager
    yubikey-personalization
    yubico-piv-tool
    pcsc-tools

    # Cryptography
    openssl
    gnupg

    # NATS tools (for credential generation)
    nats-server
    natscli

    # Utilities
    vim
    htop
    tree
    file
    unclutter

    # Terminal (for debugging)
    xterm

    # Encryption tools for SD card output
    cryptsetup
    parted
    dosfstools
  ];

  # Fonts for GUI
  fonts.packages = with pkgs; [
    noto-fonts
    noto-fonts-emoji
    liberation_ttf
    fira-code
  ];

  # Create mount point for encrypted output
  systemd.tmpfiles.rules = [
    "d /mnt/encrypted 0755 cimkeys users -"
  ];

  # Message of the day
  users.motd = ''
    ============================================
    CIM Keys - Air-Gapped PKI Management System
    ============================================

    This system is COMPLETELY OFFLINE by design.
    All network interfaces are disabled.

    The GUI will start automatically.
    Connect your YubiKey(s) before proceeding.

    Output will be saved to: /home/cimkeys/cim-keys-output/

    IMPORTANT: Save SECRETS.json securely!
    ============================================
  '';

  # Enable zram swap (no swap partition needed)
  zramSwap.enable = true;

  # System state version
  system.stateVersion = "24.11";
}
