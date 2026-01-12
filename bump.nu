use std/log

# bump the version in Cargo.toml, commit, tag, push, and create a PR
export def main [new_v: oneof<string, nothing>]: nothing -> nothing {
  log set-level 0

  log debug $"pwd=(pwd);"
  let v = ($new_v | default { (input "v=") })

  mut c = open ./Cargo.toml
  $c = ($c | update package.version $v)
  $c | to toml | save --force ./Cargo.toml

  log info $"Bumping version to ($v)..."

  # Fetch latest dependencies
  cargo fetch

  # Stage and commit changes
  git add Cargo.toml Cargo.lock
  git commit -m $"chore\(release): bump version to ($v)"

  # Push dev branch and tags
  log info "Pushing dev branch and tags..."
  git push origin dev --tags

  # Create PR to main
  log info "Creating pull request..."
  gh pr create --title $"chore\(release): bump version to ($v)" --body $"Automated version bump to ($v)" --base main --head dev

  log info $"✅ Version bump to ($v) completed!"
}
