.PHONY: all build build-frontend build-backend clean run dev dev-frontend dev-backend bump-patch bump-minor bump-major

all: build

build: build-frontend build-backend

build-frontend:
	cd frontend && bun install && bun run build

build-backend:
	cargo build --release

clean:
	cargo clean
	rm -rf frontend/dist
	rm -rf frontend/node_modules

run: build
	./target/release/archa

dev:
	@echo "--------------------------------------------------------"
	@echo "  Archa Development Mode"
	@echo "  - Backend (API): http://localhost:3000"
	@echo "  - Frontend (HMR): http://localhost:5173  <-- USE THIS"
	@echo "--------------------------------------------------------"
	@make -j 2 dev-backend dev-frontend

dev-frontend:
	cd frontend && bun run dev

dev-backend:
	cargo run

publish: build
	cargo publish --allow-dirty

bump-patch:
	@old=$$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'); \
	major=$$(echo $$old | cut -d. -f1); \
	minor=$$(echo $$old | cut -d. -f2); \
	patch=$$(echo $$old | cut -d. -f3); \
	new="$$major.$$minor.$$((patch+1))"; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" Cargo.toml; \
	sed -i '' "s/v$$old/v$$new/" frontend/src/App.tsx; \
	echo "$$old → $$new"

bump-minor:
	@old=$$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'); \
	major=$$(echo $$old | cut -d. -f1); \
	minor=$$(echo $$old | cut -d. -f2); \
	new="$$major.$$((minor+1)).0"; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" Cargo.toml; \
	sed -i '' "s/v$$old/v$$new/" frontend/src/App.tsx; \
	echo "$$old → $$new"

bump-major:
	@old=$$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'); \
	major=$$(echo $$old | cut -d. -f1); \
	new="$$((major+1)).0.0"; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" Cargo.toml; \
	sed -i '' "s/v$$old/v$$new/" frontend/src/App.tsx; \
	echo "$$old → $$new"
