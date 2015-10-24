use std::f64;
use std::thread;
use std::ops::{Add, Sub};

extern crate sdl;

use sdl::video::{SurfaceFlag, VideoFlag, Color};
use sdl::event::{Event, Key};

#[derive(Clone, Copy)]
struct Vect {
    x: f64,
    y: f64,
}

impl Add for Vect {
    type Output = Vect;
    fn add(self, vec: Vect) -> Vect {
        Vect {x: self.x + vec.x, y: self.y + vec.y}
    }
}

impl Sub for Vect {
    type Output = Vect;
    fn sub(self, vec: Vect) -> Vect {
        Vect {x: self.x - vec.x, y: self.y - vec.y}
    }
}

impl Vect {
    fn size(&self) -> f64 {
        (self.x*self.x + self.y* self.y).sqrt()
    }
    fn scale(&mut self, r: f64) {
        self.x *= r;
        self.y *= r;
    }
}

#[derive(Clone, Copy)]
struct Dot {
    pos: Vect,
    vel: Vect,
    acc: Vect,
    fixed: bool,
}

impl Dot {
    fn new(x: f64, y: f64, f: bool) -> Dot {
        Dot {
            pos: Vect {x: x, y: y},
            vel: Vect {x: 0., y: 0.},
            acc: Vect {x: 0., y: 0.},
            fixed: f,
        }
    }
    fn move_it(&mut self) {
        self.pos = self.pos + self.vel;
    }
    fn accelerate(&mut self) {
        self.vel = self.vel + self.acc;
    }
    fn get_force(self, dot: Dot) -> Vect {
        let x = dot.pos.x - self.pos.x;
        let y = dot.pos.y - self.pos.y;
        Vect {x: x, y: y}
    }
    fn set_force(&mut self, vect: Vect) {
        self.acc = vect;
    }
}

struct Chord {
    k: f64,
    chord: Vec<Dot>,
}

impl Chord {
    fn new(n: u16, k: f64) -> Chord {
        let mut ds = Vec::with_capacity(n as usize);
        ds.push(Dot::new(0., 0., true));
        for i in 1..n/2 {
            ds.push(Dot::new(i as f64,
                i as f64*0.375,
                false));
        }
        for i in n/2..n {
            ds.push(Dot::new(i as f64,
                (n as f64-i as f64)*0.375,
                false));
        }
        ds.push(Dot::new(n as f64, 0., true));
        Chord {k: k, chord: ds}
    }
    fn new_sine(n: u16, k: f64) -> Chord {
        let mut ds = Vec::with_capacity(n as usize);
        ds.push(Dot::new(0., 0., true));
        for i in 1..n {
            ds.push(Dot::new(i as f64,
                (f64::consts::PI*i as f64/n as f64).sin()*5.,
                false));
        }
        ds.push(Dot::new(n as f64, 0., true));
        Chord {k: k, chord: ds}
    }
    fn tick(&mut self) {
        let dots = &mut self.chord;
        for i in 0..dots.len() {
            let mut force = Vect {x: 0., y: 0.};
            if i>0  {
                force = force + dots[i].get_force(dots[i-1]);
            }
            if i<dots.len()-1  {
                force = force + dots[i].get_force(dots[i+1]);
            }
            force.scale(self.k);
            dots[i].set_force(force);

        }
        for i in 0..dots.len() {
            if dots[i].fixed {continue;}
            dots[i].accelerate();
            dots[i].move_it();
        }
    }
}

struct Screen {
    width: isize,
    height: isize,
    surface: sdl::video::Surface,
    should_end: bool,
}

impl Screen {
    fn new(w: isize, h: isize) -> Screen {
        sdl::init(&[sdl::InitFlag::Video]);
        sdl::wm::set_caption("String", "String");

        let s = match sdl::video::set_video_mode(w, h, 32,
                                                 &[SurfaceFlag::HWSurface],
                                                 &[VideoFlag::DoubleBuf]) {
            Ok(s) => s,
            Err(err) => panic!("failed to set video mode: {}", err)
        };
        Screen {width: w, height: h, surface: s, should_end: false}
    }
    fn tick(&mut self) {
        match sdl::event::poll_event() {
            Event::Quit => {
                self.should_end = true;
            },
            Event::Key(k, down, _, _) => {
                if down {
                    if k == Key::Escape {
                        self.should_end = true;
                    }
                }
            },
            _ => {}
        }
    }
    fn draw_square(&self, x: u16, y: u16, w: u16, (r,g,b): (u8, u8, u8)) {
        self.surface.fill_rect(
            Some(sdl::Rect {x: x as i16, y: y as i16, w: w, h: w}),
            Color::RGB(r, g, b)
        );
    }
    fn draw(&mut self, chord: &mut Chord) {
        self.surface.clear();
        for &c in &chord.chord {
            let x = c.pos.x*(self.width-100) as f64/chord.chord.len() as f64 + 50.;
            let y = c.pos.y*(self.height-100) as f64/chord.chord.len() as f64 + self.height as f64/2.;
            self.draw_square(
                x.round() as u16,
                y.round() as u16,
                4, (255,255,255)
            );
        }
        self.surface.flip();
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        sdl::quit();
    }
}

fn main() {
    let mut screen = Screen::new(800,600);
    let chord = &mut Chord::new(80, 1./(1<<12) as f64);
    screen.draw(chord);
    loop {
        chord.tick();
        screen.draw(chord);
        screen.tick();
        if screen.should_end {break;}
        thread::sleep_ms(1);
    }
}
