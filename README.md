# Intel's 8086 Disassembler & Emulator

## About

Here I implement a partial intel 8086 disassembler + emulator. This does not plan to be a complete implementation by any means the main goal of this is to better understand the internals of a CPU, in other words, what are the problems a CPU have to solve when we throw code at it.

## Pre-requisites

* Go 1.25
* nasm (Mac binary provided)
* ndisasm (Mac binary provided)

## How To Run

The easiest way to run it is just running [./test.sh](https://github.com/jorovipe97/performance-aware-homework/blob/main/test.sh) file, this files does the following:

1. Compiles an 8086 binary using nasm and an original assembly file.
2. Run my emulator so it loads the produced binary, it outputs the assembly and the final values of registers and flags.
3. Runs again nasm but this time using the assembly produced in step 2.
4. Runs cmp to check if binary produced at step 1 and 3 are equal.

### Final Notes

* This repo solves [Casey Muratori's](https://www.computerenhance.com/p/table-of-contents) hands on homeworks.
* This project was written entirely without the use of AI as the purpose is to learn.
