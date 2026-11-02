 #!/bin/bash

cargo b --release
cargo build --release --target x86_64-apple-darwin
cargo zigbuild --release --target aarch64-unknown-linux-gnu
cargo zigbuild --release --target x86_64-unknown-linux-gnu
cargo xwin build --release --target x86_64-pc-windows-msvc
cargo xwin build --release --target aarch64-pc-windows-msvc

rm tsbundler/linux/arm64/*
rm tsbundler/linux/x86/*
rm tsbundler/macos/x86/*
rm tsbundler/macos/arm64/*
rm tsbundler/windows/*

cp target/aarch64-unknown-linux-gnu/release/libirisutils.so tsbundler/linux/arm64/
cp target/x86_64-unknown-linux-gnu/release/libirisutils.so tsbundler/linux/x86/
cp target/x86_64-pc-windows-msvc/release/irisutils.dll tsbundler/windows/
cp target/x86_64-pc-windows-msvc/release/irisutils.dll.lib tsbundler/windows/
cp target/x86_64-apple-darwin/release/libirisutils.dylib tsbundler/macos/x86/
cp target/release/libirisutils.dylib tsbundler/macos/arm64/
