# Wrapped Allocators seem to cause issues with touch and don't seem to help with the OOMs 
#export RUSTFLAGS="$RUSTFLAGS -Clink-arg=-Wl,--wrap=malloc"
#export RUSTFLAGS="$RUSTFLAGS -Clink-arg=-Wl,--wrap=free"
#export RUSTFLAGS="$RUSTFLAGS -Clink-arg=-Wl,--wrap=calloc"
#export RUSTFLAGS="$RUSTFLAGS -Clink-arg=-Wl,--wrap=realloc"
#export RUSTFLAGS="$RUSTFLAGS -Clink-arg=-Wl,--wrap=memalign"
#export RUSTFLAGS="$RUSTFLAGS -Clink-arg=-Wl,--wrap=memcpy"
#export RUSTFLAGS="$RUSTFLAGS -Clink-arg=-Wl,--wrap=memset"
#export RUSTFLAGS="$RUSTFLAGS -Clink-arg=-Wl,-q"
export RUSTFLAGS="$RUSTFLAGS -Zthreads=16"
cargo vita build vpk --profile=vita