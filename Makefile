# Web build of every demo, with an index page linking them. Each demo also runs alone:
#   cargo run --release -p <demo>            (native)
#   cd <demo> && trunk serve --release       (browser)

PORT ?= 8080

.PHONY: site serve test lint

site:            ## build all demos and the index into dist/
	site/build.sh

serve: site      ## build, then serve dist/ at http://localhost:$(PORT)/
	python3 -m http.server --directory dist $(PORT)

test:            ## run every test natively
	cargo test --workspace

lint:            ## clippy, warnings as errors
	cargo clippy --workspace --all-targets -- -D warnings
