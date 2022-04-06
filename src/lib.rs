//! # Use ws2812 leds on imxrt1062 boards
//!
//! - For usage with `smart-leds`
//! - Implements the `SmartLedsWrite` trait
//!
//! ## Why?
//!
//! Because I'm apparently too stupid to get a periodic timer working with ws2812-timer-delay.
//!
//! ## How?
//!
//! This library uses a simple for loop with no-ops inside.
//! How many loops is determined by a magic number.
//!
//! The default magic_number is 111.
//!
//! WS2812s read inputs with cycles about 333ns long.
//! Write 3 bytes (GRB | Green, Red, Blue) _**per LED**_, then "latch" (set low) for 6us to 250us depending on your model.
//! Writing a single bit entails a three-bit message that looks like `[1, x, 0]`, where x is the bit you want to write.
//! That all is to say, to write a bit of `1` to an LED you must send `[1, 1, 0]`, waiting 333ns between each bit.
//!
//!
//! As for the magic number, I have no idea why that works.
//! According to my calculations, at 600MHz a cycle should take 1.667 nanoseconds.
//! The loop inside takes about 6 cycles. In theory that means 333ns should be about 33 loops.
//! In practice, it takes around 110 loops. This baffles me, so if you can explain it, please send me an email or something.
//! dwbrite@gmail.com
//!
//! My best guess is maybe _something_ is being pipelined, but ???
//!
//! Anyway, I just plugged in oscilloscope to find out what works, and as it turns out,
//! a magic number of 111 works for both 600MHz and 720MHz.

#![no_std]

use core::arch::asm;
use embedded_hal::digital::v2::OutputPin;

use smart_leds_trait::{SmartLedsWrite, RGB8};

pub struct Ws2812<PIN> {
    pin: PIN,
    magic_number: u8,
}

impl<PIN> Ws2812<PIN> where PIN: OutputPin {
    /// The timer has to already run at with a frequency of 3 MHz
    pub fn new(mut pin: PIN) -> Ws2812<PIN> {
        pin.set_low().ok();
        Self { pin, magic_number: 111 }
    }

    pub fn set_magic_number(&mut self, num: u8) {
        self.magic_number = num;
    }

    fn wait(&mut self) {
        for _ in 0..(self.magic_number) {
            unsafe {
                asm!("nop", "nop", "nop", "nop");
            }
        }
    }

    fn write_bit(&mut self, bit: bool) {
        if bit {
            self.pin.set_high().ok();
            self.wait();
            self.wait();
            self.pin.set_low().ok();
            self.wait();
        } else {
            self.pin.set_high().ok();
            self.wait();
            self.pin.set_low().ok();
            self.wait();
            self.wait();
        }
    }

    fn write_byte(&mut self, mut data: u8) {
        // 8 * 3 * 333ns = ~8us
        for _ in 0..8 {
            self.write_bit((data & 0x80) != 0);
            data <<= 1;
        }
    }
}

impl<PIN> SmartLedsWrite for Ws2812<PIN> where PIN: OutputPin {
    type Error = ();
    type Color = RGB8;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
        where
            T: Iterator<Item = I>,
            I: Into<Self::Color>,
    {
        for item in iterator {
            let item = item.into();
            self.write_byte(item.g);
            self.write_byte(item.r);
            self.write_byte(item.b);
        }

        // wait at least 6 microseconds
        for _ in 0..20 {
            self.wait();
        }
        Ok(())
    }
}
