all: test

test:
	@./run.sh diesel database setup
	@./run.sh cargo test

server:
	@./run.sh cargo run --bin octo-budget-api

psql:
	@docker-compose exec db psql -U rustapp test

.PHONY: test server
