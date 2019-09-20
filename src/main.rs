use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

const MEMORY_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const PC_START: u16 = 0x200;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
const PIXEL_SIZE: usize = 10;

const FONT_SIZE: usize = 16 * 5;
const FONT: [u8; FONT_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80, //F
];

struct CPU {
    memory: [u8; MEMORY_SIZE],
    display: [u8; DISPLAY_SIZE],
    v: [u8; REGISTER_COUNT],
    i: u16,
    dt: u8,
    st: u8,
    pc: u16,
    sp: u8,
    stack: [u16; STACK_SIZE],
    should_draw: bool,
    keys: u16,
}

fn main() {
    let mut rng = rand::thread_rng();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "chip8",
            (DISPLAY_WIDTH * PIXEL_SIZE) as u32,
            (DISPLAY_HEIGHT * PIXEL_SIZE) as u32,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut timer = sdl_context.timer().unwrap();

    let mut cpu = CPU {
        memory: [0; MEMORY_SIZE],
        display: [0; DISPLAY_SIZE],
        v: [0; REGISTER_COUNT],
        i: 0,
        dt: 0,
        st: 0,
        pc: PC_START,
        sp: 0,
        stack: [0; STACK_SIZE],
        should_draw: true,
        keys: 0,
    };

    cpu.memory[..FONT_SIZE].clone_from_slice(&FONT);

    let mut args = env::args();
    if args.len() != 2 {
        panic!("2 args");
    }

    let filename = args.nth(1).unwrap();
    let path = Path::new(&filename);

    let mut file = File::open(path).unwrap();
    let mut rom = [0u8; MEMORY_SIZE - PC_START as usize];
    file.read(&mut rom).unwrap();

    cpu.memory[PC_START as usize..].clone_from_slice(&rom);

    // println!("{:02x}", cpu.memory.iter().format(""));

    'running: loop {
        let opcode = u16::from(cpu.memory[cpu.pc as usize]) << 8
            | u16::from(cpu.memory[cpu.pc as usize + 1]);
        // println!("{:02X} {:02X}", cpu.pc, opcode);
        cpu.pc += 2;

        let nnn = opcode & 0x0FFF;
        let x = (opcode & 0x0F00 >> 8) as u8;
        let y = (opcode & 0x00F0 >> 4) as u8;
        let vx = cpu.v[x as usize];
        let vy = cpu.v[y as usize];
        let kk = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;

        match (opcode & 0xF000) >> 12 {
            0x0 => match opcode {
                0x00E0 => {
                    cpu.display = [0; DISPLAY_SIZE];
                    cpu.should_draw = true;
                }
                0x00EE => {
                    cpu.sp -= 1;
                    cpu.pc = cpu.stack[cpu.sp as usize];
                }
                _ => {
                    unimplemented!();
                }
            },
            0x1 => {
                cpu.pc = nnn;
            }
            0x2 => {
                cpu.stack[cpu.sp as usize] = cpu.pc;
                cpu.sp += 1;
                cpu.pc = nnn;
            }
            0x3 => {
                if vx == kk {
                    cpu.pc += 2;
                }
            }
            0x4 => {
                if vx != kk {
                    cpu.pc += 2;
                }
            }
            0x5 => {
                if vx == vy {
                    cpu.pc += 2;
                }
            }
            0x6 => {
                cpu.v[x as usize] = kk;
            }
            0x7 => cpu.v[x as usize] = vx.wrapping_add(kk),
            0x8 => match opcode & 0x000F {
                0x0 => {
                    cpu.v[x as usize] = vy;
                }
                0x1 => {
                    cpu.v[x as usize] = vx | vy;
                }
                0x2 => {
                    cpu.v[x as usize] = vx & vy;
                }
                0x3 => {
                    cpu.v[x as usize] = vx ^ vy;
                }
                0x4 => {
                    cpu.v[x as usize] = vx.wrapping_add(vy);
                    cpu.v[0xf] = if vx > 0xff - vy { 1 } else { 0 }
                }
                0x5 => {
                    cpu.v[x as usize] = vx.wrapping_sub(vy);
                    cpu.v[0xf] = if vx > vy { 1 } else { 0 }
                }
                0x6 => {
                    cpu.v[x as usize] = vx >> 1;
                    cpu.v[0xf] = vx & 0x1;
                }
                0x7 => {
                    cpu.v[x as usize] = vy.wrapping_sub(vx);
                    cpu.v[0xf] = if vy > vx { 1 } else { 0 };
                }
                0xe => {
                    cpu.v[x as usize] = vx << 1;
                    cpu.v[0xf] = (vx & 0x80) >> 7;
                }
                _ => unimplemented!(),
            },
            0x9 => {
                if vx != vy {
                    cpu.pc += 2;
                }
            }
            0xa => {
                cpu.i = nnn;
            }
            0xb => cpu.pc = nnn + u16::from(cpu.v[0]),
            0xc => cpu.v[x as usize] = rng.gen_range(0, 255) & kk,
            0xd => {
                cpu.v[0xf] = 0;
                for yl in 0..n {
                    let pixel = cpu.memory[usize::from(cpu.i + yl as u16)];
                    for xl in 0..8 {
                        if pixel & (0x80 >> xl) != 0 {
                            let index_x = vx as usize + xl as usize;
                            let index_y = (vy as usize + yl as usize) * DISPLAY_WIDTH;
                            let index = (index_x + index_y) % DISPLAY_SIZE;
                            if cpu.display[index] == 1 {
                                cpu.v[0xf] = 1;
                            }
                            cpu.display[index] ^= 1;
                        }
                    }
                }
                cpu.should_draw = true
            }
            0xe => match opcode & 0x00FF {
                0x9e => {
                    if cpu.keys & (1 << vx) != 0 {
                        cpu.pc += 2;
                    }
                }
                0x1a => {
                    if cpu.keys & (1 << vx) == 0 {
                        cpu.pc += 2;
                    }
                }
                _ => unimplemented!(),
            },
            0xf => match opcode & 0x00FF {
                0x07 => {
                    cpu.v[x as usize] = cpu.dt;
                }
                0x0a => unimplemented!(),
                0x15 => {
                    cpu.dt = vx;
                }
                0x18 => {
                    cpu.st = vx;
                }
                0x1e => {
                    cpu.i = cpu.i + vx as u16;
                }
                0x29 => cpu.i = vx as u16 * 5,
                0x33 => {
                    cpu.memory[cpu.i as usize] = vx / 100;
                    cpu.memory[cpu.i as usize + 1] = (vx / 10) % 10;
                    cpu.memory[cpu.i as usize + 2] = vx % 10;
                }
                0x55 => {
                    for i in 0..=x {
                        cpu.memory[cpu.i as usize + i as usize] = cpu.v[i as usize];
                    }
                }
                0x65 => {
                    for i in 0..=x {
                        cpu.v[i as usize] = cpu.memory[cpu.i as usize + i as usize]
                    }
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }

        if cpu.st > 0 {
            cpu.st -= 1;
        }
        if cpu.dt > 0 {
            cpu.dt -= 1;
        }

        if cpu.should_draw {
            cpu.should_draw = false;
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();

            for x in 0..DISPLAY_WIDTH {
                for y in 0..DISPLAY_HEIGHT {
                    let index = x + y * DISPLAY_WIDTH;
                    if cpu.display[index] != 0 {
                        canvas.set_draw_color(Color::RGB(255, 255, 255));
                    } else {
                        canvas.set_draw_color(Color::RGB(0, 0, 0));
                    }
                    canvas
                        .fill_rect(Rect::new(
                            (x * PIXEL_SIZE) as i32,
                            (y * PIXEL_SIZE) as i32,
                            PIXEL_SIZE as u32,
                            PIXEL_SIZE as u32,
                        ))
                        .unwrap();
                }
            }

            canvas.present();
        }
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        timer.delay(1000 / 60);
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
