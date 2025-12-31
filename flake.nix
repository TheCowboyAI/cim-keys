{
  description = "CIM Keys - Comprehensive cryptographic key management library";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    disko = {
      url = "github:nix-community/disko";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # CIM dependencies - fetched via SSH for internal repos
    cim-domain = {
      url = "git+ssh://git@github.com/TheCowboyAI/cim-domain";
      flake = false;
    };
    cim-domain-location = {
      url = "git+ssh://git@github.com/TheCowboyAI/cim-domain-location";
      flake = false;
    };
    cim-domain-person = {
      url = "git+ssh://git@github.com/TheCowboyAI/cim-domain-person";
      flake = false;
    };
    cim-domain-organization = {
      url = "git+ssh://git@github.com/TheCowboyAI/cim-domain-organization";
      flake = false;
    };
    cim-domain-policy = {
      url = "git+ssh://git@github.com/TheCowboyAI/cim-domain-policy";
      flake = false;
    };
    cim-domain-agent = {
      url = "git+ssh://git@github.com/TheCowboyAI/cim-domain-agent";
      flake = false;
    };
    cim-domain-relationship = {
      url = "git+ssh://git@github.com/TheCowboyAI/cim-domain-relationship";
      flake = false;
    };
    cim-domain-spaces = {
      url = "git+ssh://git@github.com/TheCowboyAI/cim-domain-conceptualspaces";
      flake = false;
    };
    cim-graph = {
      url = "git+ssh://git@github.com/TheCowboyAI/cim-graph";
      flake = false;
    };
    cim-ipld = {
      url = "git+ssh://git@github.com/TheCowboyAI/cim-ipld-graph";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, nixos-generators, disko
            , cim-domain, cim-domain-location, cim-domain-person
            , cim-domain-organization, cim-domain-policy, cim-domain-agent
            , cim-domain-relationship, cim-domain-spaces, cim-graph, cim-ipld
            , ... }@inputs:
    nixpkgs.lib.recursiveUpdate
    (flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          cmake
          gnumake
          llvmPackages.libclang
          llvmPackages.clang
          stdenv.cc
        ];

        buildInputs = with pkgs; [
          # Core cryptography libraries
          openssl
          openssl.dev

          # Smart card and YubiKey support
          pcsclite
          pcsc-tools

          # GPG and related
          gpgme
          gnupg
          libgpg-error
          libassuan
          libksba
          npth

          # Additional crypto libraries
          nettle
          gnutls
          libgcrypt

          # SSH support
          openssh
          libssh2

          # System libraries
          libiconv
          zlib

          # YubiKey specific tools (optional but useful for testing)
          yubikey-manager
          yubikey-personalization
          yubico-piv-tool

          # Development tools
          git
          jq
          ripgrep
          fd
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          pkgs.darwin.apple_sdk.frameworks.CoreFoundation
          pkgs.darwin.apple_sdk.frameworks.PCSC
        ];

        # GUI-specific dependencies for iced
        guiBuildInputs = with pkgs; [
          # Wayland
          wayland
          wayland-protocols
          libxkbcommon

          # X11
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi

          # Graphics
          vulkan-loader
          vulkan-headers
          mesa
          libGL

          # Additional
          fontconfig
          freetype
        ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
          libxkbcommon
          xorg.libxcb
        ];

        # Environment variables for compilation
        envVars = ''
          export RUST_BACKTRACE=1
          export RUST_LOG=debug

          # OpenSSL configuration
          export OPENSSL_DIR="${pkgs.openssl.dev}"
          export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
          export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"

          # PKG_CONFIG paths
          export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.pcsclite}/lib/pkgconfig:${pkgs.gpgme}/lib/pkgconfig:${pkgs.nettle}/lib/pkgconfig:$PKG_CONFIG_PATH"

          # Library paths (including GUI libraries for Wayland/X11)
          export LD_LIBRARY_PATH="${pkgs.openssl.out}/lib:${pkgs.pcsclite}/lib:${pkgs.gpgme}/lib:${pkgs.nettle}/lib:${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib:${pkgs.xorg.libX11}/lib:${pkgs.xorg.libXcursor}/lib:${pkgs.xorg.libXrandr}/lib:${pkgs.xorg.libXi}/lib:${pkgs.vulkan-loader}/lib:${pkgs.libGL}/lib:$LD_LIBRARY_PATH"

          # PCSC configuration
          export PCSCLITE_LIB_DIR="${pkgs.pcsclite}/lib"
          export PCSCLITE_INCLUDE_DIR="${pkgs.pcsclite}/include/PCSC"

          # GPG configuration
          export GPGME_LIB_DIR="${pkgs.gpgme}/lib"
          export GPGME_INCLUDE_DIR="${pkgs.gpgme}/include"

          # Clang for bindgen and C compilation
          export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
          export BINDGEN_EXTRA_CLANG_ARGS="-I${pkgs.pcsclite}/include/PCSC -I${pkgs.gpgme}/include -I${pkgs.stdenv.cc.libc.dev}/include"
          export CC="${pkgs.clang}/bin/clang"
          export CXX="${pkgs.clang}/bin/clang++"
          export CFLAGS="-I${pkgs.stdenv.cc.libc.dev}/include"
          export CXXFLAGS="-I${pkgs.stdenv.cc.libc.dev}/include"

          # macOS specific
          ${if pkgs.stdenv.isDarwin then ''
            export DYLD_LIBRARY_PATH="$LD_LIBRARY_PATH:$DYLD_LIBRARY_PATH"
          '' else ""}
        '';

        # Build package (CLI)
        cimKeysPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "cim-keys";
          version = "0.8.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          inherit nativeBuildInputs buildInputs;

          preBuild = envVars;

          # Disable tests that require hardware tokens
          checkPhase = ''
            cargo test --lib --bins -- --skip yubikey --skip gpg
          '';

          meta = with pkgs.lib; {
            description = "Comprehensive cryptographic key management library for CIM";
            homepage = "https://github.com/thecowboyai/cim-keys";
            license = licenses.mit;
            maintainers = [];
          };
        };

        # Build GUI package
        cimKeysGuiPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "cim-keys-gui";
          version = "0.8.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
            pkg-config
            makeWrapper
          ]);

          buildInputs = buildInputs ++ guiBuildInputs;

          # Build with GUI features
          buildFeatures = [ "gui" ];

          preBuild = ''
            ${envVars}

            # Copy assets that need to be embedded
            mkdir -p assets/fonts
            cp ${pkgs.noto-fonts-emoji}/share/fonts/noto/NotoColorEmoji.ttf assets/fonts/

            # Note: logo.png should already be in the source tree

            # Additional GUI environment variables
            export LD_LIBRARY_PATH="${pkgs.wayland}/lib:${pkgs.libGL}/lib:${pkgs.vulkan-loader}/lib:$LD_LIBRARY_PATH"
          '';

          # Only build the GUI binary
          cargoBuildFlags = [ "--bin" "cim-keys-gui" "--features" "gui" ];

          # Disable tests for GUI build
          doCheck = false;

          # Wrap binary with runtime library paths
          postFixup = ''
            wrapProgram $out/bin/cim-keys-gui \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath (buildInputs ++ guiBuildInputs)}"
          '';

          meta = with pkgs.lib; {
            description = "GUI for CIM Keys - Cryptographic key management";
            homepage = "https://github.com/thecowboyai/cim-keys";
            license = licenses.mit;
            maintainers = [];
          };
        };

      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs;
          buildInputs = buildInputs ++ guiBuildInputs;

          shellHook = ''
            echo "CIM Keys Development Environment"
            echo "================================="
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo ""
            echo "Available tools:"
            echo "  - YubiKey support: Enabled (pcsclite, yubikey-manager)"
            echo "  - GPG support: Enabled (gpgme, gnupg)"
            echo "  - SSH support: Enabled (openssh, libssh2)"
            echo "  - TLS/PKI support: Enabled (openssl, gnutls)"
            echo ""
            echo "Testing smart card availability:"
            if command -v pcsc_scan >/dev/null 2>&1; then
              echo "  - PC/SC tools available"
              # Start pcscd if not running (on Linux)
              ${if pkgs.stdenv.isLinux then ''
                if ! pgrep -x pcscd > /dev/null; then
                  echo "  - Starting pcscd daemon..."
                  sudo systemctl start pcscd 2>/dev/null || echo "  - Note: pcscd may need manual start"
                fi
              '' else ""}
            fi
            echo ""
            echo "To build the project: cargo build"
            echo "To run tests: cargo test"
            echo "To check YubiKey: ykman info (if YubiKey is connected)"
            echo ""

            ${envVars}
          '';
        };

        # Alternative shell for minimal functionality (no YubiKey/GPG)
        devShells.minimal = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
            openssl.dev
            libssh2
            zlib
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          shellHook = ''
            echo "CIM Keys Minimal Development Environment"
            echo "========================================"
            echo "This environment provides SSH/TLS/PKI support only"
            echo "YubiKey and GPG features are disabled"
            echo ""
            export OPENSSL_DIR="${pkgs.openssl.dev}"
            export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
            export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
          '';
        };

        # Expose packages
        packages.default = cimKeysPackage;
        packages.cim-keys-gui = cimKeysGuiPackage;

        # Apps for nix run
        apps.default = {
          type = "app";
          program = "${cimKeysPackage}/bin/cim-keys";
        };

        # GUI app that properly sets runtime library paths
        apps.gui = flake-utils.lib.mkApp {
          drv = pkgs.writeShellScriptBin "cim-keys-gui" ''
            OUTPUT_DIR="''${1:-/tmp/cim-keys-output}"

            # Set up library paths for GUI dependencies
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (buildInputs ++ guiBuildInputs)}:''${LD_LIBRARY_PATH}"

            # Ensure Wayland/X11 environment is set
            export XDG_RUNTIME_DIR="''${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

            # Reduce Vulkan validation verbosity
            export VK_LOADER_DEBUG=error
            export RUST_LOG="''${RUST_LOG:-warn}"

            echo "🔐 CIM Keys GUI"
            echo "📁 Output directory: $OUTPUT_DIR"
            echo "Starting GUI..."

            # Run the nix-built GUI binary
            exec ${cimKeysGuiPackage}/bin/cim-keys-gui "$OUTPUT_DIR"
          '';
        };
      }))
    # NixOS image outputs - Second argument to recursiveUpdate
    # Pre-built binaries are included directly from ./target/release/
    (let
      pkgsForImage = import nixpkgs { system = "x86_64-linux"; overlays = [ (import rust-overlay) ]; };

      # Pre-built binary included in nixos/sd-image/bin/ (copied from target/release/)
      # Build first with: cargo build --release --bin cim-keys-gui --features gui
      # Then copy: cp target/release/cim-keys-gui nixos/sd-image/bin/
      prebuiltBinary = ./nixos/sd-image/bin/cim-keys-gui;

      # Package the pre-built binary - use autoPatchelfHook to fix glibc interpreter
      cimKeysGuiForImage = pkgsForImage.stdenv.mkDerivation {
        pname = "cim-keys-gui";
        version = "0.8.0";
        src = prebuiltBinary;
        dontUnpack = true;

        nativeBuildInputs = with pkgsForImage; [ autoPatchelfHook makeWrapper ];

        # Libraries that autoPatchelfHook will use to resolve dependencies
        buildInputs = with pkgsForImage; [
          stdenv.cc.cc.lib  # libstdc++, libgcc_s
          glibc
          # Wayland
          wayland libxkbcommon
          # X11
          xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi xorg.libxcb
          # Graphics
          vulkan-loader mesa libGL libglvnd libdrm
          # Fonts
          fontconfig freetype harfbuzz
          # System
          openssl pcsclite zlib
        ];

        installPhase = ''
          mkdir -p $out/bin
          cp $src $out/bin/cim-keys-gui
          chmod +x $out/bin/cim-keys-gui
        '';

        # After autoPatchelf fixes the binary, wrap it for runtime paths
        postFixup = ''
          wrapProgram $out/bin/cim-keys-gui \
            --prefix LD_LIBRARY_PATH : "${pkgsForImage.lib.makeLibraryPath (with pkgsForImage; [
              wayland libxkbcommon xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi xorg.libxcb
              vulkan-loader mesa mesa.drivers libGL libglvnd libdrm
              fontconfig freetype harfbuzz openssl pcsclite zlib
            ])}" \
            --set FONTCONFIG_FILE "${pkgsForImage.fontconfig.out}/etc/fonts/fonts.conf"
        '';
      };

      # Module that adds pre-built cim-keys-gui and secrets to system
      cimKeysModule = { pkgs, lib, ... }: {
        environment.systemPackages = [ cimKeysGuiForImage ];

        # Include secrets files at /secrets/
        environment.etc."secrets/domain-bootstrap.json".source = ./secrets/domain-bootstrap.json;
        environment.etc."secrets/policy-bootstrap.json".source = ./secrets/policy-bootstrap.json;
        environment.etc."secrets/secrets.json".source = ./secrets/secrets.json;
        environment.etc."secrets/cowboyai.json".source = ./secrets/cowboyai.json;

        # Symlink /secrets -> /etc/secrets for convenience
        system.activationScripts.secretsSymlink = ''
          ln -sfn /etc/secrets /secrets
        '';
      };

    in {
      # SD card image for Raspberry Pi 4/5 (aarch64)
      # Note: aarch64 needs separate build, using module without pre-built x86_64 binary
      packages.aarch64-linux.sdImage = nixos-generators.nixosGenerate {
        system = "aarch64-linux";
        format = "sd-aarch64";
        modules = [
          ./nixos/sd-image/default.nix
        ];
      };

      # ISO image for x86_64 USB boot
      packages.x86_64-linux.isoImage = nixos-generators.nixosGenerate {
        system = "x86_64-linux";
        format = "iso";
        modules = [
          ./nixos/sd-image/default.nix
          cimKeysModule
          ({ pkgs, lib, ... }: {
            # ISO-specific overrides
            services.cage.enable = lib.mkForce false;
            services.xserver = {
              enable = true;
              displayManager.lightdm.enable = true;
              windowManager.openbox.enable = true;
            };
            services.displayManager.autoLogin = {
              enable = true;
              user = "cimkeys";
            };
          })
        ];
      };

      # Raw EFI disk image for x86_64
      packages.x86_64-linux.rawImage = nixos-generators.nixosGenerate {
        system = "x86_64-linux";
        format = "raw-efi";
        modules = [
          ./nixos/sd-image/default.nix
          cimKeysModule
          ({ pkgs, lib, ... }: {
            # Disable Wayland kiosk
            services.cage.enable = lib.mkForce false;

            # X11 with LightDM and Openbox (same as isoImage)
            services.xserver = {
              enable = true;
              displayManager.lightdm.enable = true;
              windowManager.openbox.enable = true;
            };
            services.displayManager.autoLogin = {
              enable = true;
              user = "cimkeys";
            };

            # Start cim-keys-gui on login
            environment.etc."xdg/openbox/autostart".text = ''
              ${cimKeysGuiForImage}/bin/cim-keys-gui /home/cimkeys/cim-keys-output &
            '';
          })
        ];
      };

      # SD card image for x86_64 (bootable live image)
      packages.x86_64-linux.sdImage = nixos-generators.nixosGenerate {
        system = "x86_64-linux";
        format = "sd-x86_64";
        modules = [
          ./nixos/sd-image/default.nix
          cimKeysModule
          ({ pkgs, lib, ... }: {
            services.cage.enable = lib.mkForce false;
            services.getty.autologinUser = "cimkeys";
          })
        ];
      };

      # NixOS configuration for disko-install (primary SD card install method)
      nixosConfigurations.cim-keys = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          disko.nixosModules.disko
          ./nixos/sd-image/disko.nix
          ./nixos/sd-image/default.nix
          cimKeysModule
          ({ pkgs, lib, modulesPath, ... }: {
            imports = [ (modulesPath + "/installer/scan/not-detected.nix") ];

            nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";

            # Secure boot partition mount options (must match disko.nix)
            fileSystems."/boot".options = [ "umask=0077" "uid=0" "gid=0" ];

            # Use systemd-boot for UEFI
            boot.loader.systemd-boot.enable = lib.mkForce true;
            boot.loader.efi.canTouchEfiVariables = true;
            boot.loader.grub.enable = false;

            # Secure the random seed file
            boot.loader.systemd-boot.graceful = true;

            boot.initrd = {
              availableKernelModules = [
                "ext4" "vfat" "ahci" "nvme" "xhci_pci" "thunderbolt"
                "usb_storage" "sd_mod" "rtsx_pci_sdmmc" "sdhci_pci"
                "mmc_block" "sdhci" "sdhci_acpi"
              ];
              supportedFilesystems = [ "ext4" "vfat" ];
            };
            boot.kernelModules = [ "kvm-intel" "kvm-amd" ];

            # Cage kiosk running pre-built cim-keys-gui directly
            services.cage.program = lib.mkForce "${pkgs.writeShellScript "cim-keys-launcher" ''
              OUTPUT_DIR="/home/cimkeys/cim-keys-output"
              mkdir -p "$OUTPUT_DIR"

              # Wait for smart card daemon
              sleep 2

              # CRITICAL: Tell winit/iced to use Wayland backend
              export WINIT_UNIX_BACKEND=wayland
              export XDG_SESSION_TYPE=wayland

              # wgpu backend preferences
              export WGPU_BACKEND=vulkan,gl
              export WGPU_POWER_PREF=low

              # Reduce debug noise
              export VK_LOADER_DEBUG=none
              export RUST_BACKTRACE=1
              export RUST_LOG=warn,iced=info

              # Log startup
              echo "Starting cim-keys-gui at $(date)" >> /tmp/cim-keys.log
              echo "WAYLAND_DISPLAY=$WAYLAND_DISPLAY" >> /tmp/cim-keys.log
              echo "WINIT_UNIX_BACKEND=$WINIT_UNIX_BACKEND" >> /tmp/cim-keys.log
              env >> /tmp/cim-keys.log

              # Run with error capture
              ${cimKeysGuiForImage}/bin/cim-keys-gui "$OUTPUT_DIR" 2>&1 | tee -a /tmp/cim-keys.log

              # If we get here, the app crashed - show error in terminal
              echo "cim-keys-gui exited, showing log..." >> /tmp/cim-keys.log
              exec ${pkgs.foot}/bin/foot -e ${pkgs.bash}/bin/bash -c 'echo "=== CIM Keys crashed ===" && cat /tmp/cim-keys.log && echo "" && echo "Press Enter to retry..." && read && exec ${cimKeysGuiForImage}/bin/cim-keys-gui /home/cimkeys/cim-keys-output'
            ''}";
          })
        ];
      };
    });
}