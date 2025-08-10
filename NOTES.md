# Presentation Notes

## Fighting Bots

### Explain Fighting Robots

- Battle bot videos?
- Talk about regulations a little
    - 12 lb limit or so on
    - not autonomous - so remote controlled

## How to make a (bad) fighting robot

### Supplies

#### Hardware

(What are these used for?)

- Controller
- Motor drivers (get more acquianted with why we need a driver)
- Motors
- Development board (why a specific board?)
- Power supply
- Some kind of weapon?

#### Firmware

##### Pick a language?

- C
- Micropython
- Rust (🦀)

----- 

# Making Rust In Piece

## Why Rust?

- Perforamnce
- Safety critical (i want all my fingers)
- (tasks?)

## Code

### Embassy

> Embassy is the next-generation framework for embedded applications. Write safe, correct, and energy-efficient embedded code faster, using the Rust programming language, its async facilities, and the Embassy libraries.

#### (Brief) Talk about setup

- Dependencies
- Build file
- Probe-rs

#### Component driven approach

- Playstation controller
- Driving motors
- Actuating weapon
- Safety
- Dual cores


# Notes

Probably remove motor stuff in code

Look more into motor drivers

Talk about voltage and stepping down

How does PWM work with driving motors