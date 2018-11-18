all: test

test:
	@./run.sh diesel database setup
	@./run.sh cargo test

server:
	@./run.sh cargo run --bin octo-budget-api

.PHONY: test server
