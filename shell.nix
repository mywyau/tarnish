# { pkgs ? import <nixpkgs> {} }:

# pkgs.mkShell {
#   name = "rust-backend-env";

#   buildInputs = [
#     pkgs.rustup
#     pkgs.libiconv

#     # Diesel CLI with PostgreSQL support
#     pkgs.diesel-cli

#     # Additional utilities
#     pkgs.openssl
#     pkgs.pkg-config
#     pkgs.postgresql
#   ];

#   shellHook = ''
#     rustup toolchain install stable
#     rustup default stable
#     echo "Rust version: $(rustc --version)"
#     echo "Environment setup complete. Ready to develop!"
#   '';
# }

# { pkgs ? import <nixpkgs> {} }:

# pkgs.mkShell {
#   name = "rust-backend-env";

#   buildInputs = [
#     pkgs.rustup
#     pkgs.openssl
#     pkgs.pkg-config
#     pkgs.postgresql
#     pkgs.diesel-cli
#     pkgs.libiconv
#   ];

#   shellHook = ''
#      export PGDATA="$HOME/pgsql"
#         export PGPORT="5432"

#         if [ ! -d "$PGDATA" ]; then
#           mkdir -p "$PGDATA"
#           initdb -D "$PGDATA"
#         fi

#         echo "Starting PostgreSQL..."
#         pg_ctl -D "$PGDATA" -l logfile start
#         echo "PostgreSQL started at port $PGPORT"

#     export DATABASE_URL="postgres://myuser:mypassword@localhost:5432/mydatabase"
#     rustup toolchain install stable
#     rustup default stable
#     export LDFLAGS="-L${pkgs.libiconv}/lib"
#     export CPPFLAGS="-I${pkgs.libiconv}/include"
#     echo "Environment setup complete. Ready to develop!"
#   '';
# }


{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "rust-backend-env";

  buildInputs = [
    pkgs.rustup
    pkgs.openssl
    pkgs.pkg-config
    pkgs.postgresql
    pkgs.diesel-cli
    pkgs.libiconv
  ];

  shellHook = ''
    # PostgreSQL environment setup
    export PGDATA="$HOME/pgsql"
    export PGPORT="5432"

    if [ ! -d "$PGDATA" ]; then
      echo "Initializing PostgreSQL data directory..."
      initdb -D "$PGDATA"
    fi

    # Start PostgreSQL server if not already running
    if pg_ctl -D "$PGDATA" status > /dev/null 2>&1; then
      echo "PostgreSQL server is already running."
    else
      echo "Starting PostgreSQL..."
      pg_ctl -D "$PGDATA" -l "$PGDATA/logfile" start
      echo "PostgreSQL started at port $PGPORT"
    fi

    # Set environment variables
    export DATABASE_URL="postgres://myuser:mypassword@localhost:$PGPORT/postgres"

    # Ensure Rust is set up
    rustup toolchain install stable
    rustup default stable

    echo "Environment setup complete. Ready to develop!"
  '';
}
