
# nhotyp-lang

Nhotyp is a conceptual language designed for ease of implementation during my tutoring in an introductive algorithmic course at Harbin Institute of Technology, Weihai. The assignment specifications were written in Chinese, though. An English version might be added if requested.

Nhotyp is an interpretative language imitating a few features in Python and Rust, and used prefix expressions for ease of parsing. It was so designed to make the assignment easier to complete, if one chose to think the problem through, as it required few string operations and would never require the construction of an AST just to function properly.

The said repository introduces a standard implementation which would work on correct implementations, and should report common runtime errors if it was not written properly.

## Usage

Build the compiler with Rust and execute your Nhotyp code with the compiled intepreter:

```
cargo build
cargo run your_code.nh
```

You may find some samples in the `samples/` folder.

## Specifications

Currently, English specs are yet to be announced, and a Chinese version is available at `README_zh.md`.
