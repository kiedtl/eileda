use std::cell::RefCell;

use crate::uf2;

use bitflags::bitflags;
use markdown;

use sdl2::ttf::Font as Sdl2Font;
use sdl2::ttf::FontStyle as Sdl2FontStyle;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::WindowCanvas;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

const PAR_PAD: usize = 12;
const LST_MAR: usize = 8;
const IMG_SPC: usize = 12;
const COL_SPC: usize = 12;

pub struct Margin<'a> {
    pub image: Texture<'a>,
    pub middle: usize,
}

pub struct GlobalConfig<'a> {
    pub padding: usize,
    pub margin: Option<Margin<'a>>,
    pub ttf: bool,
}

pub struct Presentation<'a> {
    pub tcreator: &'a TextureCreator<WindowContext>,
    pub config: GlobalConfig<'a>,
    pub slides: Vec<Slide<'a>>,
    pub fonts: Vec<RefCell<Sdl2Font<'a, 'a>>>,
}

pub struct Slide<'a> {
    pub title: Option<String>,
    pub content: Vec<Content<'a>>,
}

pub enum Content<'a> {
    Grid(Grid<'a>),
    Md(markdown::mdast::Node),
    Img(Texture<'a>),
    Dummy,
}

pub struct Grid<'a> {
    pub ratio: usize,
    pub first: Vec<Content<'a>>,
    pub second: Vec<Content<'a>>,

    pub _parser_col_adv: bool,
}

impl<'a> Presentation<'a> {
    pub fn draw(&self, slide: usize, canvas: &mut WindowCanvas) {
        const S: usize = 2;
        const W: usize = 960 / S;
        const H: usize = 840 / S;

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(0, 0, 0));

        let mut lx = 0;
        let mut ex = W;
        let mut sy = 0;
        let mut ey = H;

        if let Some(ref margin) = self.config.margin {
            let (iw, ih) = (
                margin.image.query().width as u32,
                margin.image.query().height as u32,
            ); // image w/h
            let x_pad = (ex - lx).saturating_sub(margin.middle / S) / 2;
            ex -= x_pad;
            lx += x_pad;

            let (lmw, lmh) = (lx as u32, H as u32); // left margin width/height
            canvas.copy(
                &margin.image,
                Some(Rect::new(0, 0, lmw.min(iw), lmh.min(ih) as _)),
                Some(Rect::new(0, 0, lmw, lmh)),
            ).unwrap();

            let (rmw, rmh) = ((W - ex) as u32, H as u32); // right margin width/height
            let img_start = iw.saturating_sub(rmw) as i32;
            canvas.copy(
                &margin.image,
                Some(Rect::new(img_start, 0, rmw.min(iw), rmh.min(ih))),
                Some(Rect::new(ex as _, 0, rmw, rmh)),
            ).unwrap();
        }

        if self.config.padding > 0 {
            lx += self.config.padding;
            ex -= self.config.padding;
            sy += self.config.padding;
            ey -= self.config.padding;
        }

        if let Some(ref title) = self.slides[slide].title {
            let (_, dy) = uf2::draw(canvas, &*uf2::FONT_NEWYORK34, lx, ex, lx, sy, title);
            sy += dy + (PAR_PAD * 3);
        }

        draw_content(self, canvas, &self.slides[slide].content, lx, ex, sy, ey);
    }
}

bitflags! {
    #[derive(PartialEq, Copy, Clone)]
    pub struct DrawFl: u16 {
        const NONE = 0b000;
        const BOLD = 0b001;
        const EMPH = 0b010;
        const HEAD = 0b100;
    }
}

fn draw_content<'a>(
    p: &Presentation,
    canvas: &mut WindowCanvas,
    content: &Vec<Content<'a>>,
    lx: usize,
    ex: usize,
    sy: usize,
    ey: usize,
) -> usize {
    let mut y = sy;

    for item in content {
        match item {
            Content::Dummy => (),
            Content::Grid(Grid {
                ratio,
                first,
                second,
                ..
            }) => {
                let bx = lx + ((ex - lx) * ratio / 100);
                let y1 = draw_content(p, canvas, first, lx, bx, y, ey);
                let y2 = draw_content(p, canvas, second, bx + COL_SPC, ex, y, ey);
                y = y1.max(y2);
            }
            Content::Img(t) => {
                let (iw, ih) = (t.query().width, t.query().height); // image w/h
                let (mw, mh) = ((ex - lx) as _, (ey - y) as _); // max w/h
                let (cw, ch); // calculated w/h

                // Scale image down into boundaries, depending on which dimension
                // (height/width) is over bounds most
                if iw < mw && ih < mh {
                    cw = iw;
                    ch = ih;
                } else if iw.saturating_sub(mw) > ih.saturating_sub(mh) {
                    cw = mw;
                    ch = (ih as f32 * (mw as f32 / iw as f32)) as u32;
                } else {
                    ch = mh;
                    cw = (iw as f32 * (mh as f32 / ih as f32)) as u32;
                }

                let dst = Rect::new(lx as _, y as _, cw, ch);
                canvas.copy(t, None, Some(dst)).unwrap();

                y += ih as usize + IMG_SPC;
            }
            Content::Md(md) => {
                let (_, ny) = draw_md(p, canvas, &md, lx, ex, lx, y, DrawFl::NONE);
                y = ny;
            }
        }
    }

    y
}

