#!/usr/bin/env bash

docker-compose up -d db

POSTGRES_PASSWORD="mysecretpassword"
POSTGRES_USER="rustapp"
POSTGRES_DB="octo-budget-api"


POSTGRES_HOST=$(docker-compose port db 5432)
export DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}/${POSTGRES_DB}"


until docker-compose exec -e PGPASSWORD=$POSTGRES_PASSWORD db sh -c 'psql -U $POSTGRES_USER -d postgres -c "\q"'; do

  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

exec "$@"
