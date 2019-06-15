FROM ubuntu:16.04

RUN apt-get update && apt-get upgrade -y && apt-get install -y openssl libpq5;

RUN mkdir -p /app/reactapp/build
WORKDIR /app

RUN touch .env

ENV DATABASE_POOL_SIZE 1
ENV LISTEN_IP 0.0.0.0
ENV PORT 8088

ADD ./target/release/octo-budget-api ./
ADD ./reactapp/build/ ./reactapp/build

CMD ["./octo-budget-api"]
