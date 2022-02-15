# Bake - Bali Project Builder and Runner Application

`bake` is a command line tool for converting Java `.class` files into a format directly usable by the Bali Java Processor.
To accomplish this, `bake` will offer the following functionality:

- decompilation of `.class` files (extraction of methods and corresponding bytecode)
- emission of a program binary that matches Bali memory organization
- functionality for writing the binary to the FPGA running Bali

`bake` is developed in Rust and uses the [`binrw`](https://github.com/jam1garner/binrw)
and [`rust-csv`](https://github.com/BurntSushi/rust-csv) packages for parsing.

## Functionality

Command format: `bake [COMMAND] [CLASS FILE]`

- `consts` - Extracts constants from a Java `.class` file.
- `methods` - Parse method structures and display their bytecode.
- `gen` - Emit a binary file to write into the Bali processor program memory.
