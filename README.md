# ScrollSplit

Minimal macOS CLI that independently reverses mouse-wheel and trackpad scrolling.

## Install with Homebrew

```sh
brew install knothhe/tap/scrollsplit
scrollsplit run
```

Grant Accessibility permission when macOS prompts, then start it in the
background:

```sh
brew services start scrollsplit
```

Homebrew taps do not require registration. This formula is published through
the public `knothhe/homebrew-tap` GitHub repository. Homebrew discovers that
repository from the `knothhe/tap` name.

To publish a new version:

```sh
gh repo create knothhe/homebrew-tap --public # first release only

# Update the Cargo.toml version, commit it, then:
./scripts/release.sh
```

The local release script runs tests, builds Apple Silicon and Intel archives,
creates and pushes the Git tag, creates the GitHub Release, then updates and
pushes `Formula/scrollsplit.rb` in `knothhe/homebrew-tap`. It requires macOS,
Rust, and an authenticated GitHub CLI (`gh auth login`).

For forks, override the repositories when publishing:

```sh
SOURCE_REPOSITORY=owner/scroll-split \
HOMEBREW_TAP_REPOSITORY=owner/homebrew-tap \
./scripts/release.sh
```

## Build from source

```sh
cargo build --release
./target/release/scrollsplit install-service
```

Grant Accessibility permission when macOS prompts. Configuration is stored at:

```text
~/Library/Application Support/ScrollSplit/config.toml
```

## Commands

```text
scrollsplit run
scrollsplit start
scrollsplit stop
scrollsplit restart
scrollsplit status
scrollsplit config show
scrollsplit config set <key> <value>
scrollsplit install-service
scrollsplit uninstall-service
```
