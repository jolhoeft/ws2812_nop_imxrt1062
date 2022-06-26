# Use ws2812 leds on imxrt1062 boards

- For usage with `smart-leds`
- Implements the `SmartLedsWrite` trait

## WARNING
WS2812s operate at 5v.
IMXRT1062 pads operate at 3v and are not 5v tolerant.
You will need a logic-level converter and some care in wiring.


## Why?

Because I'm apparently too stupid to get a periodic timer working with ws2812-timer-delay.

## How?

This library uses a simple `nop` loop in assembly to wait.

WS2812s read inputs with cycles about 333ns long.
Write 3 bytes (GRB | Green, Red, Blue) _**per LED**_, then "latch" (set low) for 6us to 250us depending on your model.
Writing a single bit entails a three-bit message that looks like `[1, x, 0]`, where x is the bit you want to write.
That all is to say, to write a bit of `1` to an LED you must send `[1, 1, 0]`, waiting 333ns between each bit.

Despite WS2812 cycles allegedly being 333ns,
some of the time constraints are very tight, and others are very loose.

Find out more about the [timing constraints of WS2812s](https://wp.josh.com/2014/05/13/ws2812-neopixels-are-not-so-finicky-once-you-get-to-know-them/).

P.S. I have the latch hard coded to 6us. If you need 250us shoot me an email or clone the project.

## Example using teensy4-bsp

```rust

#[cortex_m_rt::entry]
fn main() -> ! {
    ...

    let mut data: [RGB8; 3] = [RGB8::default(); 3];
    let empty: [RGB8; 3] = [RGB8::default(); 3];

    data[0] = RGB8 {
        r: 0x10,
        g: 0x00,
        b: 0x00,
    };
    data[1] = RGB8 {
        r: 0x00,
        g: 0x10,
        b: 0x00,
    };
    data[2] = RGB8 {
        r: 0x00,
        g: 0x00,
        b: 0x10,
    };

    configure(&mut pins.p2,{
        Config::zero()
            .set_drive_strength(DriveStrength::R0_7)
            .set_speed(Speed::Max)
            .set_slew_rate(SlewRate::Fast)
    });
    let pin = GPIO::new(pins.p2).output();

    let mut ws = ws2812_nop_imxrt1062::Ws2812::new(pin, 600.0);

    loop {
        ws.write(data.iter().cloned()).unwrap();
        systick.delay_ms(500);

        ws.write(empty.iter().cloned()).unwrap();
        systick.delay_ms(500);
    }
}
```
