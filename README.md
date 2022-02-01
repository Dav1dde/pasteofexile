Paste of Exile
==============

[![Build Status][actions-badge]][actions-url]
[![License][gplv3-badge]][gplv3-url]

[actions-badge]: https://img.shields.io/github/workflow/status/dav1dde/pasteofexile/CI?style=for-the-badge&logo=github
[actions-url]: https://github.com/Dav1dde/pasteofexile/actions?query=workflow%3ACI+branch%3Amaster
[gplv3-badge]: https://img.shields.io/badge/license-GPL3-blue.svg?style=for-the-badge
[gplv3-url]: ./LICENSE

[pobb.in](https://pobb.in), a pastebin for [Path Of Building](https://pathofbuilding.community/) builds.


![pobb.in](.github/assets/header.png)


## Development

Required dependencies:

* Node 16.7+
* Yarn
* Rust 1.52+ (including wasm toolchain: `rustup target add wasm32-unknown-unknown`)
* TrunkRS: `cargo install trunk`
* worker-build: `cargo install worker-build`
* wasm-pack: `cargo install wasm-pack`

The application can be started with:

```sh
$ yarn start
```


### Docker

Alternatively you can use docker to setup your build environment:

```sh
$ docker build -t pasteofexile .
$ docker run --rm -it \
    -v "$(pwd):/pasteofexile" \
    -p 8787:8787 \
    -u "$(id -u):$(id -g)" \
    pasteofexile \
    yarn start
```

### Code Style

Rust code is formatted with `cargo fmt` and linted with `cargo +nightly clippy --all-features -- -D warnings`.

Commits are prefixed with their scope:

* `poe:` if it is relevant for the entire project
* `app:` if it is mainly a frontend change
* `worker:` if it is mainly a backend/worker change
* `pob:` if it is a data parsing change in the `pob` crate


## Contributing

Contributions are always welcome, code, design ideas, mockups etc.

When contributing please try to follow coding conventions (`cargo fmt`, `cargo clippy`),
code style and commit formatting.

Before working on big features please open an issue first or reach out,
in case this feature is currently out of scope or already being worked on.
