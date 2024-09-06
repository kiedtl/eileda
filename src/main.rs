use sdl2;

use sdl2::event::Event;
use sdl2::image::InitFlag;
use sdl2::keyboard::Keycode;
use std::time::Duration;

mod md;
mod slide;
mod uf2;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} file.eimd", args[0]);
        return;
    }

    let sdl_context = sdl2::init().unwrap();
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG).unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("eileda", 960, 640) //1440, 810)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    canvas.set_scale(2.0, 2.0).unwrap();

    let lexed = md::lex(&args[1]);
    let mut stuff = md::parse(&texture_creator, &lexed);

    if stuff.slides.len() == 0 {
        eprintln!("Presentation is empty");
        return;
    }

    let mut cur = 0;
    stuff.draw(cur, &mut canvas);

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    if cur > 0 {
                        cur -= 1;
                    }
                    stuff.draw(cur, &mut canvas);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    if cur < stuff.slides.len() - 1 {
                        cur += 1;
                    }
                    stuff.draw(cur, &mut canvas);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    let lexed = md::lex(&args[1]);
                    stuff = md::parse(&texture_creator, &lexed);

                    if stuff.slides.len() == 0 {
                        eprintln!("Presentation is empty");
                        return;
                    }
                    cur = cur.min(stuff.slides.len() - 1);
                    stuff.draw(cur, &mut canvas);
                }
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
