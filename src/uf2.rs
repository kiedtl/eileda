use std::sync::LazyLock;

use paste::paste;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use bitmaps::Bitmap;

macro_rules! fonts {
    ($($name:ident := @sz $sz:literal $p:literal),*,) => {
        $(paste! {
            #[allow(dead_code)]
            pub static [<FONT_ $name>]: LazyLock<Ufx<$sz>> =
                LazyLock::new(|| parse::<$sz>(include_bytes!($p)));
        })*
    };
}

#[rustfmt::skip]
fonts! {
    SHAVIAN12  := @sz 4  "../assets/shavian12.uf2",
    CREAM12    := @sz 4  "../assets/cream12.uf2",
    MONACO12   := @sz 4  "../assets/monaco12.uf2",
    CHICAGO12  := @sz 4  "../assets/chicago12.uf2",
    TIMES12    := @sz 4  "../assets/times12.uf2",
    NEWYORK12  := @sz 4  "../assets/newyork12.uf2",
    ANGELES12  := @sz 4  "../assets/losangeles12.uf2",
    GENEVA12   := @sz 4  "../assets/geneva12.uf2",
    PALATINO12 := @sz 4  "../assets/palatino12.uf2",

    GENEVA14   := @sz 4  "../assets/geneva14.uf2",
    PALATINO14 := @sz 4  "../assets/palatino14.uf2",
    VENICE14   := @sz 4  "../assets/venice14.uf2",
    NEWYORK14  := @sz 4  "../assets/newyork14.uf2",

    TIMES15    := @sz 4  "../assets/times15.uf2",
    NEWYORK34  := @sz 25 "../assets/newyork34.uf5",
}

#[derive(Copy, Clone, Debug)]
pub struct Ufx<const XS: usize> {
    pub glyphs: [UfGlyph<XS>; 256],
}

#[derive(Copy, Clone, Debug)]
pub struct UfGlyph<const XS: usize> {
    pub width: u8,
    pub inner: [Icn; XS],
}

impl<const XS: usize> Default for UfGlyph<XS> {
    fn default() -> Self {
        Self {
            width: 0,
            inner: [Icn {
                raw: Bitmap::new(),
            }; XS],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Icn {
    pub raw: Bitmap<64>,
}

pub fn parse<const X: usize>(bytes: &[u8]) -> Ufx<X> {
    let mut ufx = Ufx {
        glyphs: [Default::default(); 256],
    };

    let mut g_i = 0;
    let mut s_i = 0;
    let mut b_i = 0;

    for (i, byte) in bytes.iter().enumerate() {
        match i {
            0..255 => ufx.glyphs[i].width = *byte,
            _ => {
                ufx.glyphs[g_i].inner[s_i].raw.as_mut()[b_i] = *byte;
                b_i += 1;
                if b_i == 8 {
                    b_i = 0;
                    s_i += 1;
                    if s_i == X {
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

    ufx
}

pub fn draw_char<const XS: usize>(
    canvas: &mut WindowCanvas, font: &Ufx<XS>, sx: usize, sy: usize, ch: u8
) {
    #[allow(non_snake_case)]
    let X = (XS as f64).sqrt() as usize;

    let glyph: UfGlyph<XS> = font.glyphs[ch as usize];

    for x in 0..(glyph.width as usize) {
        for y in 0..(X * 8) {
            let sprite = ((x / 8) * X) + ((y / 8) % X);
            let pixel = ((y & 7) * 8) + (7 - (x & 7));
            if glyph.inner[sprite].raw.get(pixel as _) {
                canvas.draw_point(
                    Point::new(sx as i32 + (x as i32), sy as i32 + (y as i32))
                ).unwrap();
            }
        }
    }
}

pub fn draw<const XS: usize>(
    canvas: &mut WindowCanvas, font: &Ufx<XS>, lx: usize, sx: usize, sy: usize, s: &str
) -> (usize, usize)
{
    let mut x = sx;
    let mut y = sy;
    for ch in s.as_bytes() {
        if *ch == b'\n' {
            y += 16;
            x = lx;
            continue;
        }

        draw_char(canvas, font, x, y, *ch);
        x += font.glyphs[*ch as usize].width as usize;
    }

    (x, y)
}
