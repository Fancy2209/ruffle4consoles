export PKG_CONFIG_DIR= 
export PKG_CONFIG_PATH= 
export PKG_CONFIG_SYSROOT_DIR= 
export PKG_CONFIG_LIBDIR=$DEVKITPRO/portlibs/switch/lib/pkgconfig 
cargo build -Z build-std=core,alloc,std,panic_abort --target aarch64-nintendo-switch.json --profile=switch -v
nacptool --create 'ruffle' 'ruffle contributors' '0.1.0' target/aarch64-nintendo-switch/switch/ruffle4consoles.nacp
elf2nro target/aarch64-nintendo-switch/switch/ruffle4consoles.elf target/aarch64-nintendo-switch/switch/ruffle4consoles.nro \
  --icon=icon.jpg \
  --nacp=target/aarch64-nintendo-switch/switch/ruffle4consoles.nacp
