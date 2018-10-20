all: test

test:
	@./run.sh diesel database setup
	@./run.sh cargo test

.PHONY: test
