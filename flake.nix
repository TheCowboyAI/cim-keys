{
  description = "CIM Keys - Comprehensive cryptographic key management library";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
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

          # Library paths
          export LD_LIBRARY_PATH="${pkgs.openssl.out}/lib:${pkgs.pcsclite}/lib:${pkgs.gpgme}/lib:${pkgs.nettle}/lib:$LD_LIBRARY_PATH"

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

        # Build package (CLI)
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "cim-keys";
          version = "0.7.8";

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
        packages.cim-keys-gui = pkgs.rustPlatform.buildRustPackage {
          pname = "cim-keys-gui";
          version = "0.7.8";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          inherit nativeBuildInputs;
          buildInputs = buildInputs ++ guiBuildInputs;

          # Build with GUI features
          buildFeatures = [ "gui" ];

          preBuild = ''
            ${envVars}

            # Additional GUI environment variables
            export LD_LIBRARY_PATH="${pkgs.wayland}/lib:${pkgs.libGL}/lib:${pkgs.vulkan-loader}/lib:$LD_LIBRARY_PATH"
          '';

          # Only build the GUI binary
          cargoBuildFlags = [ "--bin" "cim-keys-gui" "--features" "gui" ];

          # Disable tests for GUI build
          doCheck = false;

          meta = with pkgs.lib; {
            description = "GUI for CIM Keys - Cryptographic key management";
            homepage = "https://github.com/thecowboyai/cim-keys";
            license = licenses.mit;
            maintainers = [];
          };
        };

        # Apps for nix run
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/cim-keys";
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

            # Run from the current directory (not from nix store)
            if [ -f ./target/debug/cim-keys-gui ]; then
              echo "Starting GUI..."
              exec ./target/debug/cim-keys-gui "$OUTPUT_DIR"
            else
              echo "GUI binary not found! Please build first with:"
              echo "  nix develop -c cargo build --bin cim-keys-gui --features gui"
              exit 1
            fi
          '';
        };
      });
}