all: test

RELEASE_BUILD_DIR=./release_build/

build: build_client build_server

build_server:
	cargo build --release --bins

build_client:
	cd reactapp && yarn install && yarn build

ext_cli: ext_bin/diesel ext_bin/wasm-pack

ext_bin/diesel:
	echo "installing diesel_cli ${DIESEL_CLI_VERSION}"
	cargo install diesel_cli --version ${DIESEL_CLI_VERSION} --no-default-features --features postgres
	cp ~/.cargo/bin/diesel ./ext_bin/

ext_bin/wasm-pack:
	echo "installing wasm-pack ${WASM_PACK_CLI_VERSION}"
	cargo install wasm-pack --version ${WASM_PACK_CLI_VERSION}
	cp ~/.cargo/bin/wasm-pack ./ext_bin/

prepare_release:
	rm -rf ${RELEASE_BUILD_DIR}
	mkdir -p ${RELEASE_BUILD_DIR}/reactapp
	cp ./target/release/octo-budget-api ${RELEASE_BUILD_DIR}
	cp -r ./reactapp/build ${RELEASE_BUILD_DIR}/reactapp/

release: build prepare_release
	snap run heroku container:push web -a octo-budget
	snap run heroku container:release web -a octo-budget

docker_release_staging:
	heroku container:login
	heroku container:push web --app octo-budget-staging
	heroku container:release web --app octo-budget-staging
	heroku run diesel migration run --app octo-budget-staging

docker_release_pr:
	heroku container:login
	heroku container:push web --app octo-budget-pr-${TRAVIS_PULL_REQUEST}
	heroku container:release web --app octo-budget-pr-${TRAVIS_PULL_REQUEST}
	heroku run diesel database setup --app octo-budget-pr-${TRAVIS_PULL_REQUEST}
	heroku run './db_seed 2>-' --app octo-budget-pr-${TRAVIS_PULL_REQUEST}

prod_logs:
	snap run heroku logs -t -a octo-budget

cov:
	@./bin/coverage.sh

test: test_db_prepare
	@./run.sh cargo test
	@cd ./octo-budget-frontend && wasm-pack test --node

test_db_prepare:
	@./run.sh diesel database setup

server:
	@RUST_LOG=debug RUST_BACKTRACE=1 ./run.sh cargo run --bin octo-budget-api

psql:
	@docker-compose exec db psql -U rustapp test

redis_cli:
	@docker-compose exec redis redis-cli

.PHONY: test server docker_release_pr
