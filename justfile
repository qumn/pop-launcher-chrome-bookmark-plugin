build:
    cargo build --release

_install:
    mkdir -p ~/.local/share/pop-launcher/plugins/chrome-book/
    install -Dm0755 ./target/release/pop-launcher-chrome-bookmarks-plugin ~/.local/share/pop-launcher/plugins/chrome-book/cb
    install -Dm644 plugin.ron ~/.local/share/pop-launcher/plugins/chrome-book/plugin.ron

install:
    just build
    just _install
