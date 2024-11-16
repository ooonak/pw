# Process Watcher tool

[![Cargo Build & Test](https://github.com/ooonak/pw/actions/workflows/ci.yml/badge.svg)](https://github.com/ooonak/pw/actions/workflows/ci.yml)

## Overview

I created the "Process Watcher tool" or pw as a little project while teaching myself Rust.

The project consists of two binaries. A service (pw) running on a Linux target and a client (pwtui) that is running on your laptop. The pw service sends information about the machine and running processes. The pwtui client displayes this information. Like a distributed version of top.

## Test-spin

If you just wan't to build and start the service on your dev machine.

```
$ RUST_LOG=INFO cargo run --bin service
```

And in another terminal.

```
$ cargo run --bin service
```