fn draw_md(
    p: &Presentation,
    canvas: &mut WindowCanvas,
    node: &markdown::mdast::Node,
    lx: usize,
    ex: usize,
    sx: usize,
    sy: usize,
    fl: DrawFl,
) -> (usize, usize) {
    use markdown::mdast::Node as N;
    use markdown::mdast::*;

    let mut x = sx;
    let mut y = sy;

    let fnt = if fl.contains(DrawFl::BOLD) {
        &*uf2::FONT_VENICE14
    } else if fl.contains(DrawFl::HEAD) {
        //&*uf2::FONT_NEWYORK14
        &*uf2::FONT_TIMES15
    } else if fl.contains(DrawFl::EMPH) {
        &*uf2::FONT_ANGELES12
        //&*uf2::FONT_CREAM12
        //&*uf2::FONT_SHAVIAN12
    } else {
        &*uf2::FONT_GENEVA12
    };

    let frect = |canvas: &mut WindowCanvas, x: usize, y: usize, w: usize, h: usize, c: u32| {
        let (r, g, b) = (c >> 16, c >> 8 & 0xFF, c & 0xFF);
        canvas.set_draw_color(Color::RGB(r as _, g as _, b as _));
        canvas.fill_rect(Rect::new(x as _, y as _, w as _, h as _)).unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
    };

    #[allow(unused_variables)]
    let drect = |canvas: &mut WindowCanvas, x: usize, y: usize, w: usize, h: usize| {
        //canvas.set_draw_color(Color::RGB(186, 187, 186));
        canvas.draw_rect(Rect::new(x as _, y as _, w as _, h as _)).unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
    };

    match node {
        N::Root(Root { children, .. }) => {
            for c in children {
                let (nx, ny) = draw_md(p, canvas, c, lx, ex, x, y, fl);
                x = nx;
                y = ny;
            }
            x = sx;
        }
        N::Paragraph(Paragraph { children, .. }) => {
            for c in children {
                let (nx, ny) = draw_md(p, canvas, c, lx, ex, x, y, fl);
                x = nx;
                y = ny;
            }

            x = sx;
            y += 2 * 8;
            y += PAR_PAD;
        }
        N::Heading(Heading {
            children, depth: _, ..
        }) => {
            //y += PAR_PAD;

            // STYLE 2
            //const HEAD_HR_W: usize = 24;
            //rect(canvas, x, y + 6, HEAD_HR_W, 3);
            //x += HEAD_HR_W + 8;

            // STYLE 1
            //const HEAD_BOX_PAD_X: usize = 10;
            //const HEAD_BOX_PAD_Y: usize = 8;
            let (ox, oy) = (x, y);
            // x += HEAD_BOX_PAD_X;
            // y += HEAD_BOX_PAD_Y;

            // STYLE 3
            // for p in 0..10 {
            //     frect(canvas, x, oy, 2, 16);
            //     x += 6 - 6usize.saturating_sub(p / 1).max(1);
            // }
            // x += 6;

            for c in children {
                let (nx, ny) = draw_md(p, canvas, c, lx, ex, x, y, fl | DrawFl::HEAD);
                x = nx;
                y = ny;
            }

            // STYLE 4
            frect(canvas, ox - 4, oy - 4, x - ox + 8, fnt.height + 8, 0x232334);
            frect(canvas, ox - 4, oy + fnt.height + 4, ex - ox, 4, 0x232334);
            (x, y) = (ox, oy);
            canvas.set_draw_color(Color::RGB(220, 220, 200));
            for c in children {
                let (nx, ny) = draw_md(p, canvas, c, lx, ex, x, y, fl | DrawFl::HEAD);
                x = nx;
                y = ny;
            }
            canvas.set_draw_color(Color::RGB(0, 0, 0));

            // STYLE 3
            // x += 6;
            // for p in 0..10 {
            //     frect(canvas, x, oy, 2, 16);
            //     x += 6usize.saturating_sub(p / 2).max(1);
            // }

            // STYLE 2
            //y += PAR_PAD;
            //drect(canvas, ox, oy, (x - ox) + (HEAD_BOX_PAD_X ), (y - oy) + (HEAD_BOX_PAD_Y * 1));

            // STYLE 1
            //rect(canvas, x + 8, y + 6, ex - x, 3);

            x = sx;
            y += fnt.height;
            y += PAR_PAD;
            y += PAR_PAD / 2;
        }
        N::Strong(Strong { children, .. }) => {
            for c in children {
                let (nx, ny) = draw_md(p, canvas, c, lx, ex, x, y, fl | DrawFl::BOLD);
                x = nx;
                y = ny;
            }
        }
        N::Emphasis(Emphasis { children, .. }) => {
            for c in children {
                let (nx, ny) = draw_md(p, canvas, c, lx, ex, x, y, fl | DrawFl::EMPH);
                x = nx;
                y = ny;
            }
        }
        N::List(List {
            children,
            ordered,
            start,
            spread: _,
            ..
        }) => {
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
                let (nx, ny) = draw_md(p, canvas, c, lx + o, ex, x + o, y, fl);
                x = nx - o;
                y = ny;
            }
            x = sx;
            y += PAR_PAD;
        }
        N::ListItem(ListItem { children, .. }) => {
            for c in children {
                let (nx, ny) = draw_md(p, canvas, c, lx, ex, x, y, fl);
                x = nx;
                y = ny - PAR_PAD;
            }
        }
        N::BlockQuote(BlockQuote { children, .. }) => {
            let oldy = sy;
            for c in children {
                let (nx, ny) = draw_md(p, canvas, c, lx + 10, ex, x + 10, y, fl);
                x = nx;
                y = ny;
            }
            x = lx;
            frect(canvas, lx, oldy, 4, y - oldy - PAR_PAD, 0xbabbba);
        }
        N::Text(Text { value, .. }) => {
            let (nx, ny) = draw_text(p, canvas, lx, ex, x, y, fl, &value);
            x = nx;
            y = ny;
        }
        n => println!("Node not implemented: {:?}", n),
    }

    (x, y)
}

