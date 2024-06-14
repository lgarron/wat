.PHONY: dev
dev:
	cargo run --

.PHONY: install
install:
	cargo install --path .

.PHONY: update-readme-cli-help
update-readme-cli-help:
	bun x readme-cli-help "cargo run -- --help"
