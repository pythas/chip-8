extern crate minifb;

use std::fs;
use std::num::Wrapping;
use rand::Rng;
use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

struct Emulator {
    memory: [u8; 4096],
    video: Vec<u32>,
    stack: [u16; 16],
    v: [u8; 16],
    i: u16,
    dt: u8,
    st: u8,
    pc: u16,
    sp: u8,
    keys: [bool; 16]
}

impl Emulator {
    fn new() -> Emulator {
        let memory = {
            let mut memory = [0; 4096];

            memory[..80].clone_from_slice(&[0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80]);

            memory
        };

        Emulator {
            memory,
            video: vec![0; WIDTH * HEIGHT],
            stack: [0; 16],
            v: [0; 16],
            i: 0,
            dt: 0,
            st: 0,
            pc: 0x200,
            sp: 0,
            keys: [false; 16],
        }
    }

    fn load_rom(&mut self, path: &str) {
        let buffer = fs::read(path).expect("Could not read ROM.");

        self.memory[0x200..0x200 + buffer.len()].clone_from_slice(&buffer[..]);

        // print!("{:?}", &self.memory[..]);
    }

    fn step(&mut self) {
        let instruction = (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16;
        let a = (instruction >> 12) & 0xf;

        println!("{:x}", instruction);

        if self.dt > 0 {
            self.dt = self.dt - 1;
        }

        if self.st > 0 {
            self.st = self.st - 1;
        }

        let mut rng = rand::thread_rng();

        match a {
            0x0 => {
                let nn = instruction & 0xff;

                match nn {
                    0xe0 => {
                        for x in 0..WIDTH*HEIGHT {
                            self.video[x] = 0x00;
                        }
                    }
                    0xee => {
                        self.sp = self.sp - 1;
                        self.pc = self.stack[self.sp as usize];
                        return;
                    }
                    _ => {
                        println!("0x{:x}: Not implemented", nn);
                        std::process::exit(1);
                    }
                }
            }
            0x1 => {
                let nnn = instruction & 0xfff;

                self.pc = nnn;
                return;
            }
            0x2 => {
                let nnn = instruction & 0xfff;

                self.stack[self.sp as usize] = self.pc + 2;
                self.sp = self.sp + 1;
                self.pc = nnn;
                return;
            }
            0x3 => {
                let x = (instruction >> 8) & 0xf;
                let kk = (instruction & 0xff) as u8;

                if self.v[x as usize] == kk {
                    self.pc = self.pc + 2;
                }
            }
            0x4 => {
                let x = (instruction >> 8) & 0xf;
                let kk = (instruction & 0xff) as u8;

                if self.v[x as usize] != kk {
                    self.pc = self.pc + 4;
                    return;
                }
            }
            0x5 => {
                let x = (instruction >> 8) & 0xf;
                let y = (instruction >> 4) & 0xf;

                if self.v[x as usize] == self.v[y as usize] {
                    self.pc = self.pc + 4;
                    return;
                }
            }
            0x6 => {
                let x = (instruction >> 8) & 0xf;
                let kk = (instruction & 0xff) as u8;

                self.v[x as usize] = kk;
            }
            0x7 => {
                let x = (instruction >> 8) & 0xf;
                let kk = (instruction & 0xff) as u8;

                self.v[x as usize] = (Wrapping(self.v[x as usize]) + Wrapping(kk)).0;
            }
            0x8 => {
                let x = (instruction >> 8) & 0xf;
                let y = (instruction >> 4) & 0xf;
                let b = instruction & 0xf;

                match b {
                    0x0 => {
                        self.v[x as usize] = self.v[y as usize];
                    }
                    0x1 => {
                        self.v[x as usize] = self.v[x as usize] | self.v[y as usize];
                    }
                    0x2 => {
                        self.v[x as usize] = self.v[x as usize] & self.v[y as usize];
                    }
                    0x3 => {
                        self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize];
                    }
                    0x4 => {
                        let val = (Wrapping(self.v[x as usize]) + Wrapping(self.v[y as usize])).0;
                        self.v[x as usize] = val & 0xff;

                        if self.v[x as usize] as u16 + self.v[y as usize] as u16 > 255 {
                            self.v[0xf] = 1;
                        } else {
                            self.v[0xf] = 0;
                        }
                    }
                    0x5 => {
                        if self.v[x as usize] > self.v[y as usize] {
                            self.v[0xf] = 1;
                        } else {
                            self.v[0xf] = 0;
                        }

                        self.v[x as usize] = (Wrapping(self.v[x as usize]) - Wrapping(self.v[y as usize])).0;
                    }
                    0x6 => {
                        if self.v[x as usize] & 1 == 1 {
                            self.v[0xf] = 1;
                        } else {
                            self.v[0xf] = 0;
                        }

                        self.v[x as usize] = (Wrapping(self.v[x as usize]) / Wrapping(2)).0;
                    }
                    0x7 => {
                        if self.v[y as usize] > self.v[x as usize] {
                            self.v[0xf] = 1;
                        } else {
                            self.v[0xf] = 0;
                        }

                        self.v[x as usize] = (Wrapping(self.v[y as usize]) - Wrapping(self.v[x as usize])).0;
                    }
                    0xe => {
                        if (self.v[x as usize] & 0xff) >> 7 == 1 {
                            self.v[0xf] = 1;
                        } else {
                            self.v[0xf] = 0;
                        }

                        self.v[x as usize] = (Wrapping(self.v[x as usize]) * Wrapping(2)).0;
                    }
                    _ => {
                        println!("8xy{:x}: Not implemented", b);
                        std::process::exit(1);
                    }
                }
            }
            0x9 => {
                let x = (instruction >> 8) & 0xf;
                let y = (instruction >> 4) & 0xf;

                if self.v[x as usize] != self.v[y as usize] {
                    self.pc = self.pc + 2;
                }
            }
            0xa => {
                let nnn = instruction & 0xfff;

                self.i = nnn;
            }
            0xc => {
                let x = (instruction >> 8) & 0xf;
                let kk = (instruction & 0xff) as u8;

                self.v[x as usize] = rng.gen::<u8>() & kk;
            }
            0xd => {
                let mut x = ((instruction >> 8) & 0xf) as u8;
                let mut y = ((instruction >> 4) & 0xf) as u8;
                let n = instruction & 0xf;

                x = self.v[x as usize];
                y = self.v[y as usize];

                let bytes = &self.memory[self.i as usize..(self.i + n) as usize];

                // fn draw bytes
                for byte in bytes {
                    let bits = [
                        byte & (1 << 7) != 0,
                        byte & (1 << 6) != 0,
                        byte & (1 << 5) != 0,
                        byte & (1 << 4) != 0,
                        byte & (1 << 3) != 0,
                        byte & (1 << 2) != 0,
                        byte & (1 << 1) != 0,
                        byte & (1 << 0) != 0,
                    ];

                    for (i, bit) in bits.iter().enumerate() {
                        let color: u32 = if *bit {
                            0xffffffff
                        } else {
                            0x00000000
                        };

                        let x2 = (Wrapping(x) + Wrapping(i as u8)).0 % 64;
                        let y2 = y % 32;
                        let addr = (y2 as u16 * WIDTH as u16 + x2 as u16) as usize;
                        let orig = self.video[addr];
                        self.video[addr] = self.video[addr] ^ color;

                        if orig == 0xffffffff && self.video[addr] == 0x00000000 {
                            self.v[0xf] = 1;
                        } else {
                            self.v[0xf] = 0;
                        }
                    }

                    y = y + 1;
                }
            }
            0xe => {
                let x = (instruction >> 8) & 0xf;
                let b = instruction & 0xff;

                match b {
                    0xa1 => {
                        if self.keys[self.v[x as usize] as usize] == false {
                            self.pc = self.pc + 4;
                            return;
                        }
                    }
                    0x9e => {
                        if self.keys[self.v[x as usize] as usize] == true {
                            self.pc = self.pc + 4;
                            return;
                        }
                    }
                    _ => {
                        println!("ex{:x}: Not implemented", b);
                        std::process::exit(1);
                    }
                }
            }
            0xf => {
                let x = (instruction >> 8) & 0xf;
                let b = instruction & 0xff;

                match b {
                    0x7 => {
                        self.v[x as usize] = self.dt as u8;
                    }
                    0x15 => {
                        self.dt = self.v[x as usize] as u8;
                    }
                    0x18 => {
                        self.st = self.v[x as usize] as u8;
                    }
                    0x29 => {
                        self.i = (self.v[x as usize] * 5) as u16;
                    }
                    0x33 => {
                        self.memory[self.i as usize] = (self.v[x as usize] / 100) as u8;
                        self.memory[(self.i + 1) as usize] = ((self.v[x as usize] / 10) % 10) as u8;
                        self.memory[(self.i + 2) as usize] = ((self.v[x as usize] % 100) % 10) as u8;
                    }
                    0x55 => {
                        for i in 0..x + 1 {
                            self.memory[(self.i + i as u16) as usize] = self.v[i as usize];
                        }
                    }
                    0x65 => {
                        for i in 0..x + 1 {
                            self.v[i as usize] = self.memory[(self.i + i as u16) as usize];
                        }
                    }
                    0x1e => {
                        self.i = self.i + self.v[x as usize] as u16;
                    }
                    _ => {
                        println!("fx{:x}: Not implemented", b);
                        std::process::exit(1);
                    }
                }
            }
            _ => {
                println!("{:x}: Not implemented", a);
                std::process::exit(1);
            }
        }

        self.pc = self.pc + 2;
    }
}

