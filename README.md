# Process Watcher tool

[![Cargo Build & Test](https://github.com/ooonak/pw/actions/workflows/ci.yml/badge.svg)](https://github.com/ooonak/pw/actions/workflows/ci.yml)

## Overview

I created the "Process Watcher tool" or pw as a little project while teaching myself Rust.

The project consists of two binaries. A service (pwservice) running on a Linux target and a client (pwclient) that is running on your laptop. The service sends information about the machine, running processes and their threads. The client shows this information in a TUI. Like a distributed version of top.

## Test-spin

If you just wan't to build and start the service on your dev machine.

```
$ RUST_LOG=INFO cargo run --bin pwservice
```

And in another terminal.

```
$ cargo run --bin pwclient
```

## Cross-compile

Using the cross crate to build for e.g. RPi3. First install the cross crate.

```
$ cargo install cross --git https://github.com/cross-rs/cross
```

Then build, cross uses crosstool-ng toolchains in Docker containers.

```
# RPi3
$ cross build -p pwservice --release --target armv7-unknown-linux-gnueabihf

# BBB
$ cross build -p pwservice --release --target arm-unknown-linux-gnueabihf
```
