# pw - Process watcher tool

## Install Android dev env

Get Android command-line tools (https://developer.android.com/studio#cmdline-tools).

```
$ yay -Sy android-sdk-cmdline-tools-latest android-sdk-platform-tools android-udev
$ sudo chown -R $USER:$USER /opt/android-sdk
$ /opt/android-sdk/cmdline-tools/latest/bin/sdkmanager --install "ndk;25.2.9519653"
$ echo 'export ANDROID_NDK_HOME="/opt/android-sdk/ndk/25.2.9519653"' >> ~/.bash_profile
```

### Approach 1 (binaries, out of the box)

Install target and add information needed to cross-compile.

```
$ rustup target add armv7-linux-androideabi

$ more .cargo/config.toml
[target.armv7-linux-androideabi]
ar = "/opt/android-sdk/ndk/25.2.9519653/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "/opt/android-sdk/ndk/25.2.9519653/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi19-clang"
```

Build the project.

```
$ cargo build --target armv7-linux-androideabi --release

$ file target/armv7-linux-androideabi/release/pw
target/armv7-linux-androideabi/release/pw: ELF 32-bit LSB pie executable, ARM, EABI5 version 1 (SYSV), dynamically linked, interpreter /system/bin/linker, not stripped
```

### Approach 2 (very easy, nice for libs)

Install cargo extension for building Rust code for Android and the specific toolchain(s) you need.

```
$ cargo install cargo-ndk
$ rustup target add armv7-linux-androideabi
```

Build the project.

```
$ cargo ndk -vv -t armeabi-v7a -o ./jniLibs -p 19 build --release
```