fn main() {
    let mut window = Window::new(
        "CHIP-8 Emulator",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale: Scale::X8,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to Open Window");

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    window.set_background_color(0, 0, 20);

    let mut emulator = Emulator::new();

    emulator.load_rom("roms\\BC_test.ch8");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut pressed_keys = [false; 16];

        window.get_keys_pressed(KeyRepeat::Yes).map(|keys| {
            for t in keys {
                match t {
                    Key::Key1 => pressed_keys[0x1] = true,
                    Key::Key2 => pressed_keys[0x2] = true,
                    Key::Key3 => pressed_keys[0x3] = true,
                    Key::Key4 => pressed_keys[0xc] = true,
                    Key::Q => pressed_keys[0x4] = true,
                    Key::W => pressed_keys[0x5] = true,
                    Key::E => pressed_keys[0x6] = true,
                    Key::R => pressed_keys[0xd] = true,
                    Key::A => pressed_keys[0x7] = true,
                    Key::S => pressed_keys[0x8] = true,
                    Key::D => pressed_keys[0x9] = true,
                    Key::F => pressed_keys[0xe] = true,
                    Key::Z => pressed_keys[0xa] = true,
                    Key::X => pressed_keys[0x0] = true,
                    Key::C => pressed_keys[0xb] = true,
                    Key::V => pressed_keys[0xf] = true,
                    _ => (),
                }
            }
        });

        emulator.keys = pressed_keys;

        emulator.step();

        window.update_with_buffer(&emulator.video, WIDTH, HEIGHT).unwrap();
    }
}