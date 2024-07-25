# tarnish

### Refresh shell.nix

```
sudo nixos-rebuild switch
```

### run default.nix for docker with postgres sql image

```
nix-build
```


```
docker run -d -p 5432:5432 --name my-postgres $(cat result)
```


```
cargo build
```

```
cargo run
```

```
nix-build postgresql-docker.nix
```


```
docker run --name my-postgres -d -p 5432:5432 result/postgresql-docker:latest
```

docker exec -it my-postgres psql -U myuser -d mydatabase

docker run --name my-postgres \
-e POSTGRES_USER=myuser \
-e POSTGRES_PASSWORD=mypassword \
-e POSTGRES_DB=mydatabase \
-p 5432:5432 \
-d postgres:14

initdb -D /var/lib/postgresql/data

pg_ctl -D /var/lib/postgresql/data -l logfile start

pg_ctl -D /var/lib/postgresql/data status

psql -U myuser -d mydatabase -h localhost -p 5432

