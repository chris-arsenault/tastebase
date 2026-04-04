.PHONY: ci lint fmt typecheck terraform-fmt-check build deploy

ci: lint fmt typecheck terraform-fmt-check

lint:
	cd backend && cargo clippy -- -D warnings
	cd frontend && pnpm exec eslint .

fmt:
	cd backend && cargo fmt -- --check
	cd frontend && pnpm exec prettier --check .

typecheck:
	cd frontend && pnpm exec tsc --noEmit

terraform-fmt-check:
	terraform fmt -check -recursive infrastructure/terraform/

build:
	cd backend && cargo lambda build --release
	cd frontend && pnpm run build

deploy:
	scripts/deploy.sh
