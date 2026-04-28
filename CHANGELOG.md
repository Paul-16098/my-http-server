<!-- markdownlint-disable-file MD034 MD012 MD024 -->

# CHANGELOG

## \[unreleased\]

### Features

- *release*: Automate merging of version bump pull requests by @Paul-16098

- *github_emojis*: Add clear cache option for emoji management by @Paul-16098

- *cli*: Add verbosity flag support for enhanced logging control by @Paul-16098

- *cli*: Add verbosity flag for enhanced configuration control by @Paul-16098

- *tests*: Add insta snapshots for md2html tests by @Paul-16098

- *tests*: Add snapshots for md2html with links, images, and tables by @Paul-16098

- *cli*: Add shell completion support for bash, zsh, fish, powershell, elvish, and nushell by @Paul-16098


### Bug Fixes

- Reset to main by @Paul-16098

- *dist*: Remove duplicate target platform entry by @Paul-16098

- *release*: Add missing confirm and script directives for version release by @Paul-16098

- *release*: Remove unnecessary --squash option from PR merge command by @Paul-16098

- *release*: Add --squash option to PR merge command by @Paul-16098

- *release*: Remove unnecessary --auto option from PR merge command by @Paul-16098

- *release*: Change merge command to use --auto option by @Paul-16098

- *cli*: Clarify default config file path description by @Paul-16098

- *parser*: Remove unused mutable variable in md2html function by @Paul-16098

- *tests*: Add argument support for nextest and all-features-test commands by @Paul-16098

- *justfile*: Add coverage group and refine version argument pattern by @Paul-16098

- *config*: Improve default configuration loading and simplify test fixture setup by @Paul-16098

- *tasks*: Migrate from cargo to vscode-just for task management by @Paul-16098

- *parser*: Allow unused mutable variable for compatibility with non-github_emojis feature by @Paul-16098

- *auto-merge*: Update regex pattern for taiki-e/install-action PRs to v2.75.x by @Paul-16098

- *cofg*: Improve warning message for missing config file by @Paul-16098

- *request*: Handle non-existent request paths with 404 response by @Paul-16098

- *cli*: Add value hints for file and directory paths in argument parsing by @Paul-16098

- *cli*: Improve Nushell completion script generation and error handling by @Paul-16098

- *tests*: Correct conditional compilation for path traversal tests on Windows by @Paul-16098

- *tests*: Update path traversal tests to include NOT_FOUND status by @Paul-16098

- *deps*: Update windows-sys to version 0.60.2 and rustls-webpki to version 0.103.12 by @Paul-16098

- *build*: Include additional git files for change detection by @Paul-16098

### <details><summary>Deps</summary>


