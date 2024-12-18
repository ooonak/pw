# Cross-compilation

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

## Approach 2 (very easy, nice for libs)

Install cargo extension for building Rust code for Android and the specific toolchain(s) you need.

```
$ cargo install cargo-ndk
$ rustup target add armv7-linux-androideabi
```

Build the project.

```
$ cargo ndk -vv -t armeabi-v7a -o ./jniLibs -p 19 build --release
```

# Ideas

## Process

### Compiler

What compiler was used. Several methods, what tool is most likely to be installed?

```
$ objdump -s --section .comment /usr/bin/ls
$   readelf -p .comment /usr/bin/ls
$ strings /usr/bin/ls | grep -e GCC -e clang
$ xxd /usr/bin/ls | grep -e GCC -e clang
```

e.g.

```
$ readelf -p .comment pw

String dump of section '.comment':
  [     0]  Android (8490178, based on r450784d) clang version 14.0.6 (https://android.googlesource.com/toolchain/llvm-project 4c603efb0cca074e9238af8b4106c30add4418f6)
  [    9d]  rustc version 1.81.0 (eeb90cda1 2024-09-04)
  [    c9]  Linker: LLD 14.0.7
  [    dc]  Android (9352603, based on r450784d1) clang version 14.0.7 (https://android.googlesource.com/toolchain/llvm-project 4c603efb0cca074e9238af8b4106c30add4418f6)
```

### Dynamic libraries

Sometimes it's nice to know version of C and C++ standard library.

```
$ objdump -p pw

pw:     file format elf32-little
...

Version References:
  required from libdl.so:
    0x00050d63 0x00 03 LIBC
  required from libc.so:
    0x00050d63 0x00 02 LIBC
```

```
$ objdump -p photosorter

photosorter:     file format elf64-x86-64
...

Version References:
  required from libgcc_s.so.1:
    0x0b792650 0x00 15 GCC_3.0
  required from libcrypto.so.1.1:
    0x066d1f10 0x00 12 OPENSSL_1_1_0
  required from libc.so.6:
    0x069691b2 0x00 17 GLIBC_2.32
    0x0d696914 0x00 11 GLIBC_2.4
    0x09691a75 0x00 08 GLIBC_2.2.5
    0x06969194 0x00 07 GLIBC_2.14
  required from libstdc++.so.6:
    0x0297f864 0x00 18 GLIBCXX_3.4.14
    0x0297f869 0x00 16 GLIBCXX_3.4.19
    0x0297f861 0x00 14 GLIBCXX_3.4.11
    0x0297f879 0x00 13 GLIBCXX_3.4.29
    0x0297f865 0x00 10 GLIBCXX_3.4.15
    0x0297f871 0x00 09 GLIBCXX_3.4.21
    0x056bafd3 0x00 06 CXXABI_1.3
    0x0297f876 0x00 05 GLIBCXX_3.4.26
    0x0297f870 0x00 04 GLIBCXX_3.4.20
    0x0bafd172 0x00 03 CXXABI_1.3.2
    0x08922974 0x00 02 GLIBCXX_3.4
```
