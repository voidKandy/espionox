#!/usr/bin/env bash
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed."
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo "Error: sqlx is not installed."
    echo "Use:"
    echo "     cargo install --version='~0.6' sqlx-cli \
        --no-default-features --features=rustls,postgres"
    echo "To install it."
    exit 1
fi

# Check if yq is installed, and if not, prompt the user to install it
if ! [ -x "$(command -v yq)" ]; then
    read -p "yq is not installed. Do you want to install it? In it's current version this script only installs yq for Mac (y/n): " install_yq
    if [ "$install_yq" == "y" ]; then
        # Install yq
        echo "Installing yq..."
        wget https://github.com/mikefarah/yq/releases/download/v4.11.2/yq_darwin_amd64 -O yq
        chmod +x yq
        sudo mv yq /usr/local/bin/
        echo "yq has been installed."
    else
        echo "yq is required for this script. Please install it manually from https://github.com/mikefarah/yq"
        exit 1
    fi
fi

CONFIG_DIR="configuration/"

# Check if the configuration directory exists
if [ ! -d "$CONFIG_DIR" ]; then
    echo "Error: Configuration directory $CONFIG_DIR does not exist."
    exit 1
fi

# Read default values from default.yaml using yq
DEFAULT_CONFIG_FILE="configuration/default.yaml"
if [ -f "$DEFAULT_CONFIG_FILE" ]; then
    DEFAULT_DB_USER=$(yq eval '.database.username' "$DEFAULT_CONFIG_FILE")
    DEFAULT_DB_PASSWORD=$(yq eval '.database.password' "$DEFAULT_CONFIG_FILE")
    DEFAULT_DB_NAME=$(yq eval '.database.database_name' "$DEFAULT_CONFIG_FILE")
    DEFAULT_DB_PORT=$(yq eval '.database.port' "$DEFAULT_CONFIG_FILE")
    DEFAULT_DB_HOST=$(yq eval '.database.host' "$DEFAULT_CONFIG_FILE")
fi

# Loop through each file in the configuration directory
for CONFIG_FILE in "$CONFIG_DIR"*.yaml; do
    # Check if the file is a regular file
    if [ -f "$CONFIG_FILE" ]; then
        # Initialize variables with default values
        DB_USER="$DEFAULT_DB_USER"
        DB_PASSWORD="$DEFAULT_DB_PASSWORD"
        DB_NAME="$DEFAULT_DB_NAME"
        DB_PORT="$DEFAULT_DB_PORT"
        DB_HOST="$DEFAULT_DB_HOST"
        
        set -x
        # Read values from the YAML file using yq
        DB_USER=$(yq eval '.database.username' "$CONFIG_FILE")
        if [ "$DB_USER" = "null" ]; then
            DB_USER="$DEFAULT_DB_USER"
        fi

        DB_PASSWORD=$(yq eval '.database.password' "$CONFIG_FILE")
        if [ "$DB_PASSWORD" = "null" ]; then
            DB_PASSWORD="$DEFAULT_DB_PASSWORD"
        fi

        DB_NAME=$(yq eval '.database.database_name' "$CONFIG_FILE")
        if [ "$DB_NAME" = "null" ]; then
            DB_NAME="$DEFAULT_DB_NAME"
        fi

        DB_PORT=$(yq eval '.database.port' "$CONFIG_FILE")
        if [ "$DB_PORT" = "null" ]; then
            DB_PORT="$DEFAULT_DB_PORT"
        fi

        DB_HOST=$(yq eval '.database.host' "$CONFIG_FILE")
        if [ "$DB_HOST" = "null" ]; then
            DB_HOST="$DEFAULT_DB_HOST"
        fi

        set +x
        # Check if Docker should be skipped (only for default.yaml)
        if [[ -z "${SKIP_DOCKER}" && "$CONFIG_FILE" == "configuration/default.yaml" ]]; then
            docker run \
                -e POSTGRES_USER=${DB_USER} \
                -e POSTGRES_PASSWORD=${DB_PASSWORD} \
                -e POSTGRES_DB=${DB_NAME} \
                -p "${DB_PORT}":${DB_PORT} \
                -d postgres \
                postgres -N 1000
        fi

        export PGPASSWORD="${DB_PASSWORD}"
        until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
            echo "Postgres is still unavailable - sleeping"
            sleep 1
        done

        echo "Postgres is up and running on port ${DB_PORT}! -- running migrations now"

        set +x
        DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
        export DATABASE_URL
        sqlx database create
        sqlx migrate run

        echo "${DB_NAME} has been migrated, ready to go"
    fi
    echo "DATABASE_URL=postgres://${DEFAULT_DB_USER}:${DEFAULT_DB_PASSWORD}@${DEFAULT_DB_HOST}:${DEFAULT_DB_PORT}/${DEFAULT_DB_NAME}" > .env
done
