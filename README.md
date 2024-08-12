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
http POST http://localhost:8080/blog/post/create/mikey-1 id= 1 post_id="post_id_mikey" title="My First Post" body="This is the body of my first post."
```

### Getting a blog post
```
http GET http://localhost:8080/blog/post/retrieve/1
```

### Getting a blog post by post_id
```
http GET http://localhost:8080/blog/posts/retrieve/some_string
http GET http://localhost:8080/blog/posts/retrieve/post-id/mikey-1
```


### Updating a blog post
```
http PUT http://localhost:8080/blog/posts/update/post_id_mikey id= 1 post_id="post_id_mikey" title="Updated Title" body="This is the updated body."```
```

### Deleting a blog post
```
http DELETE http://localhost:8080/blog/post/single/1
```

### Deleting all blog posts
```
http DELETE http://localhost:8080/blog/post/all
```

### Deleting all blog posts with a response body
```
http DELETE http://localhost:8080/blog/posts/all/message
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

CREATE TABLE posts (
id SERIAL PRIMARY KEY,
post_id VARCHAR NOT NULL,
title VARCHAR NOT NULL,
body TEXT NOT NULL
);
```
### When in postgres view tables
```
\dt
```

### When in postgres delete table
```
DROP TABLE <table name>
```

### When in postgres delete all tables
```
DROP TABLE *
```