{
  description = "Tangle development environment";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # Rust
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
    # EVM dev tools
    foundry = {
      url = "github:shazow/foundry.nix/stable";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, rust-overlay, foundry, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) foundry.overlay ];
        pkgs = import nixpkgs { inherit system overlays; };
        lib = pkgs.lib;
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in {
        devShells.default = pkgs.mkShell {
          name = "tangle";
          nativeBuildInputs = [
            pkgs.gmp
            pkgs.protobuf
            pkgs.pkg-config
            pkgs.openssl
            # Needed for rocksdb-sys
            pkgs.clang
            pkgs.gcc
            pkgs.libbfd
            pkgs.libunwind
            pkgs.libblocksruntime
            pkgs.libclang.lib
            pkgs.libgcc
            pkgs.rustPlatform.bindgenHook
            # Mold Linker for faster builds (only on Linux)
            (lib.optionals pkgs.stdenv.isLinux pkgs.mold)
            (lib.optionals pkgs.stdenv.isDarwin
              pkgs.darwin.apple_sdk.frameworks.Security)
            (lib.optionals pkgs.stdenv.isDarwin
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration)
          ];
          # Fortify causes build failures: 'str*' defined both normally and as 'alias' attribute
          hardeningDisable = [ "fortify" ];
          buildInputs = [
            # Nodejs for test suite
            pkgs.nodePackages.typescript-language-server
            pkgs.nodejs_22
            pkgs.nodePackages.yarn
            # Finally the toolchain
            toolchain
            pkgs.foundry-bin
          ];
          packages = [
            pkgs.taplo
            pkgs.harper
            pkgs.cargo-nextest
            pkgs.cargo-expand
            pkgs.cargo-tarpaulin
            pkgs.lldb
          ];
          # Environment variables
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          # Needed for running DKG Node.
          LD_LIBRARY_PATH = lib.makeLibraryPath [
            pkgs.gmp
            pkgs.openssl
            pkgs.libclang
            pkgs.libgcc
            pkgs.stdenv.cc.cc
            pkgs.libbfd
            pkgs.libunwind
            pkgs.libblocksruntime
            pkgs.libopcodes
          ];
        };
      });
}
