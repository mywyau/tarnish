{ pkgs ? import <nixpkgs> {} }:

pkgs.dockerTools.buildImage {
  name = "postgresql-docker";
  tag = "latest";

  contents = [
    pkgs.postgresql
  ];

  config = {
    Cmd = [
      "postgres"
      "-c" "listen_addresses=*"
    ];
    ExposedPorts = {
      "5432/tcp" = {};
    };
    Volumes = {
      "/var/lib/postgresql/data" = {};
    };
    Env = [
      "POSTGRES_USER=postgres"
      "POSTGRES_PASSWORD=your_password"
      "POSTGRES_DB=your_database"
    ];
  };
}