- Update taiki-e/install-action action to v2.73.0 (#179) by @renovate\[bot] in [#179](https://github.com/Paul-16098/my-http-server/pull/179)

- Update actions/github-script action to v9 (#182) by @renovate\[bot] in [#182](https://github.com/Paul-16098/my-http-server/pull/182)

- Update rand package to version 0.9.3 by @Paul-16098

- Update taiki-e/install-action action to v2.75.3 (#180) by @renovate\[bot] in [#180](https://github.com/Paul-16098/my-http-server/pull/180)

- Update taiki-e/install-action action to v2.75.5 (#183) by @renovate\[bot] in [#183](https://github.com/Paul-16098/my-http-server/pull/183)

- Update rust crate rustls to v0.23.38 (#185) by @renovate\[bot] in [#185](https://github.com/Paul-16098/my-http-server/pull/185)

- Update taiki-e/install-action action to v2.75.9 (#184) by @renovate\[bot] in [#184](https://github.com/Paul-16098/my-http-server/pull/184)

- Update actions-rust-lang/setup-rust-toolchain action to v1.16.0 (#186) by @renovate\[bot] in [#186](https://github.com/Paul-16098/my-http-server/pull/186)

- Add cargo-llvm-cov to installation in justfile and workflow by @Paul-16098

- Update taiki-e/install-action action to v2.75.12 (#187) by @renovate\[bot] in [#187](https://github.com/Paul-16098/my-http-server/pull/187)

- Update taiki-e/install-action action to v2.75.13 (#188) by @renovate\[bot] in [#188](https://github.com/Paul-16098/my-http-server/pull/188)

- Update taiki-e/install-action action to v2.75.14 (#189) by @renovate\[bot] in [#189](https://github.com/Paul-16098/my-http-server/pull/189)

- Update rust crate clap to v4.6.1 (#190) by @renovate\[bot] in [#190](https://github.com/Paul-16098/my-http-server/pull/190)

- Update taiki-e/install-action action to v2.75.15 (#191) by @renovate\[bot] in [#191](https://github.com/Paul-16098/my-http-server/pull/191)

- Update taiki-e/install-action action to v2.75.16 (#194) by @renovate\[bot] in [#194](https://github.com/Paul-16098/my-http-server/pull/194)

- Update taiki-e/install-action action to v2.75.17 (#195) by @renovate\[bot] in [#195](https://github.com/Paul-16098/my-http-server/pull/195)

- Update taiki-e/install-action action to v2.75.18 (#196) by @renovate\[bot] in [#196](https://github.com/Paul-16098/my-http-server/pull/196)

- Bump actix-http in the cargo group across 1 directory (#197) by @dependabot\[bot] in [#197](https://github.com/Paul-16098/my-http-server/pull/197)

- Bump rustls-webpki from 0.103.12 to 0.103.13 in the cargo group across 1 directory (#199) by @dependabot\[bot] in [#199](https://github.com/Paul-16098/my-http-server/pull/199)

- Update taiki-e/install-action action to v2.75.19 (#200) by @renovate\[bot] in [#200](https://github.com/Paul-16098/my-http-server/pull/200)

- Update taiki-e/install-action action to v2.75.20 (#201) by @renovate\[bot] in [#201](https://github.com/Paul-16098/my-http-server/pull/201)

- Update rust crate rustls to v0.23.39 (#198) by @renovate\[bot] in [#198](https://github.com/Paul-16098/my-http-server/pull/198)

- Update taiki-e/install-action action to v2.75.21 (#202) by @renovate\[bot] in [#202](https://github.com/Paul-16098/my-http-server/pull/202)

- Update rust crate rustls-pki-types to v1.14.1 (#203) by @renovate\[bot] in [#203](https://github.com/Paul-16098/my-http-server/pull/203)

</details>

### Other

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098


### Refactor

- *tests*: Update insta snapshots for md2html tests and remove obsolete files by @Paul-16098


### Miscellaneous Tasks

- *workflow*: Add permissions section to cargo-push workflow by @Paul-16098


## \[4.1.7\] - 2026-04-05

### Features

- *tests*: Add nextest support for running tests by @Paul-16098

- *release*: Add automated version bump script in justfile by @Paul-16098


### Other

- Merge pull request #168 from Paul-16098/dev<br><br>chore(release): bump version to 4.1.7 by @Paul-16098 in [#168](https://github.com/Paul-16098/my-http-server/pull/168)


### Miscellaneous Tasks

- *bump*: Remove automated version bump script by @Paul-16098

- *workflow*: Remove permissions for contents in cargo-push job by @Paul-16098


## \[4.1.6\] - 2026-04-05

### Features

- *config*: Enhance configuration caching and XDG path initialization by @Paul-16098

- *docs*: 更新架構與資料流文檔，增強配置流程與初始化序列說明 by @Paul-16098

- *tests*: 添加 Codecov 结果上传步骤并更新 nextest 配置 by @Paul-16098

- *workflows*: Add cargo-push workflow and integrate with release process by @Paul-16098

- *workflow*: Add auto-merge PR workflow with checks and conditions by @Paul-16098

- *workflow*: Add scheduled trigger for auto-merge PR workflow by @Paul-16098

- *tests*: Migrate from cargo-make to just for test and coverage tasks by @Paul-16098


### Bug Fixes

- *changelog*: Escape brackets in contributor username by @Paul-16098

- *request*: Simplify URL handling in main_req function by @Paul-16098

- *request*: Handle root path correctly and improve error logging by @Paul-16098

- *tests*: Add debug logging for coverage and test tasks by @Paul-16098

- *workflow*: Update toml-action version to specific commit by @Paul-16098

- *request*: Improve error logging for path resolution failures by @Paul-16098

- *request*: Improve error logging for path resolution failures by @Paul-16098

- *request*: Improve error handling for path traversal attacks by @Paul-16098

- Delete lcov.info by @Paul-16098

- *security*: Enhance path traversal protection and update test assertions by @Paul-16098

- *tests*: 修复 nextest 命令中的参数格式，确保正确使用 --profile by @Paul-16098

- *tests*: 更新 Codecov 测试结果上传步骤，修复文件参数和报告类型 by @Paul-16098

- *tests*: 更新 Codecov 上传的测试结果文件路径至 ./target/nextest/ci/junit.xml by @Paul-16098

- *deps*: Update rust crate wax to 0.7.0 (#94) by @renovate\[bot] in [#94](https://github.com/Paul-16098/my-http-server/pull/94)

- *server*: Improve logging and handle TLS configuration errors by @Paul-16098

- *workflow*: Add conditional check before committing changelog updates by @Paul-16098

- Fix for code scanning alert no. 36: Workflow does not contain permissions by @Paul-16098

- *workflow*: Allow skipped checks in auto-merge condition by @Paul-16098

- *workflow*: Simplify check verification logic for PR merges by @Paul-16098

- *workflow*: Update PR filter pattern for renovate\[bot] to match specific version by @Paul-16098

- *changelog*: Escape brackets in commit messages for proper rendering by @Paul-16098

- *cli*: Update public path argument parsing to remove redundant flag by @Paul-16098

- *request*: Percent-decode request path to handle encoded filesystem names by @Paul-16098

### <details><summary>Deps</summary>


- Update taiki-e/install-action action to v2.67.13 (#87) by @renovate\[bot] in [#87](https://github.com/Paul-16098/my-http-server/pull/87)

- Update rust crate thiserror to v2.0.18 (#89) by @renovate\[bot] in [#89](https://github.com/Paul-16098/my-http-server/pull/89)

- Update rust crate rustls-pki-types to v1.14.0 (#88) by @renovate\[bot] in [#88](https://github.com/Paul-16098/my-http-server/pull/88)

- Update taiki-e/install-action action to v2.67.14 (#91) by @renovate\[bot] in [#91](https://github.com/Paul-16098/my-http-server/pull/91)

- Update taiki-e/install-action action to v2.67.19 (#93) by @renovate\[bot] in [#93](https://github.com/Paul-16098/my-http-server/pull/93)

- Bump bytes in the cargo group across 1 directory (#95) by @dependabot\[bot] in [#95](https://github.com/Paul-16098/my-http-server/pull/95)

- Update rust crate clap to v4.5.56 (#92) by @renovate\[bot] in [#92](https://github.com/Paul-16098/my-http-server/pull/92)

- Update rust crate actix-files to v0.6.10 \[security] (#99) by @renovate\[bot] in [#99](https://github.com/Paul-16098/my-http-server/pull/99)

- Update taiki-e/install-action action to v2.67.20 (#96) by @renovate\[bot] in [#96](https://github.com/Paul-16098/my-http-server/pull/96)

- Update rust crate clap to v4.5.57 (#97) by @renovate\[bot] in [#97](https://github.com/Paul-16098/my-http-server/pull/97)

- Update rust crate time to v0.3.47 \[security] (#100) by @Paul-16098

- Update rust crate ureq to v3.2.0 (#101) by @renovate\[bot] in [#101](https://github.com/Paul-16098/my-http-server/pull/101)

- Update taiki-e/install-action action to v2.67.22 (#102) by @renovate\[bot] in [#102](https://github.com/Paul-16098/my-http-server/pull/102)

- Update taiki-e/install-action action to v2.67.26 (#103) by @renovate\[bot] in [#103](https://github.com/Paul-16098/my-http-server/pull/103)

- Update rust crate tempfile to v3.25.0 (#105) by @renovate\[bot] in [#105](https://github.com/Paul-16098/my-http-server/pull/105)

- Update rust crate clap to v4.5.58 (#106) by @renovate\[bot] in [#106](https://github.com/Paul-16098/my-http-server/pull/106)

- Update rust crate env_logger to v0.11.9 (#107) by @renovate\[bot] in [#107](https://github.com/Paul-16098/my-http-server/pull/107)

- Update rust crate clap to v4.5.59 (#108) by @renovate\[bot] in [#108](https://github.com/Paul-16098/my-http-server/pull/108)

- Update taiki-e/install-action action to v2.68.0 (#104) by @renovate\[bot] in [#104](https://github.com/Paul-16098/my-http-server/pull/104)

- Update rust crate actix-web to v4.13.0 (#110) by @renovate\[bot] in [#110](https://github.com/Paul-16098/my-http-server/pull/110)

- Update taiki-e/install-action action to v2.68.6 (#109) by @renovate\[bot] in [#109](https://github.com/Paul-16098/my-http-server/pull/109)

- Update rust crate clap to v4.5.60 (#111) by @renovate\[bot] in [#111](https://github.com/Paul-16098/my-http-server/pull/111)

- Update rust crate tempfile to v3.26.0 (#113) by @renovate\[bot] in [#113](https://github.com/Paul-16098/my-http-server/pull/113)

- Update rust crate rustls to v0.23.37 (#114) by @renovate\[bot] in [#114](https://github.com/Paul-16098/my-http-server/pull/114)

- Update taiki-e/install-action action to v2.68.12 (#112) by @renovate\[bot] in [#112](https://github.com/Paul-16098/my-http-server/pull/112)

- Update taiki-e/install-action action to v2.68.13 (#115) by @renovate\[bot] in [#115](https://github.com/Paul-16098/my-http-server/pull/115)

- Update taiki-e/install-action action to v2.68.14 (#116) by @renovate\[bot] in [#116](https://github.com/Paul-16098/my-http-server/pull/116)

- Update taiki-e/install-action action to v2.68.15 (#117) by @renovate\[bot] in [#117](https://github.com/Paul-16098/my-http-server/pull/117)

- Update actions-rust-lang/setup-rust-toolchain action to v1.15.3 (#118) by @renovate\[bot] in [#118](https://github.com/Paul-16098/my-http-server/pull/118)

- Update taiki-e/install-action action to v2.68.16 (#119) by @renovate\[bot] in [#119](https://github.com/Paul-16098/my-http-server/pull/119)

- Update actions/github-script action to v8 (#120) by @renovate\[bot] in [#120](https://github.com/Paul-16098/my-http-server/pull/120)

- Update taiki-e/install-action action to v2.68.17 (#121) by @renovate\[bot] in [#121](https://github.com/Paul-16098/my-http-server/pull/121)

- Update taiki-e/install-action action to v2.68.18 (#122) by @renovate\[bot] in [#122](https://github.com/Paul-16098/my-http-server/pull/122)

- Update taiki-e/install-action action to v2.68.19 (#123) by @renovate\[bot] in [#123](https://github.com/Paul-16098/my-http-server/pull/123)

- Update taiki-e/install-action action to v2.68.20 (#125) by @renovate\[bot] in [#125](https://github.com/Paul-16098/my-http-server/pull/125)

- Update taiki-e/install-action action to v2.68.21 (#126) by @renovate\[bot] in [#126](https://github.com/Paul-16098/my-http-server/pull/126)

- Update taiki-e/install-action action to v2.68.22 (#127) by @renovate\[bot] in [#127](https://github.com/Paul-16098/my-http-server/pull/127)

- Update taiki-e/install-action action to v2.68.23 (#128) by @renovate\[bot] in [#128](https://github.com/Paul-16098/my-http-server/pull/128)

- Update taiki-e/install-action action to v2.68.25 (#129) by @renovate\[bot] in [#129](https://github.com/Paul-16098/my-http-server/pull/129)

- Update taiki-e/install-action action to v2.68.26 (#130) by @renovate\[bot] in [#130](https://github.com/Paul-16098/my-http-server/pull/130)

- Update rust crate tempfile to v3.27.0 (#131) by @renovate\[bot] in [#131](https://github.com/Paul-16098/my-http-server/pull/131)

- Update taiki-e/install-action action to v2.68.27 (#133) by @renovate\[bot] in [#133](https://github.com/Paul-16098/my-http-server/pull/133)

- Update rust crate clap to v4.6.0 (#134) by @renovate\[bot] in [#134](https://github.com/Paul-16098/my-http-server/pull/134)

- Update rust crate config to v0.15.21 (#132) by @renovate\[bot] in [#132](https://github.com/Paul-16098/my-http-server/pull/132)

- Update taiki-e/install-action action to v2.68.28 (#135) by @renovate\[bot] in [#135](https://github.com/Paul-16098/my-http-server/pull/135)

- Update taiki-e/install-action action to v2.68.29 (#136) by @renovate\[bot] in [#136](https://github.com/Paul-16098/my-http-server/pull/136)

- Update actions-rust-lang/setup-rust-toolchain action to v1.15.4 (#137) by @renovate\[bot] in [#137](https://github.com/Paul-16098/my-http-server/pull/137)

- Update taiki-e/install-action action to v2.68.31 (#138) by @renovate\[bot] in [#138](https://github.com/Paul-16098/my-http-server/pull/138)

- Update taiki-e/install-action action to v2.68.32 (#139) by @renovate\[bot] in [#139](https://github.com/Paul-16098/my-http-server/pull/139)

- Update taiki-e/install-action action to v2.68.33 (#140) by @renovate\[bot] in [#140](https://github.com/Paul-16098/my-http-server/pull/140)

- Update taiki-e/install-action action to v2.68.34 (#141) by @renovate\[bot] in [#141](https://github.com/Paul-16098/my-http-server/pull/141)

- Update taiki-e/install-action action to v2.68.35 (#142) by @renovate\[bot] in [#142](https://github.com/Paul-16098/my-http-server/pull/142)

- Bump rustls-webpki in the cargo group across 1 directory (#144) by @dependabot\[bot] in [#144](https://github.com/Paul-16098/my-http-server/pull/144)

- Update codecov/codecov-action action to v5.5.3 (#145) by @renovate\[bot] in [#145](https://github.com/Paul-16098/my-http-server/pull/145)

- Update taiki-e/install-action action to v2.68.36 (#146) by @renovate\[bot] in [#146](https://github.com/Paul-16098/my-http-server/pull/146)

- Update rust crate config to v0.15.22 (#143) by @renovate\[bot] in [#143](https://github.com/Paul-16098/my-http-server/pull/143)

- Update taiki-e/install-action action to v2.69.2 (#147) by @renovate\[bot] in [#147](https://github.com/Paul-16098/my-http-server/pull/147)

- Update taiki-e/install-action action to v2.69.3 (#148) by @renovate\[bot] in [#148](https://github.com/Paul-16098/my-http-server/pull/148)

- Update taiki-e/install-action action to v2.69.4 (#149) by @renovate\[bot] in [#149](https://github.com/Paul-16098/my-http-server/pull/149)

- Update taiki-e/install-action action to v2.69.5 (#150) by @renovate\[bot] in [#150](https://github.com/Paul-16098/my-http-server/pull/150)

- Update rust crate ureq to v3.2.1 (#151) by @renovate\[bot] in [#151](https://github.com/Paul-16098/my-http-server/pull/151)

- Update taiki-e/install-action action to v2.69.6 (#153) by @renovate\[bot] in [#153](https://github.com/Paul-16098/my-http-server/pull/153)

- Update rust crate ureq to v3.3.0 (#152) by @renovate\[bot] in [#152](https://github.com/Paul-16098/my-http-server/pull/152)

- Update taiki-e/install-action action to v2.69.7 (#154) by @renovate\[bot] in [#154](https://github.com/Paul-16098/my-http-server/pull/154)

- Update rust crate env_logger to v0.11.10 (#155) by @renovate\[bot] in [#155](https://github.com/Paul-16098/my-http-server/pull/155)

- Update taiki-e/install-action action to v2.69.8 (#156) by @renovate\[bot] in [#156](https://github.com/Paul-16098/my-http-server/pull/156)

- Update taiki-e/install-action action to v2.69.9 (#157) by @renovate\[bot] in [#157](https://github.com/Paul-16098/my-http-server/pull/157)

- Update taiki-e/install-action action to v2.69.10 (#158) by @renovate\[bot] in [#158](https://github.com/Paul-16098/my-http-server/pull/158)

- Update taiki-e/install-action action to v2.69.11 (#161) by @renovate\[bot] in [#161](https://github.com/Paul-16098/my-http-server/pull/161)

- Update codecov/codecov-action action to v6 (#160) by @renovate\[bot] in [#160](https://github.com/Paul-16098/my-http-server/pull/160)

- Update taiki-e/install-action action to v2.69.12 (#162) by @renovate\[bot] in [#162](https://github.com/Paul-16098/my-http-server/pull/162)

- Update taiki-e/install-action action to v2.69.13 (#163) by @renovate\[bot] in [#163](https://github.com/Paul-16098/my-http-server/pull/163)

- Update taiki-e/install-action action to v2.69.14 (#164) by @renovate\[bot] in [#164](https://github.com/Paul-16098/my-http-server/pull/164)

- Update rust crate markdown-ppp to v2.9.2 (#166) by @renovate\[bot] in [#166](https://github.com/Paul-16098/my-http-server/pull/166)

- Update taiki-e/install-action action to v2.71.1 (#165) by @renovate\[bot] in [#165](https://github.com/Paul-16098/my-http-server/pull/165)

</details>

### Other

- Merge pull request #86 from Paul-16098/renovate/taiki-e-install-action-2.x<br><br>chore(deps): update taiki-e/install-action action to v2.66.4 by @renovate\[bot] in [#86](https://github.com/Paul-16098/my-http-server/pull/86)

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Merge branches 'dev' and 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Merge pull request #167 from Paul-16098/dev<br><br>chore(release): bump version to 4.1.6 by @Paul-16098 in [#167](https://github.com/Paul-16098/my-http-server/pull/167)


### Refactor

- *tests*: Consolidate test setup into reusable support module by @Paul-16098


### Documentation

- *copilot-instructions*: Update project overview and architecture details by @Paul-16098


### Styling

- Use tab by @Paul-16098


### Testing

- *security*: Add debug output for path traversal test response by @Paul-16098

- *security*: Add response body output for path traversal test by @Paul-16098

- *security*: Add response body output for path traversal test by @Paul-16098


### Miscellaneous Tasks

- *gitignore*: Exclude Cargo.lock from being ignored by @Paul-16098

- *workflows*: Remove branch filters from push event in test workflow by @Paul-16098

- *workflows*: Remove scheduled trigger from Docker publish workflow by @Paul-16098

- *docs*: Remove outdated documentation files including developer guide, IP filter implementation summary, key functions, performance cache, request flow, and XDG config example by @Paul-16098

- *docker*: Remove Docker-related files including Dockerfile, .dockerignore, and docker-compose.yml by @Paul-16098

- *workflows*: Update changelog generation and back-merge process by @Paul-16098

- *workflows*: Remove Docker publish workflow and update test conditions by @Paul-16098

- *workflows*: Update push command in changelog generation workflow by @Paul-16098

- Update changelog by @Paul-16098

- *changelog*: Update changelog template for better formatting and links by @Paul-16098

- Update changelog by @Paul-16098

- *changelog*: Update header formatting and bullet style for contributors by @Paul-16098

- Update changelog by @Paul-16098

- Update changelog by @Paul-16098

- *gitattributes*: Add merge strategy for CHANGELOG.md by @Paul-16098

- Update changelog by @Paul-16098

- Update changelog by @Paul-16098

- *changelog*: Update commit message format for changelog updates by @Paul-16098

- *workflow*: Add branches to trigger changelog generation by @Paul-16098

- *Makefile*: Add --locked flag to cargo install command by @Paul-16098

- *changelog*: Skip changelog updates for specific commit message by @Paul-16098

- *changelog*: Add GITHUB_TOKEN environment variable for changelog generation by @Paul-16098

- *workflow*: Update trigger to only activate on version tag pushes by @Paul-16098

- *workflow*: Add cron schedule for changelog generation by @Paul-16098

- *workflow*: Remove comment from cron schedule by @Paul-16098

- *request*: Remove unused import for process exit by @Paul-16098

- *rust*: Simplify caching by enabling all crates caching by @Paul-16098

- *renovate*: 添加 minimumReleaseAge 配置 by @Paul-16098

- *changelog*: Enhance changelog formatting for dependency groups by @Paul-16098

- *api*: Update swagger-ui dependencies to version 5.31.0 by @Paul-16098

- *vscode*: Update extensions and settings for improved development experience by @Paul-16098

- *tests*: Comment out clippy lint allows in test module by @Paul-16098

- *ci*: Update cargo-dist version to 0.31.0 by @Paul-16098

- *workflows*: Remove deprecated Docker publish workflow by @Paul-16098

- *pre-commit*: Add configuration for Rust formatting hook by @Paul-16098

- *workflow*: Remove unused docker-test job from test.yml by @Paul-16098

- Update PR title pattern for renovate\[bot] to v2.69 by @Paul-16098

- *dist*: Reorder target platforms in configuration by @Paul-16098

- *workflow*: Update tool configuration for cargo-binstall in test.yml by @Paul-16098

- *tests*: Allow clippy warnings for unwrap and expect in test module by @Paul-16098



- @dependabot[bot] made their first contribution in #144## \[4.1.5\] - 2026-01-14

### Features

- *tests*: Reorganize Makefile tasks for better coverage reporting and add lcov output by @Paul-16098

- *vscode*: Add extensions.json for recommended VSCode extensions by @Paul-16098

- *tests*: Add coverage report generation and commit step by @Paul-16098

- *tests*: Add git push step for coverage report submission by @Paul-16098

- *vscode*: Update settings to exclude lcov.info file by @Paul-16098

- *workflows*: Update actions versions and add caching for cargo dependencies by @Paul-16098

- *config*: Add no_xdg parameter to new_layered and init_global methods by @Paul-16098

- *tests*: Update config in integration tests to specify template path by @Paul-16098

- *tests*: Add --no-fail-fast option to nextest command by @Paul-16098

- *coverage*: Update test workflow to run coverage and upload reports to Codecov by @Paul-16098

- *tests*: Enhance test coverage and refactor test cases for CLI and configuration by @Paul-16098

- *tests*: Enhance test configuration initialization for consistency and security by @Paul-16098


### Bug Fixes

- *tests*: Clean up XDG config files in test environment to prevent CI interference by @Paul-16098

- *integration*: Remove unnecessary blank line in with_cwd_lock function by @Paul-16098

- *integration*: Add error handling for writing emojis.json in with_cwd_lock function by @Paul-16098

- *tests*: Remove unnecessary blank lines and add git configuration for coverage commits by @Paul-16098

- *request*: Deny access to .gitignore and return 404 by @Paul-16098

- *request*: Deny access to sensitive files including .gitignore, cofg.yaml, and Cargo.toml by @Paul-16098

- *request*: Deny access to restricted files and improve test assertions by @Paul-16098

- *request*: Improve logging for access to restricted files by @Paul-16098

- *cov*: Remove unnecessary '--open' flag from HTML report command by @Paul-16098

### <details><summary>Deps</summary>


- Update taiki-e/install-action action to v2.66.2 (#82) by @renovate\[bot] in [#82](https://github.com/Paul-16098/my-http-server/pull/82)

- Update docker/metadata-action action to v5.10.0 (#78) by @renovate\[bot] in [#78](https://github.com/Paul-16098/my-http-server/pull/78)

- Update actions/cache action to v5 (#80) by @renovate\[bot] in [#80](https://github.com/Paul-16098/my-http-server/pull/80)

- Update docker/build-push-action action to v6.18.0 (#76) by @renovate\[bot] in [#76](https://github.com/Paul-16098/my-http-server/pull/76)

- Update sigstore/cosign-installer action to v4 (#81) by @renovate\[bot] in [#81](https://github.com/Paul-16098/my-http-server/pull/81)

- Update docker/login-action action to v3.6.0 (#77) by @renovate\[bot] in [#77](https://github.com/Paul-16098/my-http-server/pull/77)

- Update taiki-e/install-action action to v2.66.3 (#84) by @renovate\[bot] in [#84](https://github.com/Paul-16098/my-http-server/pull/84)

</details>

### Other

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Initial plan by @Copilot

- Create comprehensive test infrastructure for my-http-server<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Fix remaining test failures and ensure all tests pass<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Fix integration test hangs by initializing config before service creation<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Fix test race condition by using Once for global initialization<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Merge branch 'dev' into copilot/add-test-coverage by @Paul-16098

- Merge pull request #83 from Paul-16098/copilot/add-test-coverage<br><br>Add comprehensive test infrastructure (61 tests across 4 modules) by @Paul-16098 in [#83](https://github.com/Paul-16098/my-http-server/pull/83)

- Merge pull request #85 from Paul-16098/dev<br><br>chore(release): bump version to 4.1.5 by @Paul-16098 in [#85](https://github.com/Paul-16098/my-http-server/pull/85)


### Refactor

- *makefile*: Rename tasks for coverage and update task definitions by @Paul-16098

- *test*: Remove all test files for parser, request, security, templating, and test module by @Paul-16098


### Testing

- *main*: Add comprehensive tests for versioning, security, and request handling by @Paul-16098

- *integration, parser, request, security*: Improve test coverage and stability with sequential request handling and enhanced config initialization by @Paul-16098

- *md2html*: Update assertion to check full HTML output for basic markdown by @Paul-16098


### Miscellaneous Tasks

- *code-changes*: Update code structure for improved readability and maintainability by @Paul-16098

- *workflows*: Update action versions in Security and test workflows by @Paul-16098


## \[4.1.4\] - 2026-01-12

### Bug Fixes

- *changelog*: Enhance commit message formatting with contributor info by @Paul-16098


### Other

- Merge branch 'main' into dev by @Paul-16098

- Bump version to 4.1.3 by @Paul-16098

- Merge pull request #73 from Paul-16098/dev<br><br>v4.1.2 by @Paul-16098 in [#73](https://github.com/Paul-16098/my-http-server/pull/73)

- Merge branch 'main' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- *release*: Bump version to 4.1.3 by @Paul-16098

- Merge pull request #74 from Paul-16098/dev<br><br>chore(release): bump version to 4.1.4 by @Paul-16098 in [#74](https://github.com/Paul-16098/my-http-server/pull/74)

- Update .github/workflows/release-tag-and-backmerge.yml<br><br>Co-authored-by: Copilot <175728472+Copilot@users.noreply.github.com> by @Paul-16098

- Merge pull request #75 from Paul-16098/dev<br><br>v4.1.4 by @Paul-16098 in [#75](https://github.com/Paul-16098/my-http-server/pull/75)


## \[4.1.1\] - 2026-01-10

### Bug Fixes

- *version*: Update package version to 4.1.2 and format settings.json by @Paul-16098

- *workflows*: Update changelog workflow references and inputs by @Paul-16098


### Other

- Merge pull request #67 from Paul-16098:dev<br><br>Add GitHub Attestations and update my-http-server version by @Paul-16098 in [#67](https://github.com/Paul-16098/my-http-server/pull/67)


## \[4.1.2\] - 2026-01-12

### Features

- Feat(workflow): add GitHub Attestations and cargo-auditable support; update permissions in release.yml<br>fix(package): enable license sidecar in Wix installer and update description in Cargo.toml<br>chore(dist): enable GitHub Attestations and cargo-auditable in dist-workspace.toml by @Paul-16098

- *workflow*: Add rust-cache action and update pr-run-mode in dist workspace by @Paul-16098

- *changelog*: Add configuration for generating changelog and CI workflow by @Paul-16098


### Bug Fixes

- *version*: Update my-http-server version to 4.1.1 by @Paul-16098

- *description*: Update description in Cargo.toml and main.wxs for clarity by @Paul-16098

- *renovate*: Correct ignorePaths formatting in renovate.json by @Paul-16098

- *renovate*: Update ignorePaths formatting in renovate.json by @Paul-16098

- *changelog*: Remove unnecessary verbosity from changelog generation args by @Paul-16098

- *changelog*: Simplify first contribution message in changelog template by @Paul-16098

- *changelog*: Update changelog generation to use OUTPUT environment variable by @Paul-16098

- *changelog*: Update changelog generation to use git-cliff directly by @Paul-16098

### <details><summary>Deps</summary>


- Update taiki-e/install-action action to v2.66.1 (#72) by @renovate\[bot] in [#72](https://github.com/Paul-16098/my-http-server/pull/72)

</details>

### Other

- Merge remote-tracking branch 'origin/main' into dev by @Paul-16098

- Merge pull request #66 from Paul-16098/renovate/taiki-e-install-action-2.x<br><br>chore(deps): update taiki-e/install-action action to v2.66.0 by @renovate\[bot] in [#66](https://github.com/Paul-16098/my-http-server/pull/66)

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098


### Refactor

- *api*: Reorganize API documentation and Swagger UI files by @Paul-16098


### Styling

- *html-t.hbs*: Format HTML structure for improved readability by @Paul-16098


### Miscellaneous Tasks

- Try use cargo-dist by @Paul-16098

- *workflow*: Remove obsolete Build&release workflow file by @Paul-16098


## \[4.1.0\] - 2026-01-09

### Features

- Feat(api): add file retrieval endpoint with validation and IP filtering<br>chore(config): update IP filtering configuration structure<br>fix(swagger): update Swagger UI dependencies to latest version by @Paul-16098

- *api*: Enhance file metadata and directory listing endpoints with security and performance notes by @Paul-16098

- *api*: Add PathType enum and refactor ExistsResponse handling by @Paul-16098

- *workflow*: Enable manual trigger for security audit workflow by @Paul-16098

- *error*: Add CliError variant to AppError for better CLI error handling by @Paul-16098

- *api*: Refactor configuration and build server to support optional API feature by @Paul-16098

- *tests*: Update test job to use cargo-all-features for comprehensive testing by @Paul-16098

- *tests*: Replace cargo binstall with install-action for cargo-all-features by @Paul-16098

- *workflow*: Add workflow_run trigger for release-tag-and-backmerge by @Paul-16098

- Update version to 4.1.0 and enhance API documentation by @Paul-16098

- *build*: Add function to download GitHub emojis if not present by @Paul-16098

- Comment out emojis.json copy and add conditional compilation for emoji download by @Paul-16098

- Update CI workflow to use cargo-make for testing and add Makefile for task management by @Paul-16098

- Add install-dep task to Makefile for cargo-all-features installation by @Paul-16098

- *cli*: Enhance CLI argument parsing with layered configuration and precedence by @Paul-16098

- *config*: Add XDG config directory support and update configuration precedence by @Paul-16098

- *config*: Add support for custom 404 error page and HTML template paths with XDG configuration by @Paul-16098


### Bug Fixes

- *config*: Replace logging error with println in config loading by @Paul-16098

- *build*: Simplify error handling for GitHub emojis download by @Paul-16098

- *license*: Update copyright year from 2025 to 2026 by @Paul-16098

- *meta*: Update stylesheet link to use CDN for consistency and reliability by @Paul-16098

- *request*: Improve Markdown rendering logic and enhance error handling by @Paul-16098

- *workflow*: Uncomment conditional for test-with-httpyac job by @Paul-16098

- *workflow*: Update Docker Buildx action to version 3 by @Paul-16098

### <details><summary>Deps</summary>


- Update rust crate actix-files to v0.6.9 (#48) by @renovate\[bot] in [#48](https://github.com/Paul-16098/my-http-server/pull/48)

- Update taiki-e/install-action action to v2.62.59 (#47) by @renovate\[bot] in [#47](https://github.com/Paul-16098/my-http-server/pull/47)

- Update rust toolchain setup action to v1.15.2 by @Paul-16098

- Update rust crate rustls-pki-types to v1.13.1 (#50) by @renovate\[bot] in [#50](https://github.com/Paul-16098/my-http-server/pull/50)

- Update taiki-e/install-action action to v2.62.63 (#51) by @renovate\[bot] in [#51](https://github.com/Paul-16098/my-http-server/pull/51)

- Update rust crate log to v0.4.29 (#52) by @renovate\[bot] in [#52](https://github.com/Paul-16098/my-http-server/pull/52)

- Update rust crate markdown-ppp to v2.8.0 (#53) by @renovate\[bot] in [#53](https://github.com/Paul-16098/my-http-server/pull/53)

- Update taiki-e/install-action action to v2.62.66 (#54) by @renovate\[bot] in [#54](https://github.com/Paul-16098/my-http-server/pull/54)

- Update rust crate markdown-ppp to v2.8.1 (#55) by @renovate\[bot] in [#55](https://github.com/Paul-16098/my-http-server/pull/55)

- Update taiki-e/install-action action to v2.63.1 by @renovate\[bot]

- Update rust crate rustls to v0.23.36 by @renovate\[bot]

- Update rust crate clap to v4.5.54 (#62) by @renovate\[bot] in [#62](https://github.com/Paul-16098/my-http-server/pull/62)

- Update rust crate rustls-pki-types to v1.13.2 (#58) by @renovate\[bot] in [#58](https://github.com/Paul-16098/my-http-server/pull/58)

- Update rust crate serde_json to v1.0.149 (#59) by @renovate\[bot] in [#59](https://github.com/Paul-16098/my-http-server/pull/59)

- Update rust crate tempfile to v3.24.0 (#60) by @renovate\[bot] in [#60](https://github.com/Paul-16098/my-http-server/pull/60)

- Update taiki-e/install-action action to v2.65.15 (#57) by @renovate\[bot] in [#57](https://github.com/Paul-16098/my-http-server/pull/57)

- Update rust crate handlebars to v6.4.0 (#61) by @renovate\[bot] in [#61](https://github.com/Paul-16098/my-http-server/pull/61)

</details>

### Other

- Checkpoint from VS Code for coding agent session by @Paul-16098

- Add file-related API endpoints<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Refactor validation functions and improve data structures<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Remove temporary test file by @Copilot

- Fix minor formatting issues<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Merge pull request #43 from Paul-16098/copilot/parliamentary-pig<br><br>Add file operations API endpoints (info, list, exists) by @Paul-16098 in [#43](https://github.com/Paul-16098/my-http-server/pull/43)

- Merge pull request #45 from Paul-16098/renovate/taiki-e-install-action-2.x<br><br>chore(deps): update taiki-e/install-action action to v2.62.57 by @renovate\[bot] in [#45](https://github.com/Paul-16098/my-http-server/pull/45)

- Merge pull request #46 from Paul-16098:refactor(github-emojis)<br><br>refactor: update dependencies and improve emoji handling logic by @Paul-16098 in [#46](https://github.com/Paul-16098/my-http-server/pull/46)

- Merge pull request #49 from Paul-16098/renovate/actix-web-4.x-lockfile<br><br>chore(deps): update rust crate actix-web to v4.12.1 by @renovate\[bot] in [#49](https://github.com/Paul-16098/my-http-server/pull/49)

- Merge pull request #56 from Paul-16098:renovate/taiki-e-install-action-2.x<br><br>chore(deps): update taiki-e/install-action action to v2.63.1 by @Paul-16098 in [#56](https://github.com/Paul-16098/my-http-server/pull/56)

- Merge pull request #63 from Paul-16098/renovate/rustls-0.x-lockfile<br><br>chore(deps): update rust crate rustls to v0.23.36 by @Paul-16098 in [#63](https://github.com/Paul-16098/my-http-server/pull/63)

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Pin docker/setup-buildx-action to commit hash (#65)<br><br>* Initial plan<br><br>* fix(workflow): pin docker/setup-buildx-action to commit hash for security<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com><br><br>---------<br><br>Co-authored-by: copilot-swe-agent\[bot] <198982749+Copilot@users.noreply.github.com><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot in [#65](https://github.com/Paul-16098/my-http-server/pull/65)

- Merge pull request #64 from Paul-16098/release-5.0.0<br><br>release-5.0.0 by @Paul-16098 in [#64](https://github.com/Paul-16098/my-http-server/pull/64)


### Refactor

- *build*: Extract warning logging to a separate function for better readability by @Paul-16098

- Update dependencies and improve emoji handling logic by @Paul-16098

- Enhance GitHub emoji handling with token authorization and update .env configuration by @Paul-16098

- Update documentation and configuration for improved clarity and structure by @Paul-16098

- Update GitHub token handling in emojis initialization by @Paul-16098

- 增强配置和错误处理，添加 Clippy lint 规则，改进 API 错误响应 by @Paul-16098

- 移除不必要的配置常量，简化集成测试中的 Cofg 实例化 by @Paul-16098

- 重构工作流，移除不必要的测试步骤并调整作业名称 by @Paul-16098

- 修改工作流名称为 test by @Paul-16098


### Documentation

- 更新文档 by @Paul-16098


### Miscellaneous Tasks

- *docs*: 更新文档以反映最新的 AI 编码代理指引和配置映射 by @Paul-16098

- 删除旧的 CI/CD 工作流文件，整合构建与测试流程 by @Paul-16098

- 更新 AI 编码代理指南，调整请求流和配置选项，添加 TLS 支持说明 by @Paul-16098

- *ci*: Update test workflow to use dynamic port and remove unnecessary comments by @Paul-16098


## \[4.0.0\] - 2025-11-21

### Features

- Implement HTML and TOC caching with LRU strategy for improved performance by @Paul-16098

- Add emojis.json to Dockerfile for enhanced functionality by @Paul-16098

- 更新 copilot 指南以增強架構與路由邏輯的描述；修正 Markdown 渲染函數以返回結果狀態；新增 .gitignore 以排除快取目錄 by @Paul-16098

- 更新 copilot 指南，新增配置示例與安全性建議；修正測試任務描述與內容類型 by @Paul-16098

- 更新 Cargo.toml 和 Cargo.lock，新增 tempfile 依賴；調整 cofg.yaml 格式；重構 config.rs 和 markdown.rs 測試；移除不必要的測試檔案 by @Paul-16098

- 重构多个模块以简化代码，移除冗余逻辑并优化性能 by @Paul-16098

- *parser*: Simplify Markdown rendering and remove HTML caching by @Paul-16098

- *tests*: Add comprehensive integration, request, security, and templating tests by @Paul-16098

- *ci*: Add artifact path setting and verification for release process by @Paul-16098

- *errors*: Implement Responder and ResponseError for AppError with status mapping by @Paul-16098

- *tests*: Add with_cwd_lock helper to prevent CWD race conditions in integration tests by @Paul-16098

- *cli*: Add root_dir argument for execution context and update config loading logic by @Paul-16098

- *api*: Integrate Swagger UI and OpenAPI documentation support by @Paul-16098

- *docker*: Add swagger-ui.html and LICENSE.txt to Docker image by @Paul-16098


### Bug Fixes

- *deps*: Update rust crate actix-governor to 0.10.0 by @renovate\[bot]

- Update linker configuration in config.toml to resolve PDB LNK1318 issue by @Paul-16098

- Disable test-with-httpyac job to allow failed runs by @Paul-16098

- 移除 httpyac 安裝步驟，並更新測試命令以使用 npx by @Paul-16098

- 注釋掉測試工作流中的條件判斷以避免失敗 by @Paul-16098

- Fix(errors): improve error response body handling and simplify status code logic<br>fix(tests): update assertion for TOC test to check for non-empty result by @Paul-16098

- *ci*: Fix Unpinned tag for a non-immutable Action in workflow by @Paul-16098

### <details><summary>Deps</summary>


- Update rust crate rustls to v0.23.34 by @renovate\[bot]

- Update rust crate clap to v4.5.50 (#30) by @renovate\[bot] in [#30](https://github.com/Paul-16098/my-http-server/pull/30)

- Update rust crate rustls-pki-types to v1.13.0 (#33) by @renovate\[bot] in [#33](https://github.com/Paul-16098/my-http-server/pull/33)

- Update rust crate clap to v4.5.51 (#34) by @renovate\[bot] in [#34](https://github.com/Paul-16098/my-http-server/pull/34)

- Update rust crate rustls to v0.23.35 (#35) by @renovate\[bot] in [#35](https://github.com/Paul-16098/my-http-server/pull/35)

- Update docker/dockerfile docker tag to v1.20 (#37) by @renovate\[bot] in [#37](https://github.com/Paul-16098/my-http-server/pull/37)

- Update rust crate config to v0.15.19 (#36) by @renovate\[bot] in [#36](https://github.com/Paul-16098/my-http-server/pull/36)

- Update rust crate clap to v4.5.52 (#39) by @renovate\[bot] in [#39](https://github.com/Paul-16098/my-http-server/pull/39)

- Update rust crate actix-web to v4.12.0 (#38) by @renovate\[bot] in [#38](https://github.com/Paul-16098/my-http-server/pull/38)

- Update dependencies in Cargo.lock by @Paul-16098

- Update actions/checkout action to v6 (#41) by @renovate\[bot] in [#41](https://github.com/Paul-16098/my-http-server/pull/41)

- Update rust crate clap to v4.5.53 (#40) by @renovate\[bot] in [#40](https://github.com/Paul-16098/my-http-server/pull/40)

</details>

### Other

- Merge pull request #28 from Paul-16098/main<br><br>chore: back-merge v3.2.0 into dev by @Paul-16098 in [#28](https://github.com/Paul-16098/my-http-server/pull/28)

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Merge pull request #29 from Paul-16098/renovate/actix-governor-0.x<br><br>fix(deps): update rust crate actix-governor to 0.10.0 by @Paul-16098 in [#29](https://github.com/Paul-16098/my-http-server/pull/29)

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Potential fix for code scanning alert no. 1: Workflow does not contain permissions<br><br>Co-authored-by: Copilot Autofix powered by AI <62310815+github-advanced-security\[bot]@users.noreply.github.com> by @Paul-16098

- Implement code changes to enhance functionality and improve performance by @Paul-16098

- Merge pull request #32 from Paul-16098:renovate/rustls-0.x-lockfile<br><br>chore(deps): update rust crate rustls to v0.23.34 by @Paul-16098 in [#32](https://github.com/Paul-16098/my-http-server/pull/32)

- 👷 ci(test-with-httpyac): add for act by @Paul-16098

- Enable conditional job execution in cli.yml by @Paul-16098

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- V4.0.0 by @Paul-16098 in [#42](https://github.com/Paul-16098/my-http-server/pull/42)


### Refactor

- Optimize Dockerfile by adding cargo-chef for improved build efficiency by @Paul-16098

- Update Dockerfile to use cargo-chef for improved build efficiency and enhance .dockerignore for better context management by @Paul-16098

- *request,parser,tests,build*: Unify root routing into main_req; add TOC renderer; expose server_error; bump deps by @Paul-16098


### Documentation

- Add why use ring by @Paul-16098

- 更新 AI 開發速覽文檔，精簡內容並調整格式 by @Paul-16098


### Styling

- *README,copilot-instructions*: Update by @Paul-16098


### Testing

- Add body test by @Paul-16098

- Update assertion for ip_filter_config_structure to use assert! macro by @Paul-16098

- Update version to v3.2.0 in footer links across HTML files by @Paul-16098

- *cli*: Add root_dir initialization in argument tests by @Paul-16098


### Miscellaneous Tasks

- Add build step to Docker workflow for improved CI process by @Paul-16098

- *release*: Attest build provenance by @Paul-16098

- Update package version to 4.0.0 by @Paul-16098


## \[3.2.0\] - 2025-10-11

### Other

- Merge pull request #24 from Paul-16098/main<br><br>chore: back-merge v3.1.0 into dev by @Paul-16098 in [#24](https://github.com/Paul-16098/my-http-server/pull/24)

- 使用 TOML 讀取器替換從 Cargo.toml 提取版本的自定義腳本 by @Paul-16098

- 新增 actix-web-httpauth 依賴並更新 TLS 錯誤處理 by @Paul-16098

- 新增 HTTP 基本認證功能，更新配置結構以支持用戶名和密碼驗證 by @Paul-16098

- 新增常數時間比較函數以增強密碼驗證安全性，並優化用戶名驗證邏輯 by @Paul-16098

- 優化 Option<&str> 的常數時間比較邏輯，簡化 None 情況的處理，提升代碼可讀性 by @Paul-16098

- 修改未授權錯誤訊息，統一用戶名和密碼的提示內容 by @Paul-16098

- Merge pull request #25 from Paul-16098/feat/add-http-base-auth by @Paul-16098 in [#25](https://github.com/Paul-16098/my-http-server/pull/25)

- Initial plan by @Copilot

- Add IP filter functionality using actix-ip-filter<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Add IP filter documentation and update README<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Add comprehensive IP filter implementation summary<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- 更新 IP 過濾器設定，新增配置範例及文件說明 by @Paul-16098

- Merge pull request #26 from Paul-16098/copilot/add-ip-filter-functionality<br><br>Add IP filter functionality using actix-ip-filter by @Paul-16098 in [#26](https://github.com/Paul-16098/my-http-server/pull/26)

- 新增 actix-governor 套件，更新配置以支援速率限制功能，並調整 IP 過濾器設定 by @Paul-16098

- 新增主分支合併至開發分支的工作流程，處理合併衝突並推送更新 by @Paul-16098

- V3.2.0 by @Paul-16098

- 修正 TLS 設定邏輯中的導入，合併 PrivatePkcs8KeyDer 至 rustls::pki_types 模組，並簡化錯誤處理邏輯以增強可讀性 by @Paul-16098

- Merge pull request #27 from Paul-16098/release-3.2.0<br><br>v3.2.0 by @Paul-16098 in [#27](https://github.com/Paul-16098/my-http-server/pull/27)


### Documentation

- Consolidate and enhance documentation across multiple files by @Paul-16098


## \[3.1.0\] - 2025-10-10

### Features

- *templating*: Migrate to Handlebars engine (add html-t.hbs, remove legacy template); refactor templating engine and registration by @Paul-16098

- *markdown*: Align md->HTML pipeline with new Handlebars-based templating by @Paul-16098

- *config*: Expose templating options and defaults; wire CLI flags to config by @Paul-16098


### Bug Fixes

- Add missing DOCTYPE declaration in HTML template and remove debug log in parser by @Paul-16098

### <details><summary>Deps</summary>


- Update Cargo.toml and lockfile for Handlebars and related changes by @Paul-16098

</details>

### Other

- Initial plan by @Copilot

- Add TLS support with rustls<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Add TLS documentation and .gitignore rules for certificates<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Update Docker and README with TLS/HTTPS usage examples<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Fix cross-compilation by switching to ring crypto backend<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Refactor TLS configuration loading and update tests for clarity by @Paul-16098

- *docker*: Update image build to align with new templating defaults by @Paul-16098

- Merge pull request #22 from Paul-16098/feature/use-handlebars<br><br>feat(templating): migrate to Handlebars and align pipeline, config, tests, and docs by @Paul-16098 in [#22](https://github.com/Paul-16098/my-http-server/pull/22)

- Merge branch 'dev' into copilot/add-tls-support by @Paul-16098

- Fix TLS CLI logic and improve key selection clarity<br><br>- Only enable TLS when both cert and key are provided via CLI<br>- Change from pop() to into_iter().next() for first key selection<br>- Add tests for partial CLI arguments (cert-only, key-only)<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Merge branch 'copilot/add-tls-support' of https://github.com/Paul-16098/my-http-server into copilot/add-tls-support by @Paul-16098

- 更新依賴版本並調整格式 by @Paul-16098

- 簡化伺服器啟動日誌，根據 TLS 設定顯示 HTTP 或 HTTPS 協議 by @Paul-16098

- Merge pull request #21 from Paul-16098/copilot/add-tls-support<br><br>Add TLS/HTTPS support with rustls by @Paul-16098 in [#21](https://github.com/Paul-16098/my-http-server/pull/21)

- 更新工作流程配置，為推送事件添加分支條件 by @Paul-16098

- V3.1.0 by @Paul-16098

- Merge pull request #23 from Paul-16098:release-3.1.0<br><br>v3.1.0 by @Paul-16098 in [#23](https://github.com/Paul-16098/my-http-server/pull/23)


### Refactor

- *core*: Adjust main flow and error types for Handlebars rendering path by @Paul-16098


### Documentation

- Document Handlebars migration and template usage; refine Copilot instructions by @Paul-16098

- 更新 Copilot 指示檔，增加維護與更新條件說明 by @Paul-16098


### Testing

- *templating*: Update parser and templating tests for Handlebars; add request route test; refresh fixtures by @Paul-16098

- Remove unused common test helpers after templating refactor by @Paul-16098


### Miscellaneous Tasks

- Reorganize Dockerfile and update meta file paths; add new HTML templates and test files by @Paul-16098

- Add testing job with httpyac to GitHub Actions workflow by @Paul-16098

- Remove binary build step and run cargo in background for testing by @Paul-16098

- Update httpyac test command to remove --bail option by @Paul-16098

- Add wait step for server readiness in httpyac testing job by @Paul-16098

- Add cargo build step before running the server in GitHub Actions workflow by @Paul-16098

- Update .dockerignore and Dockerfile for improved build context and meta inclusion by @Paul-16098


## \[3.0.3\] - 2025-10-07

### Features

- 新增模板和配置，更新 Dockerfile 和 docker-compose.yml by @Paul-16098

- 更新 Dockerfile，新增非根用戶創建；更新配置文件以支持忽略特定目錄 by @Paul-16098

- 更新配置文件，升級 config 依賴至 0.15.18，移除 TOC 生成選項 by @Paul-16098

- 更新 Markdown TOC 生成邏輯，新增結構化輸出；重構請求處理以支持動態 TOC by @Paul-16098

- 更新 copilot 說明文件，調整內容結構與語言，增強可讀性與清晰度 by @Paul-16098

- 更新 README.md，調整內容結構與語言，增強可讀性與清晰度 by @Paul-16098

- *toc*: Add directory TOC rendering path and integrate with templating by @Paul-16098


### Bug Fixes

- If no env `VERSION` build will fail by @Paul-16098

- *request*: Improve error handling for non-file and non-directory requests by @Paul-16098

- *request*: Enhance error handling for path resolution in Markdown rendering by @Paul-16098

- *docker*: Add missing COPY command for docker directory in Dockerfile by @Paul-16098

### <details><summary>Deps</summary>


- Update docker/dockerfile docker tag to v1.19 by @renovate\[bot]

- Update actions/checkout action to v5 by @renovate\[bot]

- Update actions/github-script action to v8 by @renovate\[bot]

</details>

### Other

- Merge pull request #14 from Paul-16098/release-3.0.2<br><br>release: 3.0.2 by @Paul-16098 in [#14](https://github.com/Paul-16098/my-http-server/pull/14)

- Merge remote-tracking branch 'origin/main' into dev by @Paul-16098

- Merge pull request #15 from Paul-16098/renovate/docker-dockerfile-1.x<br><br>chore(deps): update docker/dockerfile docker tag to v1.19 by @Paul-16098 in [#15](https://github.com/Paul-16098/my-http-server/pull/15)

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Adjust build script for config module changes by @Paul-16098

- Merge pull request #16 from Paul-16098:feature/add-dir-toc<br><br>feat(toc): add directory TOC + cofg refactor by @Paul-16098 in [#16](https://github.com/Paul-16098/my-http-server/pull/16)

- Merge pull request #17 from Paul-16098/release-3.0.3 by @Paul-16098 in [#17](https://github.com/Paul-16098/my-http-server/pull/17)

- Merge pull request #18 from Paul-16098/release-3.0.3<br><br>chore: back-merge release 3.0.3 into dev by @Paul-16098 in [#18](https://github.com/Paul-16098/my-http-server/pull/18)

- Merge pull request #19 from Paul-16098/renovate/actions-checkout-5.x<br><br>chore(deps): update actions/checkout action to v5 by @Paul-16098 in [#19](https://github.com/Paul-16098/my-http-server/pull/19)

- Merge pull request #20 from Paul-16098/renovate/actions-github-script-8.x by @Paul-16098 in [#20](https://github.com/Paul-16098/my-http-server/pull/20)


### Refactor

- 移除不再使用的配置文件並更新 docker-compose 路徑 by @Paul-16098

- *cofg*: Replace cofg.rs with config.rs and update module wiring by @Paul-16098

- Refactor(cli): update Args conversion to use TryFrom for better error handling<br>refactor(mod): change build_config_from_cli to return AppResult for error propagation<br>fix(error): rename error variant from _Other to Other for consistency<br>fix(request): improve path handling with better error logging in main_req<br>test(cli): update tests to reflect changes in Args conversion logic by @Paul-16098


### Documentation

- Update README and Copilot instructions by @Paul-16098


### Testing

- Test my pgp key by @Paul-16098

- Update tests for config refactor and directory TOC by @Paul-16098


### Miscellaneous Tasks

- Remove dependabot config (moved to Renovate) by @Paul-16098

- Auto-tag on main merge and open maindev back-merge PR; close releasedev PRs by @Paul-16098


## \[3.0.2\] - 2025-09-29

### Features

- *http*: Add per-request HttpRequest cached helpers and integrate in main; add tests by @Paul-16098

- *docs*: Add internal architecture, flow, performance, config mapping docs and WHY comments by @Paul-16098


### Bug Fixes

- 修正發佈工序中的簽名檔案匹配模式，從遞迴匹配改為單層匹配 by @Paul-16098

- 更新發佈工序以支援多層次簽名檔案匹配 by @Paul-16098

- 更新 GPG 簽名步驟以支援遞迴列出釋出檔案和簽名檔案 by @Paul-16098

- 在 GPG 簽名步驟中新增列出檔案的指令 by @Paul-16098

- 簡化 GPG 簽名步驟，移除多餘的檔案搜尋與簽名邏輯 by @Paul-16098

- 在發佈工序中新增對 .asc 檔案的支援 by @Paul-16098

- Fix: 更新 markdown-ppp 版本至 2.1.1 並移除 Git 來源，改為使用註冊表<br>feat: 新增 Dependabot 配置以自動更新依賴 by @Paul-16098

### <details><summary>Deps</summary>


- Update docker/dockerfile docker tag to v1.18 by @renovate\[bot]

- Update rust docker tag to v1.90.0 by @renovate\[bot]

- Update actions/checkout action to v5 by @renovate\[bot]

</details>

### Other

- Back-merge release 3.0.1 into dev by @Paul-16098

- Merge pull request #7 from Paul-16098/feature/http-request-caching<br><br>feat(http): add per-request HttpRequest cached helpers; integrate in main; add tests by @Paul-16098 in [#7](https://github.com/Paul-16098/my-http-server/pull/7)

- Merge pull request #8 from Paul-16098/feature/docs-internal-architecture<br><br>feat(docs): internal architecture & rendering flow documentation by @Paul-16098 in [#8](https://github.com/Paul-16098/my-http-server/pull/8)

- Add renovate.json by @renovate\[bot]

- Merge pull request #9 from Paul-16098/renovate/configure<br><br>chore: Configure Renovate by @Paul-16098 in [#9](https://github.com/Paul-16098/my-http-server/pull/9)

- Merge pull request #10 from Paul-16098/renovate/docker-dockerfile-1.x<br><br>chore(deps): update docker/dockerfile docker tag to v1.18 by @Paul-16098 in [#10](https://github.com/Paul-16098/my-http-server/pull/10)

- Merge pull request #11 from Paul-16098/renovate/rust-1.x<br><br>chore(deps): update rust docker tag to v1.90.0 by @Paul-16098 in [#11](https://github.com/Paul-16098/my-http-server/pull/11)

- Merge pull request #13 from Paul-16098/renovate/actions-checkout-5.x<br><br>chore(deps): update actions/checkout action to v5 by @Paul-16098 in [#13](https://github.com/Paul-16098/my-http-server/pull/13)


### Refactor

- 更新 GPG 簽名流程，移除不必要的條件檢查並整合發佈工序 by @Paul-16098


### Miscellaneous Tasks

- *release*: Use gpg --detach-sign (remove armor) for artifacts by @Paul-16098



- @renovate[bot] made their first contribution## \[3.0.1\] - 2025-09-19

### Features

- Add clap for command line argument parsing and enhance configuration management by @Paul-16098

- Enhance configuration handling and add CLI tests by @Paul-16098

- Add clap for command line argument parsing and enhance configuration management by @Paul-16098

- Enhance configuration handling and add CLI tests by @Paul-16098


### Bug Fixes

- Update configuration handling in init function and markdown parser by @Paul-16098

- Update markdown-ppp source to point to the correct Git repository by @Paul-16098

- Update configuration handling in init function and markdown parser by @Paul-16098

- Update markdown-ppp source to point to the correct Git repository by @Paul-16098


### Other

- Merge pull request #3 from Paul-16098/release-3.0.0<br><br>release-3.0.0: version bump and dependency alignment by @Paul-16098 in [#3](https://github.com/Paul-16098/my-http-server/pull/3)

- Initial plan by @Copilot

- Implement GPG signing for release artifacts<br><br>Co-authored-by: Paul-16098 <127955132+Paul-16098@users.noreply.github.com> by @Copilot

- Merge pull request #6 from Paul-16098/copilot/fix-5 by @Paul-16098 in [#6](https://github.com/Paul-16098/my-http-server/pull/6)

- Merge pull request #4 from Paul-16098:feat/clap<br><br>Add CLI argument parsing and enhance configuration management by @Paul-16098 in [#4](https://github.com/Paul-16098/my-http-server/pull/4)

- Merge branch 'dev' of https://github.com/Paul-16098/my-http-server into dev by @Paul-16098

- Bump version to 3.0.1 by @Paul-16098

- Release 3.0.1 into main by @Paul-16098


### Refactor

- Rename set_context to set_context_value for clarity and update usage in parser by @Paul-16098

- Rename set_context to set_context_value for clarity and update usage in parser by @Paul-16098


### Documentation

- *README*: Add missing Docker and Docker-test badges by @Paul-16098

- 更新 AI 開發速覽，調整內容以增強專案架構與路由的清晰度 by @Paul-16098

- *README*: Add missing Docker and Docker-test badges by @Paul-16098

- 更新 AI 開發速覽，調整內容以增強專案架構與路由的清晰度 by @Paul-16098



- @Copilot made their first contribution## \[3.0.0\] - 2025-09-17

### Features

- Add callback 404 by @Paul-16098

- *config*: 新增 Cofg 結構及其 YAML 配置檔案 by @Paul-16098

- Feat(docs): 更新 README.md 以更清楚地說明功能與安裝步驟，移除不必要的說明<br>refactor(main): 精簡檔案處理邏輯，移除不再使用的 _public 目錄相關程式碼 by @Paul-16098

- Feat(cofg): add more cofg<br>docs(README): update<br>test(cofg): add cofg test by @Paul-16098

- Add templating by @Paul-16098

- Add templating cofg by @Paul-16098

- *templating*: Add hot-reload by @Paul-16098

- *make_toc*: Add make toc by @Paul-16098

- *cofg*: 更改格式 by @Paul-16098

- Feat(toc): 增加目錄生成的擴展名選項，並更新生成邏輯<br>refactor(cli): 移除不必要的 sccache 配置步驟<br>docs: 更新 README.md，添加 CI/CD 狀態徽章 by @Paul-16098

- 更新依賴版本，新增錯誤處理，重構配置加載邏輯 by @Paul-16098

- *tests*: 新增 parser 與 templating 模組的測試案例 by @Paul-16098

- 新增 cfg_aliases 和 ctrlc 依賴，重構伺服器啟動邏輯，改進監視器循環 by @Paul-16098

- 更新 md2html 函數以接受模板數據列表，改進上下文設置邏輯 by @Paul-16098

- 新增 Dockerfile 和 .dockerignore，支持容器化部署 by @Paul-16098


### Bug Fixes

- Failed to resolve: could not find `windows` in `os` by @Paul-16098

- Unresolved imports `std::os::unix::fs::symlink_dir`, `std::os::unix::fs::symlink_file` by @Paul-16098

- Expected a type, found a trait by @Paul-16098

- Missing generics for trait `Fn` by @Paul-16098

- Failed to resolve: use of unresolved module or unlinked crate `io` by @Paul-16098

- Fix: by @Paul-16098

- *watcher_loop*: Input watch path is neither a file nor a directory. by @Paul-16098

- If templating_value none unwrap  value: invalid type: unit value, expected a map for key `templating_value` by @Paul-16098

- *make_toc*: 百分比編碼使url無效 by @Paul-16098

- *tests*: 修正模板上下文處理布林值同義詞的測試案例 by @Paul-16098

- *docker*: Try fix bug by @Paul-16098

- Fix bug by @Paul-16098

- Fix bug by @Paul-16098

- Standardize formatting in docker-publish.yml and add binary section in Cargo.toml by @Paul-16098

- Update HTML template structure and remove unused configuration by @Paul-16098

- Improve formatting of HTML template in Dockerfile by @Paul-16098

- Update docker-publish.yml to use latest action versions and remove tag publishing by @Paul-16098

- *main*: Improve error handling for filename parsing and enhance debug messages by @Paul-16098


### Other

- 1.0.0 by @Paul-16098

- . by @Paul-16098

- . by @Paul-16098

- Logging request by @Paul-16098

- *logger*: 可读性優化 by @Paul-16098

- Merge pull request #1 from Paul-16098:feat/templating<br><br>feat(templating): add templating by @Paul-16098 in [#1](https://github.com/Paul-16098/my-http-server/pull/1)

- *parser*: 重構 `parser::md2html` by @Paul-16098

- Make_toc by @Paul-16098

- Update Rust and Debian base images in Dockerfile by @Paul-16098

- Add docker-publish.yml by @Paul-16098

- *server*: Refactor routing to handlers (index + catch-all), improve percent-decoded URL logging, and simplify startup; keep legacy batch ops available behind helpers by @Paul-16098

- Merge pull request #2 from Paul-16098:feature/docker-compose-dockerfile-improvements<br><br>Improve Docker/Compose, add default templates, refactor server routing by @Paul-16098 in [#2](https://github.com/Paul-16098/my-http-server/pull/2)

- *3.0.0*: Bump version and align dependencies; minor config tweak by @Paul-16098


### Documentation

- *gitignore*: Edit by @Paul-16098

- *readme*: Update Docker/Compose guide, meta volume caveat, and env var injection example (SITE_NAME) by @Paul-16098


### Testing

- 新增模板上下文解析測試，改進配置加載邏輯 by @Paul-16098


### Miscellaneous Tasks

- Style by @Paul-16098

- Chore(percent-encoding): 2.3.1 => 2.3.2<br>feat(auto-reload): add auto reload by @Paul-16098

- *Security audit*: Fix permissions error by @Paul-16098

- Ci(extra-files): add default 404 page and html-t.templating<br>bump: 2.0.0 by @Paul-16098

- Ci(extra-files): add default 404 page and html-t.templating<br>bump: 2.0.0 by @Paul-16098

- *docker*: Speed up build, fix BuildKit cache COPY, add healthcheck and default templates; widen VOLUME to include meta by @Paul-16098

- *compose*: Correct cofg path under docker/, drop obsolete version field, keep volumes for public/meta by @Paul-16098

- *config*: Container default bind 0.0.0.0:8080 and sample templating.value; keep watch/hot_reload enabled by @Paul-16098

- *templates*: Add default html-t.templating and 404.html for local/dev and container volume overrides by @Paul-16098

- Update dependencies, remove unused packages, and rename markdown functions for clarity by @Paul-16098


### Revert

- 78810d8ab72816c6ae50d15662ad3bbd5eab19aa by @Paul-16098

- 68cc969a7fb8d1e78da3000b9a14b3d5fb3d632e by @Paul-16098



- @Paul-16098 made their first contribution