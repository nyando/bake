# Bake - Bali Project Builder and Runner Application

`bake` is a command line tool for converting Java `.class` files into a format directly usable by the Bali Java Processor.
To accomplish this, `bake` offers the following functionality:

- decompilation of `.class` files (extraction of methods and corresponding bytecode)
- emission of a program binary that matches Bali memory organization
- functionality for writing the binary to the FPGA running Bali via serial output

`bake` is developed in Rust and uses the [`binrw`](https://github.com/jam1garner/binrw) library for parsing Java class files.

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

## Memory Layout and Structure

Bali, similar to the JVM, uses a Harvard architecture model for executing its code.
Program memory and data memory are kept separately, with program memory being written once and read-only during execution.
Data memory is distributed over the following memory areas:

- method call stack
- evaluation stack
- local variable array
- static memory area

Within the Bali architecture, these areas are managed jointly by the top-level CPU module and the control module within the CPU.
The control module is structured in such a way that it only needs to know about method-local execution.
This includes not having access to the _actual_ local variable array addresses, but only the method-local indices.
The CPU module keeps track of the method's local variable address offset and adds that to each index set by the control module.
The LVA offsets are tracked in the call stack along with return addresses.

## Method Index Table

In order to retrieve information on how to construct a procedure frame,
the following method information is stored in a LUT at the beginning of program memory:

- method address in program memory
- number of arguments to the method (i. e. number of elements to pop from the stack when invoking the method)
- size of the local variable array of the method (larger than or equal to the number of arguments)

The `bake` tool assigns an order to the methods, with `main` always being the first.
Therefore, the data block starting at address `0x0000` contains the address and number of local variables of the `main` function.
Although `main` always takes an array of `String`s as an argument,
this argument is ignored in Bali, since Bali does not handle `String` objects.
The method address has a length of 16 bits.
Argument count and method LVA size are both limited to 256 and thus require only one byte each.

| Method Index | Method Address | Argument Count | Method LVA Size |
|:-------------|:---------------|:---------------|:----------------|
| 0 (`0x0000`) | `0x0015`       | `0x02`         | `0x04`          |
| 1 (`0x0004`) | `0x0028`       | `0x01`         | `0x02`          |
| 2 (`0x0008`) | `0x0040`       | `0x02`         | `0x03`          |

The same table also stores 32-bit constant values extracted from the Java class file constant pool.
The corresponding references in the code are replaced with the index of the constant.
This means that the argument to the `ldc` instruction is replaced by a corresponding LUT index.

## `.class` File Translation

The code segments of the `.class` file are mostly copied directly into the Bali binary.
However, the following instructions are modified while copying the code:

- `invokestatic` - The constant pool reference is replaced by a method LUT index (see the section _Method Index Table_).
- `ldc` - The constant pool reference is replaced by a method LUT index (see the section _Method Index Table_).
- `iinc x y` - This instruction is equivalent to executing the instructions `iload x`, `bipush y`, `iadd`, `istore x`,
  so it is treated as a macro and replaced with those four instructions during translation.
  This is done to reduce complexity in the implementation of the Bali processor.
