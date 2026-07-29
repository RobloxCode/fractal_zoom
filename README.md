# fractal_zoom

An interactive Mandelbrot / Julia set viewer written in Rust. Renders on the
CPU into a window via `minifb` — no GPU setup, no shaders, just pixels.

> **Note:** this project was vibecoded — built by prompting Claude rather
> than written by hand line-by-line. It was compiled and tested in a sandbox
> to make sure it actually builds and runs, but treat it as a starting point
> to poke at and modify, not battle-tested production code.

## Build & run

```bash
cd fractal_zoom
cargo build --release
cargo run --release
```

First build downloads dependencies from crates.io, so you need internet
access once. On Linux you also need X11 dev headers if they aren't already
installed (most desktop distros have them):

```bash
sudo apt install libx11-dev libxkbcommon-dev   # Debian/Ubuntu
```

## Controls

| Input                 | Action                                      |
|------------------------|----------------------------------------------|
| Scroll wheel           | Zoom in/out, centered on the cursor          |
| Left-click + drag      | Pan                                          |
| `R`                    | Reset view                                   |
| `J`                    | Toggle between Mandelbrot and Julia set      |
| `+` / `-`               | Increase/decrease max iterations (more detail vs. more speed) |
| `Esc`                  | Quit                                         |

## How it works

- `View` holds the current center point, zoom scale, and iteration cap.
- Each frame, every pixel is mapped to a point in the complex plane and run
  through the standard escape-time algorithm (`escape_iters`) to decide how
  "inside" or "outside" the set it is.
- Iteration counts are mapped to color with a simple polynomial palette
  (`color_for`) — feel free to swap in your own gradient.
- Zooming keeps the point under your cursor fixed in place, rather than
  always zooming toward the center, so you can scroll toward a spot you want
  to explore.

## Known limitations

- Zoom is `f64`-based, so precision runs out around a scale of ~1e-13 —
  past that the image will start to pixelate/degrade instead of revealing
  more detail. Getting further would need arbitrary-precision arithmetic.
- Rendering is single-threaded and CPU-only, so deep zooms with a high
  iteration count can get slow. Parallelizing the per-pixel loop (e.g. with
  `rayon`) would be a natural next step.
- `Cargo.toml` builds `minifb` with only the `x11` backend enabled (no
  Wayland) to keep the dependency tree small; it still works fine under
  XWayland. For native Wayland support, change the `minifb` line to
  `minifb = "0.24"` (default features).
