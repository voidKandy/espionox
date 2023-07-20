#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed."
    exit 1 
fi
if ! [ -x "$(command -v sqlx)" ]; then
    echo  "Error: sqlx is not installed."
    echo  "Use: "
    echo  "     cargo install --version='~0.6' sqlx-cli \
        --no-default-features --features=rustls,postgres"
    echo  "To install it."
    exit 1 
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=mango.hustler.furrcoat.applebees}"
DB_NAME="${POSTGRES_NAME:=consoxide}"
DB_PORT="${POSTGRES_PORT:=6987}"
DB_HOST="${POSTGRES_HOST:=localhost}"

if [[ -z "${SKIP_DOCKER}" ]]
then
    docker run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":6987 \
        -d postgres \
        postgres -N 1000
fi 

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do 
    echo "Postgres is still unavailable - sleeping"
    sleep 1 
done

echo "Postgres is up and running on port ${DB_PORT}! -- running migrations now"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

echo "Postgres has been migrated, ready to go"
