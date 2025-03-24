
## How to run
Having troubles with the scripts to generate two separate databases, instead an easy and naive way for now is to run two different docker containers.
Here are the commands:
```bash
docker run --name rust-postgres-db \
     -e POSTGRES_PASSWORD=psw \
     -e POSTGRES_USER=usr \
     -e POSTGRES_DB=blogs \
     -p 5432:5432 \
     -d postgres

docker run --name rust-postgres-db-test \
     -e POSTGRES_PASSWORD=psw \
     -e POSTGRES_USER=usr \
     -e POSTGRES_DB=blogs-test \
     -p 5433:5432 \
     -d postgres

sqlx migrate run --database-url postgres://usr:psw@localhost:5432/blogs
sqlx migrate run --database-url postgres://usr:psw@localhost:5433/blogs-test
```
