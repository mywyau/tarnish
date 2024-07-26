# tarnish

## To run the dependencies using Nix

```
nix-shell
```

## To run the App

```
cargo build
```

```
cargo run
```

## To run Tests

```
cargo test
```

To run a single test

```
cargo test --test <name of test spec>

e.g.

cargo test --test crud_operations 
```

## API Endpoints


## HTTPie example requests


### Creating a blog post
```
http POST http://localhost:8080/blog/posts/create title="My First Post" body="This is the body of my first post."
```

### Getting a blog post
```
http GET http://localhost:8080/blog/posts/retrieve/1
```

### Updating a blog post
```
http PUT http://localhost:8080/blog/posts/update/1 title="Updated Title" body="This is the updated body."```
```

### Deleting a blog post
```
http DELETE http://localhost:8080/blog/posts/delete/single/1
```

## Postgres SQL

To check if the connection is established and ready

```
pg_isready
```

### Login as a user into the Postgres SQL db

```
psql -U myuser -d postgres -h localhost -p 5432
```


### Creating the table for Blog Posts
```
CREATE TABLE posts (
id SERIAL PRIMARY KEY,
title VARCHAR NOT NULL,
body TEXT NOT NULL
);
```