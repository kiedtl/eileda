use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use bitmaps::Bitmap;

pub const CHICAGO12: &[u8] = include_bytes!("../assets/chicago12.uf2");

#[derive(Copy, Clone, Debug)]
pub struct Uf2 {
    pub glyphs: [UfGlyph2; 256],
}

#[derive(Copy, Clone, Debug)]
pub struct UfGlyph2 {
    pub width: u8,
    pub inner: [Icn; 4],
}

impl Default for UfGlyph2 {
    fn default() -> Self {
        Self {
            width: 0,
            inner: [Icn {
                raw: Bitmap::new(),
            }; 4],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Icn {
    pub raw: Bitmap<64>,
}

pub fn parse(bytes: &[u8]) -> Uf2 {
    let mut uf2 = Uf2 {
        glyphs: [Default::default(); 256],
    };

    let mut g_i = 0;
    let mut s_i = 0;
    let mut b_i = 0;

    for (i, byte) in bytes.iter().enumerate() {
        match i {
            0..255 => uf2.glyphs[i].width = *byte,
            _ => {
                uf2.glyphs[g_i].inner[s_i].raw.as_mut()[b_i] = *byte;
                b_i += 1;
                if b_i == 8 {
                    b_i = 0;
                    s_i += 1;
                    if s_i == 4 {
                        s_i = 0;
                        g_i += 1;
                        if g_i == 256 {
                            break;
                        }
                    }
                }
            },
        }
    }

    uf2
}

pub fn draw_char(
    canvas: &mut WindowCanvas, font: &Uf2, sx: usize, sy: usize, ch: u8
) {
    let glyph: UfGlyph2 = font.glyphs[ch as usize];

    for x in 0..glyph.width {
        for y in 0..16 {
            let sprite = match (x, y) {
                (0..=7, 0..=7) => 0,
                (0..=7, 8..=15) => 1,
                (8..=15, 0..=7) => 2,
                (8..=15, 8..=15) => 3,
                _ => unreachable!(),
            };

            let i = ((y & 7) * 8) + (7 - (x & 7));
            if glyph.inner[sprite].raw.get(i as _) {
                canvas.draw_point(
                    Point::new(sx as i32 + (x as i32), sy as i32 + (y as i32))
                ).unwrap();
            }
        }
    }
}

pub fn draw_string(
    canvas: &mut WindowCanvas, font: &Uf2, sx: usize, sy: usize, s: &str
) {
    let mut x = sx;
    let mut y = sy;
    for ch in s.as_bytes() {
        if *ch == b'\n' {
            y += 16;
            x = sx;
            continue;
        }

        draw_char(canvas, font, x, y, *ch);
        x += font.glyphs[*ch as usize].width as usize;
    }
}
