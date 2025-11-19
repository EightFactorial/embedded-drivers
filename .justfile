# Generate the changelog
changelog path="CHANGELOG.md":
    git-cliff --output {{path}}

# Run the clippy linter
clippy:
    cargo clippy --workspace --no-default-features -- -D warnings
    cargo clippy --workspace --features=alloc,defmt -- -D warnings
    cargo clippy --workspace --features=async,defmt -- -D warnings

# Build the project
build mode="release":
    cargo build --profile={{mode}}

# Check all project dependencies
deny:
    cargo deny check all

# Run all workspace tests
test:
    cargo test --workspace --no-default-features
    cargo test --workspace --features=alloc,defmt
    cargo test --workspace --features=async,defmt

# Check all files for typos
typos:
    typos

# Update all dependencies
update:
    cargo update --verbose
    @echo '{{CYAN+BOLD}}note{{NORMAL}}: or, if you have `just` installed, run `just inspect <dep>@<ver>`'

# Show the dependency tree for a specific package
inspect package:
    cargo tree --invert --package={{package}}

# Update and run all checks
pre-commit: (update) (deny) (typos) (clippy) (test)
    @echo '{{GREEN+BOLD}}Success!{{NORMAL}} All checks passed!'
