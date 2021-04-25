extern crate num;

pub mod util;

use num::complex::Complex64;
use util::hsl_rgb;

const DEFAULT_ZOOM: f64 = 0.003333333;

#[derive(Copy, Clone, Debug)]
pub enum PixelFormat {
    RGBA8,
    BGRA8,
}

#[derive(Copy, Clone, Debug)]
/// Configuration data for rendering a Mandelbrot fractal image
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub center: Complex64,
    pub zoom: f64,
    pub iter: u16,
    pub pix: PixelFormat,
}

impl Config {
    /// Simple helper to construct configuration without exposing Complex64 implementation
    pub fn new(
        width: u32,
        height: u32,
        cx: f64,
        cy: f64,
        zoom: f64,
        iter: u16,
        pix: PixelFormat,
    ) -> Config {
        Config {
            width,
            height,
            center: Complex64 { re: cx, im: cy },
            zoom,
            iter,
            pix,
        }
    }
}

/// Mandelbrot structure contains the fields necessary to render a Mandelbrot fractal.
pub struct Mandelbrot {
    config: Config,
    hist: Vec<u32>,
    iter_total: i32,
    viewport: (Complex64, Complex64),
}

impl Mandelbrot {
    //! Implementation to create an image based on the current configuration. The image
    //! data is stored in-memory as RGBA image data. The resultant image is an RGBA PNG
    //! image.
    //!
    //! * Example Usage
    //! ```
    //! extern crate num;
    //!
    //! use num::complex::Complex64;
    //! use fractal_rs::{Config, Mandelbrot, PixelFormat};
    //!
    //! let center = Complex64 { re: 0.0, im: 0.0 };
    //! let config = Config {
    //!     width: 600,
    //!     height: 400,
    //!     center,
    //!     zoom: 0.5,
    //!     iter: 100,
    //!     pix: PixelFormat::RGBA8,
    //! };
    //! let mut m = Mandelbrot::new(config);
    //! let img = m.render();
    //!
    //! ```
    //!
    //!

    /// Creates a new Mandelbrot struct with the Config passed in the parameter
    /// list. A viewport is constructed based on the configuration that factors
    /// in the width, height and zoom factor.
    pub fn new(config: Config) -> Mandelbrot {
        let zoom = DEFAULT_ZOOM / config.zoom;
        let viewport_height = config.height as f64 / 2.0 * zoom;
        let viewport_width = config.width as f64 / 2.0 * zoom;
        let viewport: (Complex64, Complex64) = (
            Complex64::new(
                config.center.re - viewport_width,
                config.center.im + viewport_height,
            ),
            Complex64::new(
                config.center.re + viewport_width,
                config.center.im - viewport_height,
            ),
        );
        let hist = Vec::with_capacity(config.iter as usize);
        Mandelbrot {
            config,
            hist,
            iter_total: 0,
            viewport,
        }
    }

    /// Gets the current viewport. The viewport is returned as a tuple (top-left, bottom-right)
    pub fn viewport(&self) -> (Complex64, Complex64) {
        (self.viewport.0, self.viewport.1)
    }

