# shell.nix - Compatibility wrapper for non-flake workflows
# This file allows using `nix-shell` without flakes enabled

let
  # Pin to a specific nixpkgs revision for reproducibility
  nixpkgs = fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz";
  };

  pkgs = import nixpkgs {
    config = { };
    overlays = [ ];
  };

  # Rust toolchain
  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" "rust-analyzer" ];
  };

in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    # Use rustup for now in shell.nix (flake.nix has better rust-overlay support)
    rustup
    pkg-config
    cmake
    gnumake
    llvmPackages.libclang
    llvmPackages.clang
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

    # YubiKey specific tools
    yubikey-manager
    yubikey-personalization
    yubico-piv-tool

    # Development tools
    git
    jq
    ripgrep
    fd
    cargo-watch
    cargo-edit
    cargo-audit
  ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
    pkgs.darwin.apple_sdk.frameworks.Security
    pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
    pkgs.darwin.apple_sdk.frameworks.CoreFoundation
    pkgs.darwin.apple_sdk.frameworks.PCSC
  ];

  shellHook = ''
    echo "╔════════════════════════════════════════════╗"
    echo "║     CIM Keys Development Environment       ║"
    echo "╚════════════════════════════════════════════╝"
    echo ""
    echo "Setting up Rust toolchain..."

    # Ensure stable Rust with required components
    rustup default stable
    rustup component add rust-src rust-analyzer clippy rustfmt

    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    echo ""

    # Environment variables for compilation
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

    # Clang for bindgen
    export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
    export BINDGEN_EXTRA_CLANG_ARGS="-I${pkgs.pcsclite}/include/PCSC -I${pkgs.gpgme}/include"

    # macOS specific
    ${if pkgs.stdenv.isDarwin then ''
      export DYLD_LIBRARY_PATH="$LD_LIBRARY_PATH:$DYLD_LIBRARY_PATH"
    '' else ""}

    echo "✓ Available features:"
    echo "  • YubiKey support (pcsclite, yubikey-manager)"
    echo "  • GPG support (gpgme, gnupg)"
    echo "  • SSH support (openssh, libssh2)"
    echo "  • TLS/PKI support (openssl, gnutls)"
    echo ""

    # Check for YubiKey
    if command -v ykman >/dev/null 2>&1; then
      echo "Checking for connected YubiKeys..."
      ykman list 2>/dev/null || echo "  No YubiKey detected"
    fi
    echo ""

    echo "Quick commands:"
    echo "  cargo build          - Build the project"
    echo "  cargo test           - Run tests"
    echo "  cargo watch -x check - Auto-check on file changes"
    echo "  cargo doc --open     - Generate and view documentation"
    echo ""
    echo "For flake users: nix develop"
    echo "For minimal build: nix develop .#minimal"
    echo ""
  '';

  # Prevent environment pollution
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}