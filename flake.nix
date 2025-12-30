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

  outputs = { self, nixpkgs, rust-overlay, flake-utils, nixos-generators
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
    # All CIM dependencies are pre-fetched from GitHub and included in /tmp/cim-build
    # A first-boot script sets up the workspace and builds cim-keys
    (let
      # Module to include all CIM sources for on-device build
      cimSourcesModule = { pkgs, lib, ... }: {
        # Include cim-keys source
        environment.etc."cim-build/cim-keys".source = ./.;

        # Include all dependencies from GitHub
        environment.etc."cim-build/cim-domain".source = cim-domain;
        environment.etc."cim-build/cim-domain-location".source = cim-domain-location;
        environment.etc."cim-build/cim-domain-person".source = cim-domain-person;
        environment.etc."cim-build/cim-domain-organization".source = cim-domain-organization;
        environment.etc."cim-build/cim-domain-policy".source = cim-domain-policy;
        environment.etc."cim-build/cim-domain-agent".source = cim-domain-agent;
        environment.etc."cim-build/cim-domain-relationship".source = cim-domain-relationship;
        environment.etc."cim-build/cim-domain-spaces".source = cim-domain-spaces;
        environment.etc."cim-build/cim-graph".source = cim-graph;
        environment.etc."cim-build/cim-ipld".source = cim-ipld;

        # First-boot build script
        systemd.services.cim-keys-build = {
          description = "Build CIM Keys on first boot";
          wantedBy = [ "multi-user.target" ];
          after = [ "network.target" ];
          serviceConfig = {
            Type = "oneshot";
            RemainAfterExit = true;
            User = "cimkeys";
            WorkingDirectory = "/tmp/cim-build";
          };
          path = with pkgs; [ coreutils cargo rustc pkg-config openssl.dev gcc gnumake ];
          script = ''
            # Only run once
            if [ -f /home/cimkeys/.cim-keys-built ]; then
              exit 0
            fi

            echo "Setting up CIM Keys build environment..."

            # Copy sources to writable location
            mkdir -p /tmp/cim-build
            cp -r /etc/cim-build/* /tmp/cim-build/
            chmod -R u+w /tmp/cim-build

            # Update Cargo.toml to use local paths
            cd /tmp/cim-build/cim-keys
            sed -i 's|path = "\.\./cim-domain"|path = "/tmp/cim-build/cim-domain"|g' Cargo.toml
            sed -i 's|path = "\.\./cim-domain-location"|path = "/tmp/cim-build/cim-domain-location"|g' Cargo.toml
            sed -i 's|path = "\.\./cim-domain-person"|path = "/tmp/cim-build/cim-domain-person"|g' Cargo.toml
            sed -i 's|path = "\.\./cim-domain-organization"|path = "/tmp/cim-build/cim-domain-organization"|g' Cargo.toml
            sed -i 's|path = "\.\./cim-domain-policy"|path = "/tmp/cim-build/cim-domain-policy"|g' Cargo.toml
            sed -i 's|path = "\.\./cim-domain-agent"|path = "/tmp/cim-build/cim-domain-agent"|g' Cargo.toml
            sed -i 's|path = "\.\./cim-domain-relationship"|path = "/tmp/cim-build/cim-domain-relationship"|g' Cargo.toml
            sed -i 's|path = "\.\./cim-domain-spaces"|path = "/tmp/cim-build/cim-domain-spaces"|g' Cargo.toml
            sed -i 's|path = "\.\./cim-graph"|path = "/tmp/cim-build/cim-graph"|g' Cargo.toml
            sed -i 's|path = "\.\./cim-ipld"|path = "/tmp/cim-build/cim-ipld"|g' Cargo.toml

            # Set up environment
            export OPENSSL_DIR="${pkgs.openssl.dev}"
            export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.pcsclite}/lib/pkgconfig"

            # Build
            echo "Building cim-keys..."
            cargo build --release 2>&1 | tee /tmp/cim-keys-build.log

            # Install binaries
            if [ -f target/release/cim-keys ]; then
              mkdir -p /home/cimkeys/.local/bin
              cp target/release/cim-keys /home/cimkeys/.local/bin/
              cp target/release/cim-keys-gui /home/cimkeys/.local/bin/ 2>/dev/null || true
              chown -R cimkeys:users /home/cimkeys/.local
              touch /home/cimkeys/.cim-keys-built
              echo "CIM Keys built successfully!"
            else
              echo "Build failed - check /tmp/cim-keys-build.log"
            fi
          '';
        };

        # Add local bin to PATH
        environment.variables.PATH = lib.mkForce "/home/cimkeys/.local/bin:/run/current-system/sw/bin";
      };

    in {
      # SD card image for Raspberry Pi 4/5 (aarch64)
      packages.aarch64-linux.sdImage = nixos-generators.nixosGenerate {
        system = "aarch64-linux";
        format = "sd-aarch64";
        modules = [
          ./nixos/sd-image/default.nix
          cimSourcesModule
        ];
      };

      # ISO image for x86_64 USB boot
      packages.x86_64-linux.isoImage = nixos-generators.nixosGenerate {
        system = "x86_64-linux";
        format = "iso";
        modules = [
          ./nixos/sd-image/default.nix
          cimSourcesModule
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

      # Raw disk image for x86_64
      packages.x86_64-linux.rawImage = nixos-generators.nixosGenerate {
        system = "x86_64-linux";
        format = "raw";
        modules = [
          ./nixos/sd-image/default.nix
          cimSourcesModule
          ({ pkgs, lib, ... }: {
            services.cage.enable = lib.mkForce false;
            services.getty.autologinUser = "cimkeys";
          })
        ];
      };
    });
}