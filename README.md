# tarnish

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


### Healt check
```
http GET http://localhost:8080/health
```


### Creating a blog post
```
http POST http://localhost:8080/blog/post/create id:=1 post_id="post_id_mikey" title="My First Post" body="This is the body of my first post."
```

### Getting a blog post
```
http GET http://localhost:8080/blog/post/retrieve/1
```

```
http GET http://localhost:8080/blog/post/get/all
```

### Getting a blog post by post_id
```
http GET http://localhost:8080/blog/post/retrieve/some_string

http GET http://localhost:8080/blog/post/retrieve/post-id/mikey-1
```


### Updating a blog post
```
http PUT http://localhost:8080/blog/posts/update/post_id_mikey id:=1 post_id="post_id_mikey" title="Updated Title" body="This is the updated body."```
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

### Deleting a single blog post based on post_id
```
http DELETE http://localhost:8080/blog/post/single/{post_id}
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
```
psql postgres://myuser:mypassword@localhost:5432/postgres
```
```
psql postgres://test:test-password@localhost:5430/test_db
```

psql -U test -d test_db -h localhost -p 5432

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

### Start up postgres in docker container if not present  
docker-compose up -d

### Set up database table if not present
diesel migration run

### Superuser
psql -U postgres


DROP TABLE IF EXISTS __diesel_schema_migrations CASCADE;


http POST http://localhost:8080/blog/skill/create id:=1 skill_id="skill-001" skill_name="Rust Programming" body="Comprehensive skill in Rust programming."

http GET http://localhost:8080/blog/skill/retrieve/skill-id/a 



http POST localhost:8080/blog/worklog/create id:=1 worklog_id="1234abcd" work_title="My First Worklog" body="This is the content of my worklog." created_at="2024-08-23T12:00:00" updated_at="2024-08-23T12:00:00"


http POST localhost:8080/blog/worklog/create id:=1 worklog_id="1234abcd" work_title="My First Worklog" body="This is the content of my worklog." time_created="2024-08-23T12:00:00" time_updated="2024-08-23T12:00:00"


DO $$
DECLARE
r RECORD;
BEGIN
-- Loop over all tables
FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = current_schema()) LOOP
EXECUTE 'DROP TABLE IF EXISTS ' || quote_ident(r.tablename) || ' CASCADE';
END LOOP;
END $$;

cargo watch -x run

```
http POST http://localhost:8080/blog/worklog/create \
Content-Type:application/json \
id:=1 \
worklog_id="worklog123" \
work_title="New Rust Blog Post" \
body="This is the body of the new worklog." \
created_at="2023-08-29T14:00:00Z" \
updated_at="2023-08-29T14:00:01Z"
```

http GET http://localhost:8080/blog/worklog/retrieve/worklog-id/worklog123