[windows]
set shell := ["pwsh", "-c"]

export RUST_LOG := 'debug'

_clean-cov:
    cargo llvm-cov clean --workspace
# Run tests with nextest
[arg('ARG', help="additional arguments to pass to cargo nextest, e.g., --features=foo")]
[default]
[group('test')]
test *ARG:
    cargo nextest run {{ ARG }}
# Run tests with all features enabled
[arg('ARG', help="additional arguments to pass to cargo nextest, e.g., --features=foo")]
[group('test')]
all-features-test *ARG:
    cargo hack --feature-powerset nextest run {{ ARG }}
_b-cov: _clean-cov
    cargo hack --feature-powerset llvm-cov --no-report nextest --profile ci
# Generate coverage reports
[group('coverage')]
cov: _b-cov
    cargo llvm-cov report --output-path lcov.info --lcov
# Generate HTML coverage report
[group('coverage')]
html-cov: _b-cov
    cargo llvm-cov report --html
# Release version
[arg('version', pattern='^\d+\.\d+\.\d+|$', help="version to release, e.g., 1.0.0")]
[confirm("Are you sure you want to release version " + version + " ?")]
[script('nu')]
release version:
    # Get the current version from Cargo.toml
    open ./Cargo.toml |{{ if version != "" { ' update package.version ' + version + ' |' } else { '' } }} save ./Cargo.toml --force

    # Fetch latest dependencies
    cargo fetch

    # Stage and commit changes
    git add Cargo.toml Cargo.lock
    git commit -m $"chore\(release): bump version to {{ version }}"

    git push origin dev --tags
    gh pr create --title $"chore\(release): bump version to {{ version }}" --body $"Automated version bump to {{ version }}" --base main --head dev | gh pr merge $in --auto --squash --subject $"chore\(release): bump version to {{ version }}"
