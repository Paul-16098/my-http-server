set windows-shell := ['pwsh', '-c']

export RUST_LOG := 'debug'

_install-dep:
    cargo binstall cargo-all-features cargo-nextest

_clean-cov: _install-dep
    cargo llvm-cov clean --workspace

# Run tests with nextest
test: _install-dep
    cargo nextest run

# Run tests with all features enabled
all-features-test: _install-dep
    cargo all-features -- nextest run

_b-cov: _clean-cov _install-dep
    cargo all-features llvm-cov --no-report nextest --profile ci

# Generate coverage reports
cov: _b-cov
    cargo llvm-cov report --output-path lcov.info --lcov

# Generate HTML coverage report
html-cov: _b-cov
    cargo llvm-cov report --html

# Release version
[arg('version', help="version to release, e.g., 1.0.0")]
[confirm("Are you sure you want to release version?")]
[script('nu')]
release version:
    # Get the current version from Cargo.toml
    open ./Cargo.toml | update package.version {{ version }} | save ./Cargo.toml --force

    # Fetch latest dependencies
    cargo fetch

    # Stage and commit changes
    git add Cargo.toml Cargo.lock
    git commit -m $"chore\(release): bump version to {{ version }}"

    git push origin dev --tags
    gh pr create --title $"chore\(release): bump version to {{ version }}" --body $"Automated version bump to {{ version }}" --base main --head dev | gh pr merge --auto $in --squash --subject $"chore\(release): bump version to {{ version }}"
