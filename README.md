
## How to run
Launch a docker container with Postgres with the following command:

```bash
docker run --name rust-postgres-db \
    -e POSTGRES_PASSWORD=yourpassword \
    -e POSTGRES_USER=yourusernae \
    -e POSTGRES_DB=blogs-db \
    -p 5432:5432 \
    -d postgres
```
