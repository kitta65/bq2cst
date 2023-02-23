.PHONY: setup
setup:
	command -v wasm-pack || cargo install wasm-pack

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: test
test: setup
	cargo fmt --check && \
	cargo test && \
	wasm-pack test --node

.PHONY: build
build: test
	wasm-pack build --target nodejs --scope dr666m1 && \
	cp ./LICENSE* pkg/

.PHONY: publish
publish:
	python -c 'import tomllib,os;f=open("Cargo.toml","rb");data=tomllib.load(f);f.close();assert data["package"]["version"]==os.getenv("GITHUB_REF","").replace("refs/tags/","")' && \
	cd ./pkg && npm publish --access public