fn draw_text(
    p: &Presentation,
    canvas: &mut WindowCanvas,
    lx: usize,
    ex: usize,
    sx: usize,
    sy: usize,
    fl: DrawFl,
    value: &str,
) -> (usize, usize)
{
    if p.config.ttf {
        let mut style = Sdl2FontStyle::NORMAL;
        if fl.contains(DrawFl::BOLD) {
            style |= Sdl2FontStyle::BOLD;
        }
        if fl.contains(DrawFl::HEAD) {
            // style |= FontStyle::UNDERLINE;
        }
        if fl.contains(DrawFl::EMPH) {
            style |= Sdl2FontStyle::ITALIC;
        }
        let mut font = p.fonts[0].borrow_mut();
        font.set_style(style);

        let mut x = sx;
        let mut y = sy;

        // let h = (p.fonts[0].height() as usize) / 2;
        // let surf = p.fonts[0].render(value)
        //     .blended_wrapped(Color::RGB(0, 0, 0), (ex as u32) * 2).unwrap();
        // let text = surf.as_texture(p.tcreator).unwrap();
        // let (tw, th) = (text.query().width as usize, text.query().height as usize);
        // canvas.set_scale(1.0, 1.0).unwrap();
        // canvas.copy(
        //     &text,
        //     None,
        //     Some(Rect::new((x * 2) as i32, (y * 2) as i32, tw as _, th as _))
        // ).unwrap();
        // canvas.set_scale(2.0, 2.0).unwrap();
        // x += tw / 2;
        // if th > h*2 {
        //     y += th / 2;
        //     x = lx;
        // }

        for group in value.split_inclusive(&[' ', '\n']) {
            if x + (font.size_of(group).unwrap().0 as usize) / 2 >= ex {
                y += (font.height() as usize) / 2;
                x = lx;
            }

            for ch in group.as_bytes() {
                if *ch == b'\n' {
                    // y += 8 * X;
                    // x = lx;
                    x += font.size_of(" ").unwrap().0 as usize;
                    continue;
                }

                let surf = font.render_char(*ch as _)
                    .blended(canvas.draw_color())
                    .unwrap();
                let text = surf.as_texture(p.tcreator).unwrap();
                let (tw, th) = (text.query().width as usize, text.query().height as usize);
                canvas.set_scale(1.0, 1.0).unwrap();
                canvas.copy(
                    &text,
                    None,
                    Some(Rect::new((x * 2) as i32, (y * 2) as i32, tw as _, th as _))
                ).unwrap();
                canvas.set_scale(2.0, 2.0).unwrap();
                x += tw / 2;
            }
        }

        (x, y)
    } else {
        let fnt = if fl.contains(DrawFl::BOLD) {
            &*uf2::FONT_VENICE14
        } else if fl.contains(DrawFl::HEAD) {
            &*uf2::FONT_TIMES15
        } else if fl.contains(DrawFl::EMPH) {
            &*uf2::FONT_ANGELES12
        } else {
            &*uf2::FONT_GENEVA12
        };

        uf2::draw(canvas, fnt, lx, ex, sx, sy, &value)
    }
}
