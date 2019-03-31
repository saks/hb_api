all: test

RELEASE_BUILD_DIR=./release_build/

build:
	cd reactapp && yarn build
	cargo build --release --bin octo-budget-api
	rm -rf ${RELEASE_BUILD_DIR}
	mkdir -p ${RELEASE_BUILD_DIR}/reactapp
	cp ./target/release/octo-budget-api ${RELEASE_BUILD_DIR}
	cp -r ./reactapp/build ${RELEASE_BUILD_DIR}/reactapp/
	docker build --pull -t octo-budget-builder:latest .

release:
	snap run heroku container:push web -a octo-budget
	snap run heroku container:release web -a octo-budget

test:
	@./run.sh diesel database setup
	@./run.sh cargo test

server:
	@./run.sh cargo run --bin octo-budget-api

psql:
	@docker-compose exec db psql -U rustapp test

redis_cli:
	@docker-compose exec redis redis-cli

.PHONY: test server
