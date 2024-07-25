{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "rust-backend-env";

  buildInputs = [
    pkgs.rustup
    pkgs.libiconv

    # Diesel CLI with PostgreSQL support
    pkgs.diesel-cli

    # Additional utilities
    pkgs.openssl
    pkgs.pkg-config
    pkgs.postgresql
  ];

  shellHook = ''
    rustup toolchain install stable
    rustup default stable
    echo "Rust version: $(rustc --version)"
    echo "Environment setup complete. Ready to develop!"
  '';
}