    /// Creates in-memory image data based on the current configuration.
    pub fn render(&mut self) -> Vec<u8> {
        let mut image = Vec::with_capacity((self.config.width * self.config.height * 4) as usize);
        let mut image_iter = Vec::with_capacity((self.config.width * self.config.height) as usize);
        // prepare the histogram
        for _ in 0..self.config.iter as usize {
            self.hist.push(0);
        }
        let x_step = (self.viewport.1.re - self.viewport.0.re).abs() / self.config.width as f64;
        let y_step = (self.viewport.0.im - self.viewport.1.im).abs() / self.config.height as f64;
        // pass 1 - populate the raw pixel values and histogram
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let c = Complex64 {
                    re: self.viewport.0.re + (x as f64 * x_step),
                    im: self.viewport.0.im - (y as f64 * y_step),
                };
                let iter = self.calc_point(c);
                if iter < self.config.iter as f64 {
                    self.hist[iter.floor() as usize] += 1;
                    self.iter_total += 1;
                }
                image_iter.push(iter);
            }
        }
        // pass 2 - normalize the histogram 0.0 - 1.0
        let mut hues: Vec<f32> = Vec::with_capacity((self.config.iter + 1) as usize);
        let mut val: f32 = 0.0;
        for i in 0..self.config.iter as usize {
            val += self.hist[i] as f32 / self.iter_total as f32;
            hues.push(val);
        }
        hues.push(0.0);
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let idx = (y * self.config.width + x) as usize;
                let m = image_iter[idx];
                let mut hue: f32 = 0.0;
                // we allow direct float comparison since this is not comparing 2 calculated values
                // but rather 2 previously stored floats
                #[allow(clippy::float_cmp)]
                if m != self.config.iter as f64 {
                    hue = 360.0
                        - Mandelbrot::linear_interpolation(
                            hues[m.floor() as usize] as f64,
                            hues[m.ceil() as usize] as f64,
                            m % 1.0,
                        ) * 360.0;
                }
                // we allow direct float comparison since this is not comparing 2 calculated values
                // but rather 2 previously stored floats
                #[allow(clippy::float_cmp)]
                if image_iter[idx] != self.config.iter as f64 {
                    let rgb = Mandelbrot::point_color(hue);
                    // set the image colors
                    match self.config.pix {
                        PixelFormat::RGBA8 => {
                            image.push(rgb.0);
                            image.push(rgb.1);
                            image.push(rgb.2);
                            image.push(255u8);
                        }
                        PixelFormat::BGRA8 => {
                            image.push(rgb.2);
                            image.push(rgb.1);
                            image.push(rgb.0);
                            image.push(255u8);
                        }
                    }
                } else {
                    image.push(0u8);
                    image.push(0u8);
                    image.push(0u8);
                    image.push(255u8);
                }
            }
        }
        image
    }

    fn linear_interpolation(a: f64, b: f64, t: f64) -> f32 {
        (a * (1.0 - t) + b * t) as f32
    }

    fn calc_point(&self, c: Complex64) -> f64 {
        let mut z: Complex64 = Complex64::new(0.0, 0.0);
        let mut iter: u16 = 0;

        while (z.norm_sqr() <= 4.0) && (iter < self.config.iter) {
            z = (z * z) + c;
            iter += 1;
        }
        if iter == self.config.iter {
            return iter as f64;
        }
        let abs_z = z.norm_sqr().sqrt();
        iter as f64 + 1.0_f64 - abs_z.log2().log10()
    }

    fn point_color(hue: f32) -> (u8, u8, u8) {
        let lum = 0.5;
        let sat = 0.90;
        hsl_rgb(hue, sat, lum)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// test basic initialization
    #[test]
    fn test_std_config() {
        let config = Config {
            width: 1200,
            height: 800,
            center: Complex64::new(-0.0, 0.0),
            zoom: 1.0,
            iter: 720,
            pix: PixelFormat::RGBA8,
        };
        let m = Mandelbrot::new(config);
        assert!(
            (-2.000 - m.viewport.0.re).abs() < 0.01,
            "viewport left should be -2.0"
        );
        assert!(
            (1.3333 - m.viewport.0.im).abs() < 0.01,
            "viewport top should be 1.333i"
        );
        assert!(
            (2.000 - m.viewport.1.re).abs() < 0.01,
            "viewport right should be 2.0"
        );
        assert!(
            (-1.3333 - m.viewport.1.im).abs() < 0.01,
            "viewport bottom should be = 1.333i"
        );
    }

    /// test render method basic functionality
    #[test]
    fn test_render() {
        let config = Config {
            width: 100,
            height: 100,
            center: Complex64::new(-0.0, 0.0),
            zoom: 0.1,
            iter: 100,
            pix: PixelFormat::RGBA8,
        };
        let mut m = Mandelbrot::new(config);
        let image = m.render();
        assert_eq!(40_000, image.len(), "expected len 40_000");
        // in this image the  center pixels should be black
        // check 10 pixel in center row
        for i in (20_000..20040).step_by(4) {
            assert_eq!(0, image[i], "expected 0 for B");
            assert_eq!(0, image[i + 1], "expected 0 for G");
            assert_eq!(0, image[i + 2], "expected 0 for R");
        }
        // upper left should be green-blue
        assert!(image[397] > 0, "G should be > 0");
        assert!(image[396] < image[397], "B should be less than G");
        assert!(image[398] < image[397], "R should be less than G");
    }
}
