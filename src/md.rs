use std::fs;

use crate::uf2;

use bitflags::bitflags;
use markdown;
use sdl2::render::WindowCanvas;

pub fn parse(file: &str) -> Option<markdown::mdast::Node> {
    let data = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Nuh uh");
            return None;
        },
    };

    Some(markdown::to_mdast(&data, &Default::default()).unwrap())
}

bitflags! {
    #[derive(PartialEq, Copy, Clone)]
    pub struct DrawFl: u16 {
        const NONE = 0b00;
        const BOLD = 0b01;
        const EMPH = 0b10;
        const HEADER = 0b11;
    }
}

pub fn draw(
    canvas: &mut WindowCanvas,
    node: &markdown::mdast::Node,
    lx: usize,
    sx: usize,
    sy: usize,
    fl: DrawFl,
) -> (usize, usize)
{
    const PAR_PAD: usize = 12;

    use markdown::mdast::*;
    use markdown::mdast::Node as N;

    let mut x = sx;
    let mut y = sy;

    let fnt = if fl.contains(DrawFl::BOLD) {
        &*uf2::FONT_VENICE14
    } else if fl.contains(DrawFl::EMPH) {
        &*uf2::FONT_ANGELES12
    } else {
        &*uf2::FONT_GENEVA12
    };

    match node {
        N::Root(Root{ children, .. }) => {
            for c in children {
                let (nx, ny) = draw(canvas, c, lx, x, y, fl);
                x = nx;
                y = ny;
            }
            x = sx;
        },
        N::Paragraph(Paragraph { children, .. }) => {
            for c in children {
                let (nx, ny) = draw(canvas, c, lx, x, y, fl);
                x = nx;
                y = ny;
            }
            x = sx;
            y += 2 * 8;
            y += PAR_PAD;
        },
        N::Heading(Heading { children, depth: _, .. }) => {
            for c in children {
                let (nx, ny) = draw(canvas, c, lx, x, y, fl | DrawFl::HEADER);
                x = nx;
                y = ny;
            }
            x = sx;
            y += 5 * 8;
        },
        N::Strong(Strong { children, .. }) => for c in children {
            let (nx, ny) = draw(canvas, c, lx, x, y, fl | DrawFl::BOLD);
            x = nx;
            y = ny;
        },
        N::Emphasis(Emphasis { children, .. }) => for c in children {
            let (nx, ny) = draw(canvas, c, lx, x, y, fl | DrawFl::EMPH);
            x = nx;
            y = ny;
        },
        N::List(List { children, ordered, start, spread: _, .. }) => {
            for (i, c) in children.iter().enumerate() {
                let o;
                if !*ordered {
                    uf2::draw_char(canvas, &*uf2::FONT_NEWYORK14, x, y, 0xA5);
                    o = 0
                        + uf2::FONT_NEWYORK14.glyphs[0xA6].width as usize
                        + uf2::FONT_NEWYORK14.glyphs[0x20].width as usize;
                } else {
                    let n = start.unwrap_or(1) as usize + i;
                    o = uf2::draw(canvas, &*uf2::FONT_NEWYORK14, lx, x, y, &format!("{}) ", n)).0;
                }
                let (nx, ny) = draw(canvas, c, lx + o, x + o, y, fl);
                x = nx - o;
                y = ny;
            }
            x = sx;
            y += PAR_PAD;
        },
        N::ListItem(ListItem { children, .. }) => for c in children {
            let (nx, ny) = draw(canvas, c, lx, x, y, fl);
            x = nx;
            y = ny - PAR_PAD;
        },
        N::Text(Text { value, .. }) => {
            let (nx, ny) = if fl.contains(DrawFl::HEADER) {
                uf2::draw(canvas, &*uf2::FONT_NEWYORK34, lx, x, y, &value)
            } else {
                uf2::draw(canvas, fnt, lx, x, y, &value)
            };
            x = nx;
            y = ny;
        },
        n => println!("Node not implemented: {:?}", n),
    }

    (x, y)
}
