# Using RuSTy

## Program Structure

### Single Files

A full program can be contained within a single file, typically with a `.st` extention.
The RuSTy compiler will parse the entire program as a single file and produce a Compilation Unit containing all symbols or types in the program.

The compiler can be called for a single file using the `rustyc` command

#### Example

Given the program `my_prog.st` as follows
```st
VAR_GLOBAL
    gX : INT;
END_VAR

PROGRAM mainProg
    gX := 1;
END_PROGRAM
```

The program can be compiled with: 

```sh
rustyc my_prog.st
```

---
**NOTE**

The previous command will generate a dynamically linkable object with position independent code called `a.out`

See : [Compiling a linkable object](#compiling-a-linkable-object)

---

### Multiple 

Passing multiple files to the compiler is currently not supported.
To compile multiple files, each file should be generated independently.
To handle internal dependencies, each file needs a declaration of every datatype or POU it would reference.
For more information see [Working with External files - TODO]()

## Compiling a static object

## Compiling a linkable object

## Creating a shared library 

## Linking with an external application

## Writing a main
