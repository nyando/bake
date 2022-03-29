# Bake - Bali Project Builder and Runner Application

`bake` is a command line tool for converting Java `.class` files into a format directly usable by the Bali Java Processor.
To accomplish this, `bake` will offer the following functionality:

- decompilation of `.class` files (extraction of methods and corresponding bytecode)
- emission of a program binary that matches Bali memory organization
- functionality for writing the binary to the FPGA running Bali

`bake` is developed in Rust and uses the [`binrw`](https://github.com/jam1garner/binrw)
and [`rust-csv`](https://github.com/BurntSushi/rust-csv) packages for parsing.

## Functionality

Command format: `bake [COMMAND]`

- `binary` - Emit a binary file to write into the Bali processor program memory.
  - `--classfile [CLASSFILE]`: Java Class File to convert to Bali binary format.
- `consts` - Extract constants from a Java `.class` file.
  - `--classfile [CLASSFILE]`: Java Class File to convert to Bali binary format.
- `method` - Parse method structures and display their bytecode.
  - `--classfile [CLASSFILE]`: Java Class File to convert to Bali binary format.
- `serial` - Write a Bali binary to a processor via a UART connection.
  - `--bin [BINARY]`: Bali binary to write to device.
  - `--device [DEVICE]`: Device name (`/dev` file on Linux or `COM`-Port on Windows).

## Program Memory Layout

Bali, similar to the JVM, uses a Harvard architecture model for executing its code.
Program memory and data memory are kept separately, with program memory being written once and read-only during execution.
Data memory is distributed over the following memory areas:

- thread stack
- evaluation stack
- local variable array
- static memory area ("heap", with a static collection of one or more objects)

## Method Index Table

In order to retrieve information on how to construct a procedure frame,
method information such as address in memory and maximum number of local variables is stored in a LUT at the beginning of memory.
The `bake` tool assigns some order to the methods, with `main` always being the first.
Therefore, the data block starting at address `0x0000` contains the address and number of local variables of the `main` function.
The address has a length of 16 bits, while the maximum number of local variables is 256, so the number requires a single byte.

| Method Index | Method Address | Argument Count | Method LVA Size |
|:-------------|:---------------|:---------------|:----------------|
| 0 (`0x0000`) | `0x0015`       | `0x02`         | `0x04`          |
| 1 (`0x0004`) | `0x0028`       | `0x01`         | `0x02`          |
| 2 (`0x0008`) | `0x0040`       | `0x02`         | `0x03`          |

The same table also stores 32-bit constant values extracted from the Java class file constant pool.
The corresponding references in the code are replaced with the index of the constant.

