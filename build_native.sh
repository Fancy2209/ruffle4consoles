# TODO: There is an issue with building SDL2 RS on armv7, need to investigate
cargo ndk -t arm64-v8a -t x86_64 -o ./android-project/app/libs build --release
