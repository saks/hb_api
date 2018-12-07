#!/usr/bin/env bash

docker-compose up -d db redis

redis_port=6379


POSTGRES_PASSWORD="mysecretpassword"
POSTGRES_USER="rustapp"
POSTGRES_DB="test"
# POSTGRES_PASSWORD=""
# POSTGRES_USER="postgres"
# POSTGRES_DB="postgres"


POSTGRES_HOST=$(docker-compose port db 5432)
redis_host=$(docker-compose port redis $redis_port)
# POSTGRES_HOST="172.18.0.2"
export DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}/${POSTGRES_DB}"
export REDIS_URL="redis://${redis_host}:${redis_port}"


until docker-compose exec -e PGPASSWORD=$POSTGRES_PASSWORD db sh -c 'psql -U $POSTGRES_USER -d postgres -c "\q"'; do

  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

export RUST_TEST_THREADS=1

exec "$@"
