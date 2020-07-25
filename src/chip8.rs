use rand::{rngs::ThreadRng, Rng};
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::path::Path;

pub struct Chip8 {
    pub pc: usize,
    pub v: [u8; 16],
    pub mem: [u8; 4096],
    pub stack: Vec<usize>,
    pub framebuffer: [u8; 64 * 32],
    pub i: usize,
    pub rng: ThreadRng,
    pub keys: [bool; 16],
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub draw: bool,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            pc: 0x200,
            v: [0; 16],
            mem: [0; 4096],
            stack: Vec::new(),
            framebuffer: [0; 64 * 32],
            i: 0,
            rng: rand::thread_rng(),
            keys: [false; 16],
            sound_timer: 0,
            delay_timer: 0,
            draw: false,
        };
        chip8.load_fontset();
        chip8
    }

    pub fn execute_cycle(&mut self) {
        self.draw = false;
        let opcode = self.get_opcode();

        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as u8;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = (opcode & 0x0FFF) as usize;

        match (opcode & 0xF000) >> 12 {
            0x0 => {
                match opcode & 0x00FF {
                    // Clear the screen.
                    0xE0 => {
                        for byte in self.framebuffer.iter_mut() {
                            *byte = 0;
                        }
                        self.pc += 2;
                    }
                    // Return from subroutine.
                    0xEE => {
                        self.pc = self.stack.pop().unwrap();
                    }
                    // Execute machine language subroutine at NNN.
                    _ => unreachable!(),
                }
            }
            // Jump to address NNN.
            0x1 => {
                self.pc = nnn as usize;
            }
            // Execute subroutine at NNN.
            0x2 => {
                self.stack.push(self.pc + 2);
                self.pc = nnn as usize;
            }
            // Skip the following instruction if the value of register VX equals NN.
            0x3 => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // Skip the following instruction if the value of register VX is not equal to NN.
            0x4 => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // Skip the following instruction if the value of register VX is equal to the value of register VY.
            0x5 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // Store number NN in register VX.
            0x6 => {
                self.v[x] = nn;
                self.pc += 2;
            }
            // Add the value NN to register VX.
            0x7 => {
                self.v[x] = self.v[x].wrapping_add(nn);
                self.pc += 2;
            }
            0x8 => {
                match opcode & 0x000F {
                    // Store the value of register VY in register VX.
                    0x0 => {
                        self.v[x] = self.v[y];
                    }
                    // Set VX to VX OR VY.
                    0x1 => {
                        self.v[x] = self.v[x] | self.v[y];
                    }
                    // Set VX to VX AND VY.
                    0x2 => {
                        self.v[x] = self.v[x] & self.v[y];
                    }
                    // Set VX to VX XOR VY.
                    0x3 => {
                        self.v[x] = self.v[x] ^ self.v[y];
                    }
                    /* Set Vx = Vx + Vy, set VF = carry.
                    The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1,
                    otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx. */
                    0x4 => {
                        let (sum, overflow) = self.v[x].overflowing_add(self.v[y]);
                        self.v[x] = sum;
                        if overflow {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                    }
                    /* Set Vx = Vx - Vy, set VF = NOT borrow.
                    If Vx > Vy, then VF is set to 1, otherwise 0.
                    Then Vy is subtracted from Vx, and the results stored in Vx. */
                    0x5 => {
                        let (subtraction, borrow) = self.v[x].overflowing_sub(self.v[y]);
                        self.v[x] = subtraction;
                        if borrow {
                            self.v[0xF] = 0;
                        } else {
                            self.v[0xF] = 1;
                        }
                    }
                    /* Store the value of register VY shifted right one bit in register VX¹
                    Set register VF to the least significant bit prior to the shift
                    VY is unchanged. */
                    0x6 => {
                        self.v[0xF] = self.v[y] & 0x1;
                        self.v[x] = self.v[y] >> 1;
                    }
                    /* Set Vx = Vy - Vx, set VF = NOT borrow.
                    If Vy > Vx, then VF is set to 1, otherwise 0.
                    Then Vx is subtracted from Vy, and the results stored in Vx. */
                    0x7 => {
                        let (subtraction, borrow) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[x] = subtraction;
                        if borrow {
                            self.v[0xF] = 0;
                        } else {
                            self.v[0xF] = 1;
                        }
                    }
                    /* Store the value of register VY shifted left one bit in register VX
                    Set register VF to the most significant bit prior to the shift
                    VY is unchanged. */
                    0xE => {
                        self.v[0xF] = (self.v[y] & 0x80) >> 7;
                        self.v[x] = self.v[y] << 1;
                    }
                    _ => unreachable!(),
                }
                self.pc += 2;
            }
            // Skip the following instruction if the value of register VX is not equal to the value of register VY.
            0x9 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // Store memory address NNN in register I.
            0xA => {
                self.i = nnn as usize;
                self.pc += 2;
            }
            // Jump to address NNN + V0.
            0xB => {
                self.pc = nnn + self.v[0] as usize;
            }
            // Set VX to a random number with a mask of NN.
            0xC => {
                let r: u8 = self.rng.gen();
                self.v[x] = r & nn;
                self.pc += 2;
            }
            /* Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
            The interpreter reads n bytes from memory, starting at the address stored in I.
            These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
            Sprites are XORed onto the existing screen. If this causes any pixels to be erased,
            VF is set to 1, otherwise it is set to 0.
            If the sprite is positioned so part of it is outside the coordinates of the display,
            it wraps around to the opposite side of the screen. */
            0xD => {
                let vx = self.v[x];
                let vy = self.v[y];
                let mut did_erase = false;

                let sprite_data = self.mem.get(self.i..self.i + n as usize).unwrap();
                for row in 0..n {
                    let byte = sprite_data[row as usize];
                    for bit in 0..8 {
                        let pixel_value = (byte & (0x80 >> bit)) >> (7 - bit);
                        let x_coord = (vx + bit) as usize % 64; // Modulo to wrap around.
                        let y_coord = (vy + row) as usize % 32;
                        let buffer_index = x_coord + (y_coord * 64);

                        if self.framebuffer[buffer_index as usize] == 1 && pixel_value == 1 {
                            did_erase = true;
                        }

                        self.framebuffer[buffer_index as usize] ^= pixel_value;
                    }
                }

                if did_erase {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }

                self.draw = true;
                self.pc += 2;
            }
            0xE => {
                match opcode & 0x00FF {
                    // 	Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed.
                    0x9E => {
                        if self.keys[self.v[x] as usize] == true {
                            self.keys[self.v[x] as usize] = false; 
                            self.pc += 2;
                        }
                        self.pc += 2;
                    }
                    // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed.
                    0xA1 => {
                        if self.keys[self.v[x] as usize] == false {
                            self.pc += 2;
                        }
                        self.pc += 2;
                    }
                    _ => unreachable!(),
                };
            }
            0xF => {
                match opcode & 0x00FF {
                    // Store the current value of the delay timer in register VX.
                    0x07 => {
                        self.v[x] = self.delay_timer;
                        self.pc += 2;
                    }
                    /* Wait for a key press, store the value of the key in Vx.
                    All execution stops until a key is pressed, then the value of that key is stored in Vx. */
                    0x0A => {
                        if let Some(key_index) = self.keys.iter().position(|&key| key == true) {
                            self.v[x] = key_index as u8;
                            self.keys[key_index] = false;
                            self.pc += 2;
                        }
                    }
                    // Set delay timer = Vx. DT is set equal to the value of Vx.
                    0x15 => {
                        self.delay_timer = self.v[x];
                        self.pc += 2;
                    }
                    // Set sound timer = Vx. DT is set equal to the value of Vx.
                    0x18 => {
                        self.sound_timer = self.v[x];
                        self.pc += 2;
                    }
                    // Set I = I + Vx. The values of I and Vx are added, and the results are stored in I.
                    0x1E => {
                        self.i = self.i + self.v[x] as usize;
                        self.pc += 2;
                    }
                    /* Set I = location of sprite for digit Vx.
                    The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. */
                    0x29 => {
                        self.i = self.v[x] as usize * 5;
                        self.pc += 2;
                    }
                    /* Store BCD representation of Vx in memory locations I, I+1, and I+2.
                    The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I,
                    the tens digit at location I+1, and the ones digit at location I+2.  */
                    0x33 => {
                        self.mem[self.i] = self.v[x] / 100;
                        self.mem[self.i + 1] = (self.v[x] / 10) % 10;
                        self.mem[self.i + 2] = (self.v[x] % 100) % 10;
                        self.pc += 2;
                    }
                    /* Store registers V0 through Vx in memory starting at location I.
                    The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
                    I is set to I + X + 1 after operation. */
                    0x55 => {
                        for i in 0..=x {
                            self.mem[self.i + i] = self.v[i];
                        }
                        self.i = self.i + x + 1;
                        self.pc += 2;
                    }
                    /* Read registers V0 through Vx from memory starting at location I.
                    The interpreter reads values from memory starting at location I into registers V0 through Vx.
                    I is set to I + X + 1 after operation. */
                    0x65 => {
                        for i in 0..=x {
                            self.v[i] = self.mem[self.i + i];
                        }
                        self.i = self.i + x + 1;
                        self.pc += 2;
                    }
                    _ => unreachable!(),
                };
            }
            _ => unreachable!(),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        if self.sound_timer == 1 {
            println!("Beep!");
        }
    }

    pub fn load_program(&mut self, path: &Path) -> Result<(), Error> {
        let program = File::open(path)?;
        for (i, byte) in program.bytes().enumerate() {
            self.mem[0x200 + i] = byte?;
        }
        Ok(())
    }

    fn get_opcode(&self) -> u16 {
        ((self.mem[self.pc] as u16) << 8) | self.mem[self.pc + 1] as u16
    }

    fn load_fontset(&mut self) {
        let fontset: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        for (i, byte) in fontset.iter().enumerate() {
            self.mem[i] = *byte;
        }
    }
}
