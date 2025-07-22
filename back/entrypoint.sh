#!/bin/sh
set -e

until pg_isready -h db -p 5432 -U $POSTGRES_USER; do
  echo "Waiting for postgres..."
  sleep 2
done

cargo sqlx migrate run

exec "$@"
