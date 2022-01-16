use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, TextureCreator, TextureQuery};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

const BORDER: u32 = 40;
const NUM_SQUAREWAVES: usize = 12;
const NUM_MAP: [(Keycode, usize); 9] = [
    (Keycode::Num1, 0),
    (Keycode::Num2, 1),
    (Keycode::Num3, 2),
    (Keycode::Num4, 3),
    (Keycode::Num5, 4),
    (Keycode::Num6, 5),
    (Keycode::Num7, 6),
    (Keycode::Num8, 7),
    (Keycode::Num9, 8),
];
const VOLUME: f32 = 0.01;

struct SquareWave {
    phase: [f32; NUM_SQUAREWAVES],
    phase_inc: [f32; NUM_SQUAREWAVES],
    volume: [f32; NUM_SQUAREWAVES],
}

impl SquareWave {
    fn new() -> SquareWave {
        SquareWave {
            phase: [0.0; NUM_SQUAREWAVES],
            phase_inc: [0.0; NUM_SQUAREWAVES],
            volume: [0.0; NUM_SQUAREWAVES],
        }
    }

    fn set_freq(&mut self, freq: f32) {
        self.phase_inc[0] = 349.24 / freq;
        self.phase_inc[1] = 392.00 / freq;
        self.phase_inc[2] = 440.00 / freq;
        self.phase_inc[3] = 493.92 / freq;
        self.phase_inc[4] = 523.28 / freq;
        self.phase_inc[5] = 587.36 / freq;
        self.phase_inc[6] = 659.28 / freq;
        self.phase_inc[7] = 698.48 / freq;
        self.phase_inc[8] = 784.00 / freq;
    }
}

impl AudioCallback for &mut SquareWave {
    type Channel = f32;

    fn callback(&mut self, channels: &mut [f32]) {
        for channel in channels.iter_mut() {
            *channel = 0.0;
            for sw in 0..NUM_SQUAREWAVES {
                *channel += if self.phase[sw] <= 0.5 {
                    self.volume[sw]
                } else {
                    -self.volume[sw]
                };
                self.phase[sw] = (self.phase[sw] + self.phase_inc[sw]) % 1.0;
            }
        }
    }
}

fn get_text<'a>(
    f: &Font,
    s: &str,
    c: Color,
    texture_creator: &'a TextureCreator<WindowContext>,
) -> Result<(Texture<'a>, u32, u32), String> {
    let surface = f.render(s).blended(c).map_err(|e| e.to_string())?;

    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    let TextureQuery { width, height, .. } = texture.query();

    Ok((texture, width, height))
}

fn main() -> Result<(), String> {
    println!("linked sdl2_ttf: {}", sdl2::ttf::get_linked_version());
    let sdl_context = sdl2::init()?;
    let audio_subsystem = sdl_context.audio()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let window = video_subsystem
        .window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| format!("Could not initialize video subsystem: {}", e.to_string()))?;

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| format!("could not make a canvas: {}", e.to_string()))?;

    let texture_creator = canvas.texture_creator();
    let font = ttf_context.load_font(
        "/home/monkey/.fonts/System San Francisco Display Regular.ttf",
        16,
    )?;

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let mut sw = SquareWave::new();

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        sw.set_freq(spec.freq as f32);
        &mut sw
    })?;

    device.resume();

    let mut fps_vec = VecDeque::new();
    let mut prev_stamp = Instant::now();
    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;
    'running: loop {
        let curr_stamp = Instant::now();
        fps_vec.push_front(1.0 / ((curr_stamp - prev_stamp).as_secs_f64()));
        if fps_vec.len() > 10 {
            fps_vec.pop_back();
        }
        let fps: f64 = fps_vec.iter().sum::<f64>() / fps_vec.len() as f64;

        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(0, 128, 0));

        let (width, height) = canvas.output_size()?;
        canvas.fill_rect(rect!(
            BORDER,
            BORDER,
            width - (BORDER * 2),
            height - (BORDER * 2)
        ))?;

        let p1 = Point::new(100, 200);
        let p2 = Point::new(300, 400);
        canvas.set_draw_color(Color::RGB(128, 0, 0));
        canvas.draw_line(p1, p2)?;
        let (text_texture, text_width, text_height) = get_text(
            &font,
            format!("FPS: {fps:.1}").as_str(),
            Color::RGB(255, 0, 0),
            &texture_creator,
        )?;
        let text_rect = rect!(50, 50, text_width, text_height);
        canvas.copy(&text_texture, None, Some(text_rect))?;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode:
                        Some(
                            keycode @ (Keycode::Num1
                            | Keycode::Num2
                            | Keycode::Num3
                            | Keycode::Num4
                            | Keycode::Num5
                            | Keycode::Num6
                            | Keycode::Num7
                            | Keycode::Num8
                            | Keycode::Num9),
                        ),
                    repeat: false,
                    ..
                } => {
                    for (key, num) in NUM_MAP {
                        if keycode == key {
                            sw.volume[num] = VOLUME;
                        }
                    }
                }
                Event::KeyUp {
                    keycode:
                        Some(
                            keycode @ (Keycode::Num1
                            | Keycode::Num2
                            | Keycode::Num3
                            | Keycode::Num4
                            | Keycode::Num5
                            | Keycode::Num6
                            | Keycode::Num7
                            | Keycode::Num8
                            | Keycode::Num9),
                        ),
                    repeat: false,
                    ..
                } => {
                    for (key, num) in NUM_MAP {
                        if keycode == key {
                            sw.volume[num] = 0.00;
                        }
                    }
                }
                _ => {}
            }
        }

        canvas.present();
        let goal_fps = 120;
        prev_stamp = curr_stamp;
        let fin_stamp = Instant::now();
        if fin_stamp < (curr_stamp + Duration::new(0, 1_000_000_000u32 / goal_fps)) {
            std::thread::sleep(
                curr_stamp + Duration::new(0, 1_000_000_000u32 / goal_fps) - fin_stamp,
            );
        }
    }

    Ok(())
}
