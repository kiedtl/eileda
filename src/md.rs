use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

use crate::slide::*;
use sdl2::image::LoadTexture;
use sdl2::render::TextureCreator;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;
use sdl2::rwops::RWops;

#[derive(Clone, Debug)]
pub enum Item {
    Pad(usize),
    Margin(usize, PathBuf),
    BeginSlide(Option<String>),
    Md(markdown::mdast::Node),
    Img(PathBuf),
    BeginGrid(usize),
    NextColumn,
    EndGrid,
}

pub fn lex(file: &str) -> Vec<Item> {
    let data = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Nuh uh");
            return Vec::new();
        }
    };

    let mut items = Vec::new();
    let mut mdbuf = String::new();

    for line in data.lines() {
        if line.starts_with(".") && !line.starts_with("..") {
            if mdbuf != "" {
                items.push(Item::Md(
                    markdown::to_mdast(&mdbuf, &markdown::ParseOptions {
                        constructs: markdown::Constructs {
                            hard_break_trailing: false,
                            ..Default::default()
                        },
                        ..Default::default()
                    }).unwrap(),
                ));
                mdbuf.clear();
            }

            let cmd = line.split(" ").collect::<Vec<_>>();
            match cmd[0] {
                ".SLD" => {
                    let t = if cmd.len() == 1 { None } else {
                        Some(line[cmd[0].len() + 1..].to_string())
                    };
                    items.push(Item::BeginSlide(t));
                }
                ".MAR" if cmd.len() == 3 => {
                    items.push(Item::Margin(
                        cmd[1].parse().unwrap_or(0),
                        PathBuf::from(cmd[2]),
                    ));
                }
                ".PAD" if cmd.len() == 2 => {
                    items.push(Item::Pad(cmd[1].parse().unwrap_or(0)));
                }
                ".IMG" if cmd.len() == 2 => {
                    items.push(Item::Img(PathBuf::from(cmd[1])));
                }
                ".GRD" => {
                    if cmd.len() <= 2 {
                        if cmd.len() == 1 {
                            items.push(Item::BeginGrid(50));
                        } else if let Ok(rat) = cmd[1].parse() {
                            items.push(Item::BeginGrid(rat));
                        } else if cmd[1] == "end" {
                            items.push(Item::EndGrid);
                        } else {
                            eprintln!("Error: Bad grid directive");
                        }
                    }
                }
                ".COL" => {
                    if cmd.len() == 1 {
                        items.push(Item::NextColumn);
                    }
                }
                c => eprintln!("Error: Unknown or invalid directive: {}", c),
            }
        } else {
            mdbuf.push_str(&(line.to_owned() + "\n"));
        }
    }

    items.push(Item::Md(
        markdown::to_mdast(&mdbuf, &Default::default()).unwrap(),
    ));
    mdbuf.clear();

    items
}

pub fn parse<'a>(
    tcreator: &'a TextureCreator<WindowContext>,
    fcreator: &'a Sdl2TtfContext,
    items: &Vec<Item>,
) -> Presentation<'a> {
    macro_rules! embed_ttf {
        ($path:literal, $sz:literal) => {
            {
                let data = include_bytes!($path);
                let rwops = Box::new(RWops::from_bytes(data).unwrap());
                RefCell::new(
                    fcreator.load_font_from_rwops(*rwops, $sz).unwrap()
                )
            }
        }
    }

    let mut p = Presentation {
        tcreator,
        config: GlobalConfig {
            padding: 16,
            margin: None,
            ttf: false,
        },
        slides: Vec::new(),

        f_bold_68: embed_ttf!("../assets/ttf/Inter-Bold.otf", 68),
        f_norm_24: embed_ttf!("../assets/ttf/Inter-Regular.otf", 24),
        f_bold_24: embed_ttf!("../assets/ttf/Inter-Bold.otf", 24),
        f_emph_24: embed_ttf!("../assets/ttf/Inter-Italic.otf", 24),
        f_both_24: embed_ttf!("../assets/ttf/Inter-BoldItalic.otf", 24),
    };

    let mut last_title = None;

    let _push = |p: &mut Presentation<'a>, content: Content<'a>| {
        let slide_last_idx = p.slides.len() - 1;

        if p.slides[slide_last_idx].content.len() == 0 {
            p.slides[slide_last_idx].content.push(content);
            return;
        }

        let content_last_idx = p.slides[slide_last_idx].content.len() - 1;
        match p.slides[slide_last_idx].content[content_last_idx] {
            Content::Grid(Grid {
                ref mut first,
                _parser_col_adv,
                ..
            }) if _parser_col_adv == false => {
                first.push(content);
            }
            Content::Grid(Grid {
                ref mut second,
                _parser_col_adv,
                ..
            }) if _parser_col_adv == true => {
                second.push(content);
            }
            _ => p.slides[slide_last_idx].content.push(content),
        }
    };

    for item in items.into_iter() {
        if p.slides.len() == 0 {
            match item {
                Item::Pad(pad) => p.config.padding = *pad,
                Item::Margin(middle_size, path) => p.config.margin = Some(Margin {
                    image: match tcreator.load_texture(path) {
                        Ok(t) => t,
                        Err(s) => {
                            eprintln!("Couldn't load image: {}", s);
                            continue;
                        }
                    },
                    middle: *middle_size,
                }),
                Item::BeginSlide(t) => {
                    let newt = t.clone().or(last_title.clone());
                    p.slides.push(Slide {
                        title: newt.clone(),
                        content: Vec::new(),
                    });
                    last_title = newt.clone();
                },
                _ => eprintln!("Unexpected headers: {:?}", item),
            }
        } else {
            let slide_last_idx = p.slides.len() - 1;

            match item {
                Item::BeginSlide(t) => {
                    let newt = t.clone().or(last_title.clone());
                    p.slides.push(Slide {
                        title: newt.clone(),
                        content: Vec::new(),
                    });
                    last_title = newt.clone();
                },
                Item::BeginGrid(r) => _push(
                    &mut p,
                    Content::Grid(Grid {
                        ratio: *r,
                        first: Vec::new(),
                        second: Vec::new(),
                        _parser_col_adv: false,
                    }),
                ),
                Item::NextColumn => {
                    let content_last_idx = p.slides[slide_last_idx].content.len() - 1;
                    match p.slides[slide_last_idx].content[content_last_idx] {
                        Content::Grid(Grid {
                            ref mut _parser_col_adv,
                            ..
                        }) => {
                            if *_parser_col_adv == true {
                                eprintln!("Error: Spurious .COL directives");
                            }
                            *_parser_col_adv = true;
                        }
                        _ => eprintln!("Error: Spurious .COL directives (no parent)"),
                    }
                }
                Item::EndGrid => p.slides[slide_last_idx].content.push(Content::Dummy),
                Item::Md(md) => _push(&mut p, Content::Md(md.clone())),
                Item::Img(path) => _push(
                    &mut p,
                    Content::Img(match tcreator.load_texture(path) {
                        Ok(t) => t,
                        Err(s) => {
                            eprintln!("Couldn't load image: {}", s);
                            continue;
                        }
                    }),
                ),
                _ => eprintln!("Unexpected content: {:?}", item),
            }
        }
    }

    p
}
