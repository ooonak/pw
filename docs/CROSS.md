# Manual approach

Requires a lot of stuff installed, toolchains for every target.

## Install tooling for the proper targets

```
# RPi3
$ rustup target add armv7-unknown-linux-gnueabihf

# BBB
$ rustup target add arm-unknown-linux-gnueabihf
```

Configure config.toml
```
$ cat .cargo/config.toml
[target.armv7-linux-androideabi]
ar = "/opt/android-sdk/ndk/25.2.9519653/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "/opt/android-sdk/ndk/25.2.9519653/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi19-clang"

# RPi 2/3/4
[target.armv7-unknown-linux-gnueabihf]
#linker = "arm-none-linux-gnueabihf-gcc"
rustflags = [ "-C", "link-arg=-fuse-ld=lld"]

# BBB
[target.arm-unknown-linux-gnueabihf]
rustflags = [ "-C", "link-arg=-fuse-ld=lld"]
```

```
$ cargo build --bin service --target armv7-unknown-linux-gnueabihf
```
