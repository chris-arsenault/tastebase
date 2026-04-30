.PHONY: ci lint fmt typecheck test terraform-fmt-check build deploy

# Mirrors the shared workflow at chris-arsenault/ahara/.github/workflows/ci.yml.
# Keep these targets in sync with that workflow.
ci: lint fmt typecheck test terraform-fmt-check

lint:
	cd backend && CARGO_TARGET_DIR=target-clippy cargo clippy --release -- -D warnings -W clippy::cognitive_complexity
	cd frontend && pnpm exec eslint .

fmt:
	cd backend && cargo fmt -- --check
	cd frontend && pnpm exec prettier --check .

typecheck:
	cd frontend && pnpm exec tsc --noEmit

test:
	cd backend && CARGO_TARGET_DIR=target-cov cargo test --release --lib
	cd frontend && if pnpm exec vitest --help > /dev/null 2>&1; then pnpm exec vitest run; fi

terraform-fmt-check:
	terraform fmt -check -recursive infrastructure/terraform/

build:
	cd backend && cargo lambda build --release
	cd frontend && pnpm run build

deploy:
	scripts/deploy.sh
