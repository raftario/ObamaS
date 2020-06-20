# ObamaS

The Obama System, or ObamaS, an entreprise grade x86_64 operating system

## Requirements

* [rustup](https://rustup.rs/)
* [QEMU](https://www.qemu.org/download/)

## Setup

```
rustup toolchain install nightly
rustup component add --toolchain nightly rust-src llvm-tools-preview
cargo install cargo-xbuild
cargo install bootimage
```

## Build

```
cargo bootimage
```

## Run

```
cargo xrun
```

## Test

```
cargo xtest
```
