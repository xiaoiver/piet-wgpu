# piet-wgpu

![build status](https://github.com/xiaoiver/piet-wgpu/actions/workflows/rust.yml/badge.svg)

The [wgpu] backend for the [Piet 2D graphics abstraction].

Features:

- [-] Use [naga_oil] to combine and manipulate shader chunks.
- [-] Use SDF for rendering Circle, Ellipse, Rect and Text.
- [-] Use GPU extruding for Line, Polyline and Path.
- [-] Auto batching.

## Examples

### with_winit

```bash
$ cargo run -p with_winit
```

### wasm

We use [cargo-run-wasm] instead of [trunk] serving the wasm example. It seems that trunk has [ISSUE](https://github.com/trunk-rs/trunk/issues/445) with cargo workspace in watching mode.

```bash
$ cargo run_wasm -p with_winit
```

## Other libs

- [vello] An experimental GPU compute-centric 2D renderer.
- [vger-rs] Use an uber shader to draw 2D SDF for Circle, Rect and Path.
- [piet-wgpu] Tessellation with [lyon].
- [rough-rs] Rust port of Rough.js.

[Piet 2D graphics abstraction]: https://github.com/linebender/piet
[wgpu]: https://github.com/gfx-rs/wgpu
[naga_oil]: https://github.com/bevyengine/naga_oil
[cargo-run-wasm]: https://github.com/rukai/cargo-run-wasm
[trunk]: https://trunkrs.dev/
[vello]: https://github.com/linebender/vello
[vger-rs]: https://github.com/audulus/vger-rs
[piet-wgpu]: https://github.com/lapce/piet-wgpu/
[lyon]: https://github.com/nical/lyon
[rough-rs]: https://github.com/orhanbalci/rough-rs
