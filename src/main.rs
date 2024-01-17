use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime};
mod model;
use crate::model::*;

pub const WINDOW_TITLE: &str = "rust-samegame";
pub const CELL_SIZE: i32 = 40;
pub const SCREEN_WIDTH: i32 = BOARD_W * CELL_SIZE;
pub const SCREEN_HEIGHT: i32 = BOARD_H * CELL_SIZE + 82;

struct Image<'a> {
    texture: Texture<'a>,
    #[allow(dead_code)]
    w: u32,
    h: u32,
}

impl<'a> Image<'a> {
    fn new(texture: Texture<'a>) -> Self {
        let q = texture.query();
        let image = Image {
            texture,
            w: q.width,
            h: q.height,
        };
        image
    }
}

struct Resources<'a> {
    images: HashMap<String, Image<'a>>,
    fonts: HashMap<String, sdl2::ttf::Font<'a, 'a>>,
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window(WINDOW_TITLE, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Blend);

    let texture_creator = canvas.texture_creator();
    let mut resources = load_resources(&texture_creator, &mut canvas, &ttf_context);

    let mut event_pump = sdl_context.event_pump()?;

    let mut game = Game::new();

    'running: loop {
        let started = SystemTime::now();

        let mut command = Command::None;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    match code {
                        Keycode::Escape => {
                            break 'running;
                        }
                        Keycode::Space => {
                            if game.is_over || game.is_clear {
                                game = Game::new();
                            }
                        }
                        _ => {}
                    };
                }
                Event::MouseMotion { x, y, .. } => {
                    let cell_x = (x as usize) / CELL_SIZE as usize;
                    let cell_y = (y as usize) / CELL_SIZE as usize;
                    if game.is_valid_cell(cell_x, cell_y) {
                        command = Command::Hover(cell_x, cell_y);
                    }
                }
                Event::MouseButtonDown { x, y, .. } => {
                    let cell_x = (x as usize) / CELL_SIZE as usize;
                    let cell_y = (y as usize) / CELL_SIZE as usize;
                    println!("Click {} {} {} {}", x, y, cell_x, cell_y);
                    if game.is_valid_cell(cell_x, cell_y) {
                        command = Command::Click(cell_x, cell_y);
                    }
                }
                _ => {}
            }
        }

        game.update(command);

        render(&mut canvas, &game, &mut resources)?;

        let finished = SystemTime::now();
        let elapsed = finished.duration_since(started).unwrap();
        let frame_duration = Duration::new(0, 1_000_000_000u32 / model::FPS as u32);
        if elapsed < frame_duration {
            ::std::thread::sleep(frame_duration - elapsed)
        }
    }

    Ok(())
}

fn load_resources<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    #[allow(unused_variables)] canvas: &mut Canvas<Window>,
    ttf_context: &'a Sdl2TtfContext,
) -> Resources<'a> {
    let mut resources = Resources {
        images: HashMap::new(),
        fonts: HashMap::new(),
    };

    let entries = fs::read_dir("resources/image").unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".bmp") {
            let temp_surface = sdl2::surface::Surface::load_bmp(&path).unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&temp_surface)
                .expect(&format!("cannot load image: {}", path_str));

            let basename = path.file_name().unwrap().to_str().unwrap();
            let image = Image::new(texture);
            resources.images.insert(basename.to_string(), image);
        }
    }

    load_font(
        &mut resources,
        &ttf_context,
        "./resources/font/boxfont2.ttf",
        32,
        "boxfont",
    );

    resources
}

fn load_font<'a>(
    resources: &mut Resources<'a>,
    ttf_context: &'a Sdl2TtfContext,
    path_str: &str,
    point_size: u16,
    key: &str,
) {
    let font = ttf_context
        .load_font(path_str, point_size)
        .expect(&format!("cannot load font: {}", path_str));
    resources.fonts.insert(key.to_string(), font);
}

fn render(
    canvas: &mut Canvas<Window>,
    game: &Game,
    resources: &mut Resources,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(128, 128, 128));
    canvas.fill_rect(Rect::new(0, SCREEN_HEIGHT - 82, SCREEN_WIDTH as u32, 82))?;

    let font = resources.fonts.get_mut("boxfont").unwrap();

    // render board
    for y in 0..BOARD_H as usize {
        for x in 0..BOARD_W as usize {
            if game.board[y][x].exist {
                let image = resources
                    .images
                    .get(&format!("color{}.bmp", game.board[y][x].color))
                    .unwrap();
                canvas
                    .copy(
                        &image.texture,
                        Rect::new(0, 0, image.w, image.h),
                        Rect::new(x as i32 * CELL_SIZE, y as i32 * CELL_SIZE, image.w, image.h),
                    )
                    .unwrap();
            }
        }
    }

    render_font(
        canvas,
        font,
        format!("POINTING: {:5}", game.hover_score).to_string(),
        290,
        405,
        Color::RGBA(255, 255, 255, 255),
        false,
    );

    render_font(
        canvas,
        font,
        format!("SCORE: {:8}", game.score).to_string(),
        290,
        438,
        Color::RGBA(255, 255, 255, 255),
        false,
    );

    if game.is_over {
        render_font(
            canvas,
            font,
            "GAME OVER".to_string(),
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT / 2,
            Color::RGBA(255, 255, 255, 255),
            true,
        );
    }

    if game.is_clear {
        render_font(
            canvas,
            font,
            "CLEAR!!".to_string(),
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT / 2,
            Color::RGBA(255, 255, 0, 255),
            true,
        );
    }

    canvas.present();

    Ok(())
}

fn render_font(
    canvas: &mut Canvas<Window>,
    font: &sdl2::ttf::Font,
    text: String,
    x: i32,
    y: i32,
    color: Color,
    center: bool,
) {
    let texture_creator = canvas.texture_creator();

    let surface = font.render(&text).blended(color).unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    let x: i32 = if center {
        x - texture.query().width as i32 / 2
    } else {
        x
    };
    canvas
        .copy(
            &texture,
            None,
            Rect::new(x, y, texture.query().width, texture.query().height),
        )
        .unwrap();
}
