use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};

const WIDTH: usize = 900;
const HEIGHT: usize = 600;

/// Camera / view state: what region of the complex plane we're looking at.
struct View {
    center_x: f64,
    center_y: f64,
    // "scale" = half-width of the view in the complex plane.
    scale: f64,
    max_iter: u32,
    julia_mode: bool,
    julia_c: (f64, f64),
}

impl View {
    fn new() -> Self {
        View {
            center_x: -0.5,
            center_y: 0.0,
            scale: 1.5,
            max_iter: 200,
            julia_mode: false,
            julia_c: (-0.7, 0.27015),
        }
    }

    fn reset(&mut self) {
        if self.julia_mode {
            self.center_x = 0.0;
            self.center_y = 0.0;
            self.scale = 1.5;
        } else {
            self.center_x = -0.5;
            self.center_y = 0.0;
            self.scale = 1.5;
        }
    }

    // Convert a pixel coordinate to a point in the complex plane.
    fn pixel_to_complex(&self, px: f64, py: f64) -> (f64, f64) {
        let aspect = WIDTH as f64 / HEIGHT as f64;
        let x = self.center_x + (px / WIDTH as f64 - 0.5) * 2.0 * self.scale * aspect;
        let y = self.center_y + (py / HEIGHT as f64 - 0.5) * 2.0 * self.scale;
        (x, y)
    }
}

fn escape_iters(c_re: f64, c_im: f64, z0_re: f64, z0_im: f64, max_iter: u32) -> u32 {
    let mut zr = z0_re;
    let mut zi = z0_im;
    let mut n = 0;
    while zr * zr + zi * zi <= 4.0 && n < max_iter {
        let new_zr = zr * zr - zi * zi + c_re;
        let new_zi = 2.0 * zr * zi + c_im;
        zr = new_zr;
        zi = new_zi;
        n += 1;
    }
    n
}

// Smooth-ish palette: map iteration count to an RGB color.
fn color_for(n: u32, max_iter: u32) -> u32 {
    if n >= max_iter {
        return 0x000000;
    }
    let t = n as f64 / max_iter as f64;
    let r = (9.0 * (1.0 - t) * t * t * t * 255.0) as u32;
    let g = (15.0 * (1.0 - t) * (1.0 - t) * t * t * 255.0) as u32;
    let b = (8.5 * (1.0 - t) * (1.0 - t) * (1.0 - t) * t * 255.0) as u32;
    (r << 16) | (g << 8) | b
}

fn render(buffer: &mut [u32], view: &View) {
    for py in 0..HEIGHT {
        for px in 0..WIDTH {
            let (x, y) = view.pixel_to_complex(px as f64, py as f64);
            let n = if view.julia_mode {
                escape_iters(view.julia_c.0, view.julia_c.1, x, y, view.max_iter)
            } else {
                escape_iters(x, y, 0.0, 0.0, view.max_iter)
            };
            buffer[py * WIDTH + px] = color_for(n, view.max_iter);
        }
    }
}

fn main() {
    let mut view = View::new();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Fractal Zoom - scroll to zoom, drag to pan, R reset, J julia, +/- iterations, Esc quit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut dragging = false;
    let mut drag_start_mouse: (f32, f32) = (0.0, 0.0);
    let mut drag_start_center: (f64, f64) = (0.0, 0.0);
    let mut needs_redraw = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // --- Zoom via scroll wheel, centered on the cursor ---
        if let Some((_, scroll_y)) = window.get_scroll_wheel() {
            if scroll_y != 0.0 {
                if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
                    // Point under the cursor before zoom.
                    let (before_x, before_y) = view.pixel_to_complex(mx as f64, my as f64);

                    let zoom_factor = if scroll_y > 0.0 { 0.9 } else { 1.0 / 0.9 };
                    view.scale *= zoom_factor;
                    view.scale = view.scale.clamp(1e-13, 4.0);

                    // Recompute where that same pixel maps to after zoom,
                    // and shift the center so the point under the cursor stays put.
                    let (after_x, after_y) = view.pixel_to_complex(mx as f64, my as f64);
                    view.center_x += before_x - after_x;
                    view.center_y += before_y - after_y;
                }
                needs_redraw = true;
            }
        }

        // --- Pan via left-click drag ---
        if window.get_mouse_down(MouseButton::Left) {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Pass) {
                if !dragging {
                    dragging = true;
                    drag_start_mouse = (mx, my);
                    drag_start_center = (view.center_x, view.center_y);
                } else {
                    let aspect = WIDTH as f64 / HEIGHT as f64;
                    let dx = (mx - drag_start_mouse.0) as f64;
                    let dy = (my - drag_start_mouse.1) as f64;
                    view.center_x =
                        drag_start_center.0 - (dx / WIDTH as f64) * 2.0 * view.scale * aspect;
                    view.center_y = drag_start_center.1 - (dy / HEIGHT as f64) * 2.0 * view.scale;
                    needs_redraw = true;
                }
            }
        } else {
            dragging = false;
        }

        // --- Keyboard controls ---
        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            view.reset();
            needs_redraw = true;
        }
        if window.is_key_pressed(Key::J, minifb::KeyRepeat::No) {
            view.julia_mode = !view.julia_mode;
            view.reset();
            needs_redraw = true;
        }
        if window.is_key_pressed(Key::Equal, minifb::KeyRepeat::Yes) {
            view.max_iter = (view.max_iter + 50).min(5000);
            needs_redraw = true;
        }
        if window.is_key_pressed(Key::Minus, minifb::KeyRepeat::Yes) {
            view.max_iter = (view.max_iter.saturating_sub(50)).max(50);
            needs_redraw = true;
        }

        if needs_redraw {
            render(&mut buffer, &view);
            needs_redraw = false;
        }

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("Failed to update window buffer");
    }
}
