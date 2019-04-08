FROM ubuntu:18.04

RUN apt-get update && apt-get upgrade -y && apt-get install -y openssl libpq5;

RUN mkdir /app
WORKDIR /app

RUN touch .env

ENV DATABASE_POOL_SIZE 1
ENV LISTEN_IP 0.0.0.0
ENV PORT 8088

ADD ./release_build ./

CMD ["./octo-budget-api"]
