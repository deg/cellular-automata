# Cellular-automata demos. `make` alone lists every target.
#
# Every target that takes a demo reads DEMO (default: sandpile), e.g.
#    make run DEMO=sandpile
# A demo is any top-level directory holding both Cargo.toml and index.html.

DEMOS := $(strip $(foreach d,$(patsubst %/index.html,%,$(wildcard */index.html)),$(if $(wildcard $(d)/Cargo.toml),$(d))))
DEMO  ?= sandpile
PORT  ?= 8080

.DEFAULT_GOAL := help
.PHONY: help demos run web site serve test lint fmt fmt-check check slow timing setup siblings clean check-demo

##@ Help

help: ## Show this help (the default)
	@printf "\033[1;34mUsage:\033[0m make [target] [DEMO=name] [PORT=8080]\n"
	@awk 'BEGIN {FS = ":.*## "} \
	    /^##@/ { printf "\n\033[1;36m%s\033[0m\n", substr($$0, 5) } \
	    /^[a-zA-Z0-9_-]+:.*## / { printf "  \033[1;32m%-10s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
	@printf "\nDemos: $(DEMOS)\n"

demos: ## List the demos
	@printf '%s\n' $(DEMOS)

##@ Run a demo

run: check-demo ## Run DEMO in a native window (release build)
	cargo run --release -p $(DEMO)

web: check-demo ## Serve DEMO alone in the browser, rebuilding on save
	cd $(DEMO) && trunk serve --release --open --port $(PORT)

##@ All demos on the web

site: ## Build every demo and the index page into dist/
	site/build.sh

serve: site ## Build the site, then serve it at http://localhost:PORT/
	@echo "Open http://localhost:$(PORT)/  (Ctrl-C to stop)"
	python3 -m http.server --directory dist $(PORT)

##@ Develop

check: fmt-check lint test ## Everything to pass before a commit: format, lint, tests

test: ## Run every test natively
	cargo test --workspace

lint: ## Clippy over everything, warnings as errors
	cargo clippy --workspace --all-targets -- -D warnings

fmt: ## Format all code
	cargo fmt --all

fmt-check: ## Fail if any code is not formatted
	cargo fmt --all --check

slow: ## Slow checks the READMEs rely on, e.g. Life's gun wrecked at 6,390 (release build)
	cargo test --workspace --release -- --ignored --skip timing

timing: ## Time 100,000- and 1,000,000-grain sandpiles, 4 kinds (about a minute; SANDPILE_TIMING=hex for one)
	cargo test --release -p sandpile timing -- --ignored --nocapture

##@ Toolchain

setup: ## Install the WASM target and trunk
	rustup target add wasm32-unknown-unknown
	cargo install trunk --locked

siblings: ## Build and test ../CPeMulator, ../nscheme, ../llm_wiki (run before and after a Rust update)
	tools/check-rust-siblings.sh

clean: ## Remove build output (target/ and dist/)
	cargo clean
	command rm -rf dist

check-demo:
	@case " $(DEMOS) " in *" $(DEMO) "*) ;; \
	  *) echo "No demo '$(DEMO)'. Demos: $(DEMOS)" >&2; exit 1 ;; esac
