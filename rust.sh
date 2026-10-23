#!/bin/bash
cd "$(dirname "$0")/composeApp/src/main/rust"

"${CARGO:-cargo}" ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -o ../jniLibs \
  build --release

"${CARGO:-cargo}" build --release

mkdir -p ../../../desktop-resources/linux-x64
cp target/release/libvociferate_core.so ../../../desktop-resources/linux-x64/

"${CARGO:-cargo}" run --features=uniffi/cli --bin uniffi-bindgen generate \
  --library target/release/libvociferate_core.so \
  --language kotlin \
  --out-dir ../../commonMain/kotlin/

"${CARGO:-cargo}" build --release --target x86_64-pc-windows-gnu

mkdir -p ../../../desktop-resources/windows-x64
cp target/x86_64-pc-windows-gnu/release/vociferate_core.dll ../../../desktop-resources/windows-x64/ 2>/dev/null || \
cp target/x86_64-pc-windows-gnu/release/libvociferate_core.dll ../../../desktop-resources/windows-x64/