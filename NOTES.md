# Migrations
export DATABASE_URL=postgres://app:secret@127.0.0.1:5432/newsletter
sqlx migrate add create_subsriptions_table

# Creating/Starting Database

## Running Database already?
SKIP_DOCKER=true ./scripts/init_db.sh