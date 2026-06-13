# ScrollSplit

Minimal macOS CLI that independently reverses mouse-wheel and trackpad scrolling.

## Build and install

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
