use std::fs;

use crate::uf2;

use bitflags::bitflags;
use markdown;
use sdl2::render::WindowCanvas;

#[derive(Clone, Debug)]
pub enum Item {
    Pad(usize),
    Md(markdown::mdast::Node),
}

pub fn parse(file: &str) -> Vec<Item> {
    let data = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Nuh uh");
            return Vec::new();
        },
    };

    let mut items = Vec::new();
    let mut mdbuf = String::new();

    for line in data.lines() {
        if line.starts_with(".") {
            items.push(Item::Md(markdown::to_mdast(&mdbuf, &Default::default()).unwrap()));
            mdbuf.clear();

            let cmd = line.split(" ").collect::<Vec<_>>();
            match cmd[0] {
                ".PAD" if cmd.len() == 2 => {
                    items.push(Item::Pad(cmd[1].parse().unwrap_or(0)));
                },
                c => eprintln!("Error: Unknown or invalid directive: {}", c),
            }
        } else {
            mdbuf.push_str(&(line.to_owned() + "\n"));
        }
    }

    items.push(Item::Md(markdown::to_mdast(&mdbuf, &Default::default()).unwrap()));
    mdbuf.clear();

    items
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

pub fn draw(canvas: &mut WindowCanvas, items: Vec<Item>) {
    let mut lx = 0;
    let mut ex = 960 / 2;
    let mut y = 0;

    for item in items {
        match item {
            Item::Pad(pad) => {
                lx += pad;
                ex -= pad;
                y += pad;
            },
            Item::Md(md) => {
                let (_, ny) = draw_node(canvas, &md, lx, ex, lx, y, DrawFl::NONE);
                y = ny;
            },
        }
    }
}

fn draw_node(
    canvas: &mut WindowCanvas,
    node: &markdown::mdast::Node,
    lx: usize,
    ex: usize,
    sx: usize,
    sy: usize,
    fl: DrawFl,
) -> (usize, usize)
{
    const PAR_PAD: usize = 12;
    const LST_MAR: usize = 8;

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
                let (nx, ny) = draw_node(canvas, c, lx, ex, x, y, fl);
                x = nx;
                y = ny;
            }
            x = sx;
        },
        N::Paragraph(Paragraph { children, .. }) => {
            for c in children {
                let (nx, ny) = draw_node(canvas, c, lx, ex, x, y, fl);
                x = nx;
                y = ny;
            }
            x = sx;
            y += 2 * 8;
            y += PAR_PAD;
        },
        N::Heading(Heading { children, depth: _, .. }) => {
            for c in children {
                let (nx, ny) = draw_node(canvas, c, lx, ex, x, y, fl | DrawFl::HEADER);
                x = nx;
                y = ny;
            }
            x = sx;
            y += 5 * 8;
            y += PAR_PAD;
        },
        N::Strong(Strong { children, .. }) => for c in children {
            let (nx, ny) = draw_node(canvas, c, lx, ex, x, y, fl | DrawFl::BOLD);
            x = nx;
            y = ny;
        },
        N::Emphasis(Emphasis { children, .. }) => for c in children {
            let (nx, ny) = draw_node(canvas, c, lx, ex, x, y, fl | DrawFl::EMPH);
            x = nx;
            y = ny;
        },
        N::List(List { children, ordered, start, spread: _, .. }) => {
            for (i, c) in children.iter().enumerate() {
                let o;
                if !*ordered {
                    uf2::draw_char(canvas, &*uf2::FONT_NEWYORK14, x + LST_MAR, y, 0xA5);
                    o = LST_MAR
                        + uf2::FONT_NEWYORK14.glyphs[0xA5].width as usize
                        + uf2::FONT_NEWYORK14.glyphs[0x20].width as usize;
                } else {
                    let l = &format!("{}) ", start.unwrap_or(1) as usize + i);
                    uf2::draw(canvas, &*uf2::FONT_NEWYORK14, lx + LST_MAR, ex, x + LST_MAR, y, l);
                    o = LST_MAR + uf2::measure(&*uf2::FONT_NEWYORK14, &l);
                }
                let (nx, ny) = draw_node(canvas, c, lx + o, ex, x + o, y, fl);
                x = nx - o;
                y = ny;
            }
            x = sx;
            y += PAR_PAD;
        },
        N::ListItem(ListItem { children, .. }) => for c in children {
            let (nx, ny) = draw_node(canvas, c, lx, ex, x, y, fl);
            x = nx;
            y = ny - PAR_PAD;
        },
        N::Text(Text { value, .. }) => {
            let (nx, ny) = if fl.contains(DrawFl::HEADER) {
                uf2::draw(canvas, &*uf2::FONT_NEWYORK34, lx, ex, x, y, &value)
            } else {
                uf2::draw(canvas, fnt, lx, ex, x, y, &value)
            };
            x = nx;
            y = ny;
        },
        n => println!("Node not implemented: {:?}", n),
    }

    (x, y)
}
