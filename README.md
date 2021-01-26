# QuiZX: a speedy Rust port of PyZX

[PyZX](https://github.com/Quantomatic/pyzx) is a Python library for quantum circuit optimisation and compiling using the [ZX-calculus](https://zxcalculus.com). It's great for hacking, learning, and trying things out in [Jupyter](https://jupyter.org/) notebooks. However, it's written to maximise clarity and fun, not performance.

This is a port of some of the core functionality of PyZX to the [Rust](https://www.rust-lang.org/) programming language. This is a modern systems programming language, which enables writing software that is very fast and memory efficient.

To build from source, first [get Rust](https://www.rust-lang.org/tools/install), then use the included package and build manager `cargo`.

    git clone https://github.com/Quantomatic/quizx.git
    cd quizx
    cargo build
    cargo test

This will download the dependendies, build in debug mode and run the tests. There is a miscellaneous collection of programs written using the library, found in `src/bin`. To build in release (i.e. fast) mode, run:

    cargo build --release

Then, run one of the binaries via the `cargo run` command, or directly via e.g.

    cd target/release
    ./spider_chain

## A bit about performance

As a very anecdotal example of the performance difference, the program `spider_chain` builds a chain of 1 million green spiders and fuses them all. In PyZX, you can fuse all the spiders in a ZX-diagram as follows:

```python
from pyzx.basicrules import *

success = True
while success
    success = any(fuse(g, g.edge_s(e), g.edge_t(e)) for e in g.edges()):
```

In QuiZX, the Rust code is slightly more verbose, but similar in spirit:
```rust
use quizx::basic_rules::*;

loop {
    match g.find_edge(|v0,v1,_| check_spider_fusion(&g, v0, v1)) {
        Some((v0,v1,_)) => spider_fusion_unsafe(&mut g, v0, v1),
        None => break,
    };
}
```

On my laptop, the PyZX code takes about 98 seconds to fuse 1 million spiders, whereas the QuiZX code takes 17 milliseconds.

## TODO

QuiZX is very much a work in progress. It is not intended to have all the features of PyZX, but certainly the core stuff. Here's where it's at:

- ZX-diagrams
  - [X] building ZX-diagrams and doing basic graph manipulations
  - [X] converting ZX-diagrams to Z + hadamard form
  - [X] switchable underlying graph model (fast vector-based model for sparse graphs, slower hash-based model for dense graphs)
- ZX-calculus rules
  - [X] spider fusion
  - [X] local complementation
  - [X] pivoting
  - [X] remove identity spiders
  - [X] colour-change
  - [ ] pivoting variations (boundary-pivot and gadget-pivot)
  %% - [ ] strong complementarity (optional, pivoting is more useful in practice)
- tensor evaluation based on [ndarray](https://github.com/rust-ndarray/ndarray)
  - [X] exact scalars with [cyclotomic](https://en.wikipedia.org/wiki/Cyclotomic_field)
      rational numbers
  - [X] floating point scalars based on [num_complex](https://crates.io/crates/num-complex)
  - [X] tensor contraction for arbitrary ZX-diagrams
  - [X] equality of tensors with exact scalars
  %% - [ ] approximate equality of tensors with floating point scalars
  %% - [ ] space optimisations
  %% - [ ] choose good contraction ordering (currently uses
  %%       reverse-insertion-order)
  %% - [ ] more human-readable tensor output (e.g. converting to normal matrices, pretty printing)
- circuits
  - [X] circuit data type
  - [X] read and write QASM
  - [X] conversion from circuits to ZX-diagrams
  - [ ] circuit extraction

Pull requests are welcome!

