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
