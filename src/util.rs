/// Converts HSL values to RGB values.
///
/// # Arguments
/// * `hue` - A floating point value between 0.0 and 360.0 that holds the hue
/// * `sat` - A floating point value between 0.0 and 1.0 that holds the saturation
/// * `lum` - A floating point value between 0.0 and 1.0 that holds the luminosity
///
/// # Usage
/// ```
/// use fractal_rs::util::hsl_rgb;
///
/// let hue = 120.0; // green
/// let sat = 0.80;  // lime green
/// let lum = 0.80;  // lime green
///
/// let color = hsl_rgb(hue, sat, lum);
/// println!("RGB[{}, {}, {}]", color.0, color.1, color.2);
/// ```
/// the example will print:
/// > RGB[16, 80, 16]
pub fn hsl_rgb(hue: f32, sat: f32, lum: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * lum - 1.0).abs()) * sat;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = lum - c / 2.0;
    let rgb_prime: (f32, f32, f32);
    let arc = hue as i32 / 60;
    match arc {
        0 => {
            rgb_prime = (c, x, 0.0);
        }
        1 => {
            rgb_prime = (x, c, 0.0);
        }
        2 => {
            rgb_prime = (0.0, c, x);
        }
        3 => {
            rgb_prime = (0.0, x, c);
        }
        4 => {
            rgb_prime = (x, 0.0, c);
        }
        _ => {
            rgb_prime = (c, 0.0, x);
        }
    }
    (
        ((rgb_prime.0 + m) * 255.0) as u8,        ((rgb_prime.1 + m) * 255.0) as u8,
        ((rgb_prime.2 + m) * 255.0) as u8,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    /// test hsl_to_rgb conversion
    #[test]
    fn test_hsl_rgb() {
        // red
        let result = hsl_rgb(0.0, 1.0, 0.5);
        assert_eq!(255, result.0);
        assert_eq!(0, result.1);
        assert_eq!(0, result.2);
        // lime
        let result = hsl_rgb(120.0, 1.0, 0.5);
        assert_eq!(0, result.0);
        assert_eq!(255, result.1);
        assert_eq!(0, result.2);
        // yellow
        let result = hsl_rgb(60.0, 1.0, 0.5);
        assert_eq!(255, result.0);
        assert_eq!(255, result.1);
        assert_eq!(0, result.2);
        // light purple
        let result = hsl_rgb(240.0, 0.75, 0.75);
        assert_eq!(143, result.0);
        assert_eq!(143, result.1);
        assert_eq!(239, result.2);
    }
}
