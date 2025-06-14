/// Each component is stored as f32 in a normalized range
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let h = h.rem_euclid(360.0);
        let s = f32::clamp(s, 0.0, 1.0);
        let l = f32::clamp(l, 0.0, 1.0);

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match h {
            0.0..=60.0 => (c, x, 0.0),
            60.0..=120.0 => (x, c, 0.0),
            120.0..=180.0 => (0.0, c, x),
            180.0..=240.0 => (0.0, x, c),
            240.0..=300.0 => (x, 0.0, c),
            300.0..=360.0 => (c, 0.0, x),
            _ => unreachable!(),
        };

        Self::new(r + m, g + m, b + m)
    }

    pub const fn to_quantized(&self) -> [u8; 3] {
        [
            f32::clamp(self.r * 255.0, 0.0, 255.0) as u8,
            f32::clamp(self.g * 255.0, 0.0, 255.0) as u8,
            f32::clamp(self.b * 255.0, 0.0, 255.0) as u8,
        ]
    }

    pub const fn from_quantized(r: u8, g: u8, b: u8) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }

    pub const fn is_black(&self) -> bool {
        let [r, g, b] = self.to_quantized();
        (r == 0) && (g == 0) && (b == 0)
    }

    pub fn to_hex(&self) -> String {
        let [r, g, b] = self.to_quantized();
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }

    pub fn from_hex(hex: &str) -> Option<Color> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        match hex.chars().collect::<Box<[char]>>().as_ref() {
            [c_r, c_g, c_b] | [c_r, c_g, c_b, _] => Some(Color::new(
                (c_r.to_digit(16)? as f32) / 15.0,
                (c_g.to_digit(16)? as f32) / 15.0,
                (c_b.to_digit(16)? as f32) / 15.0,
            )),
            [c1_r, c2_r, c1_g, c2_g, c1_b, c2_b] | [c1_r, c2_r, c1_g, c2_g, c1_b, c2_b, _, _] => {
                Some(Color::new(
                    (((c1_r.to_digit(16)? << 4) | c2_r.to_digit(16)?) as f32) / 255.0,
                    (((c1_g.to_digit(16)? << 4) | c2_g.to_digit(16)?) as f32) / 255.0,
                    (((c1_b.to_digit(16)? << 4) | c2_b.to_digit(16)?) as f32) / 255.0,
                ))
            }
            _ => None,
        }
    }

    pub fn difference(&self, other: &Color) -> f32 {
        Lab::from_srgb(self).difference(&Lab::from_srgb(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Lab {
    l: f32,
    a: f32,
    b: f32,
}

impl Lab {
    // https://stackoverflow.com/questions/9018016#answer-26998429
    fn from_srgb(srgb: &Color) -> Self {
        const EPS: f32 = 216.0 / 24389.0;
        const K: f32 = 24389.0 / 27.0;
        const XR: f32 = 0.964221;
        const YR: f32 = 1.0;
        const ZR: f32 = 0.825211;

        fn f1(v: f32) -> f32 {
            if v < 0.04045 {
                v / 12.0
            } else {
                f32::powf((v + 0.055) / 1.055, 2.4)
            }
        }

        let r = f1(srgb.r);
        let g = f1(srgb.b);
        let b = f1(srgb.b);

        #[allow(clippy::excessive_precision)]
        let x = 0.436052025 * r + 0.385081593 * g + 0.143087414 * b;
        #[allow(clippy::excessive_precision)]
        let y = 0.222491598 * r + 0.716886060 * g + 0.060621486 * b;
        #[allow(clippy::excessive_precision)]
        let z = 0.013929122 * r + 0.097097002 * g + 0.714185470 * b;

        let xr = x / XR;
        let yr = y / YR;
        let zr = z / ZR;

        fn f2(v: f32) -> f32 {
            if v > EPS {
                f32::powf(v, 1.0 / 3.0)
            } else {
                (K * v + 16.0) / 116.0
            }
        }

        let fx = f2(xr);
        let fy = f2(yr);
        let fz = f2(zr);

        let l = (116.0 * fy) - 16.0;
        let a = 500.0 * (fx - fy);
        let b = 200.0 * (fy - fz);

        Self {
            l: 2.55 * l + 0.5,
            a: a + 0.5,
            b: b + 0.5,
        }
    }

    fn difference(&self, other: &Lab) -> f32 {
        ((self.l - other.l).powi(2) + (self.a - other.a).powi(2) + (self.b - other.b).powi(2))
            .sqrt()
    }
}

#[cfg(test)]
mod test {
    use crate::Color;

    #[test]
    fn test() {
        fn test_hex(r: u8, g: u8, b: u8) {
            let color = Color::from_quantized(r, g, b);
            assert_eq!(Color::from_hex(&color.to_hex()).unwrap(), color);
        }

        test_hex(255, 127, 0);
        test_hex(69, 42, 127);
    }
}
