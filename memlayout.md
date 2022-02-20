# Program Memory Layout

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

| Method Index (addr) | Method Address | Method Local Variable Array Size |
|:--------------------|:---------------|:---------------------------------|
| 0 (`0x0000`)        | `0x0015`       | `0x0004`                         |
| 1 (`0x0004`)        | `0x0028`       | `0x0002`                         |
| 2 (`0x0008`)        | `0x0040`       | `0x0003`                         |
