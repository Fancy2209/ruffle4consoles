# egui-sdl2-event

Provides event handling for [egui](https://github.com/emilk/egui) when SDL2 is used as the windowing system.

This crate does not perform any rendering, but it can be combined with something like
the [egui-wgpu](https://github.com/emilk/egui/tree/master/crates/egui-wgpu) backend.

Most of the code is just adaptively copied from [egui_sdl2_gl](https://github.com/ArjunNair/egui_sdl2_gl)

Simple [example](https://github.com/kaphula/egui-sdl2-event/tree/master/egui-sdl2-event-example) program.


## Changes

### 1.31.1

- Update egui to `1.31.1`
- Bump the version to follow egui version. 
- Change example to dynamic linking TODO