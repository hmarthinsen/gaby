[![Build Status](https://travis-ci.com/hmarthinsen/gaby.svg?branch=master)](https://travis-ci.com/hmarthinsen/gaby)

# Gaby
A simple Game Boy emulator written in Rust.
NB! This is a work in progress.
It is not possible to play games on this emulator yet.

## Some notes

### Synchronization

The main loop of the emulator performs one tick of the system clock.
The subsystems of the emulator, like e.g. the CPU or the video system, are responsible for updating themselves through a `tick` function that each subsystem must implement.
Each subsystem has to keep track of how many cycles their own operations are to take.

### Subsystems

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

#### CPU

At tick, it checks if the current instruction has finished executing.
If not, nothing happens.
Else, the next instruction is fetched, decoded and executed.

Each instruction takes the form of an opcode, plus up to two operands.

#### DMA

#### Video

#### Audio

#### Timer

#### Interrupts

### Traits

An object implementing `Source` can output either a `u8` or `u16`.
An object implementing `Target` accepts a `u8` or `u16`.

### Design

An instruction has one or two operands that can be either immediate, register or indirect.
Register and indirect can be used both as source and target operands, but immediate only as source.
Instructions are implemented as functions that take operands as arguments.
The operands implement the `Source` and/or `Target` traits.
This way, the instructions can be written in a general way, abstracting memory access details.
