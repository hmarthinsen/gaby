# Gaby

[![Rust](https://github.com/hmarthinsen/gaby/actions/workflows/rust.yml/badge.svg)](https://github.com/hmarthinsen/gaby/actions/workflows/rust.yml)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fhmarthinsen%2Fgaby.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Fhmarthinsen%2Fgaby?ref=badge_shield)

A simple Game Boy emulator written in Rust.
NB! This is a work in progress.
It is not possible to play games on this emulator yet.

## Synchronization

The main loop of the emulator performs one tick of the system clock.
The subsystems of the emulator, like e.g. the CPU or the video system, are responsible for updating themselves through a `tick` function that each subsystem must implement.
Each subsystem has to keep track of how many cycles their own operations are to take.

## Subsystems

Subsystems are the units that do something each cycle.
They all need access to the Game Boy memory.

Subsystem | Description
--- | ---
CPU | Fetches, decodes and executes instructions.
DMA | Transfers data.
Video | Renders the output video buffer, line by line.
Audio | Renders the output audio buffer.
Timer | Triggers an interrupt after a given number of cycles.
Interrupts | Dispatch interrupts.
Serial I/O | *Not implemented.*

### CPU

At tick, it checks if the current instruction has finished executing.
If not, nothing happens.
Else, the next instruction is fetched, decoded and executed.

Each instruction takes the form of an opcode, plus up to two operands.

### DMA

TODO

### Video

TODO

### Audio

The audio subsystem consists of four sound generators:

1. rectangle wave with sweep and envelope,
2. rectangle wave with envelope,
3. digital wave,
4. white noise with envelope.

### Timer

TODO

### Interrupts

TODO

## Traits

An object implementing `Source` can output either a `u8` or `u16`.
An object implementing `Target` accepts a `u8` or `u16`.

## Design

An instruction has one or two operands that can be either

- [OK] immediate (data follows instruction directly in memory),
- register (data in register), or
- indirect (data in memory).

Depending on how the address is stored, indirect can be either

- indirect register (address in word register or `0xFF00 + C`),
- indirect immediate (address is immediate word), or
- indirect high immediate (address is `0xFF00` + immediate byte).

Register and indirect can be used both as source and target operands, but immediate only as source.

Instructions are implemented as functions that take operands as arguments.
The operands implement the `Source` and/or `Target` traits.
This way, the instructions can be written in a general way, abstracting memory access details.

The instructions are member functions of the CPU object, which owns the registers, so the instructions always have access to the registers, but the other data that the instructions operate on, like memory, have to come in via the operands. This can lead to problems if e.g. the instruction is going to both read and write to memory, which would require the two operands to both borrow memory, but one of them borrowing mutably.

To avoid this, the source operand always contains a copy of the data, and doesn't borrow anything.
