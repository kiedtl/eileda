use std::fs;
use std::path::PathBuf;

use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use sdl2::image::LoadTexture;
use crate::slide::*;

#[derive(Clone, Debug)]
pub enum Item {
    BeginSlide,
    Pad(usize),
    Md(markdown::mdast::Node),
    Img(PathBuf),
}

pub fn lex(file: &str) -> Vec<Item> {
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
            if mdbuf != "" {
                items.push(Item::Md(markdown::to_mdast(&mdbuf, &Default::default()).unwrap()));
                mdbuf.clear();
            }

            let cmd = line.split(" ").collect::<Vec<_>>();
            match cmd[0] {
                ".SLD" if cmd.len() == 1 => {
                    items.push(Item::BeginSlide);
                },
                ".PAD" if cmd.len() == 2 => {
                    items.push(Item::Pad(cmd[1].parse().unwrap_or(0)));
                },
                ".IMG" if cmd.len() == 2 => {
                    items.push(Item::Img(PathBuf::from(cmd[1])));
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

pub fn parse<'a>(tcreator: &'a TextureCreator<WindowContext>, items: &Vec<Item>) -> Presentation<'a> {
    let mut p = Presentation {
        config: GlobalConfig {
            padding: 0,
        },
        slides: Vec::new(),
    };

    for item in items.into_iter() {
        if p.slides.len() == 0 {
            match item {
                Item::Pad(pad) => p.config.padding = *pad,
                Item::BeginSlide => p.slides.push(Slide { content: Vec::new() }),
                _ => eprintln!("Unexpected headers: {:?}", item),
            }
        } else {
            let last = p.slides.len() - 1;

            match item {
                Item::BeginSlide => p.slides.push(Slide { content: Vec::new() }),
                Item::Md(md) => p.slides[last].content.push(Content::Md(md.clone())),
                Item::Img(path) => p.slides[last].content.push(Content::Img(
                    match tcreator.load_texture(path) {
                        Ok(t) => t,
                        Err(s) => {
                            eprintln!("Couldn't load image: {}", s);
                            continue;
                        },
                    }
                )),
                _ => eprintln!("Unexpected content: {:?}", item),
            }
        }
    }

    p
}
