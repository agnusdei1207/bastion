###### https://www.rustfinity.com/blog/create-high-performance-rest-api-with-rust#spinning-up-a-new-postgresql-database-using-docker

# DB

```bash
    # DB 셋팅
    sqlx database setup
    rm -rf migrations/
    sqlx migrate add create_tables
    # sql 작성
    cat create.sql >> migrations/20250402122726_create_tables.sql
    sqlx database drop
    sqlx database create
    sqlx migrate run
```

# 설치

cargo add tokio -F full
cargo add dotenvy
