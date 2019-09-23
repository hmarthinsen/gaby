# Gaby
A simple Game Boy emulator written in Rust.

## Some notes

### Traits

An object implementing `Read` can output either a `u8` or `u16`.
An object implementing `Write` accepts a `u8` or `u16`.

A `Word` or `Byte` can implement one or both of these traits.

### Design

An instruction has one or two operands that can be either immediate, register or indirect.
Register and indirect can be used both as source and target operands, but immediate only as source.
Instructions are implemented as functions that operands as arguments.
The operands implement the `Read` and/or `Write` traits.
This way, the instructions can be written in a general way, abstracting memory access details.
