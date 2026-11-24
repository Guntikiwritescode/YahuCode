# YahuCode

YahuCode is a satirical esoteric programming language. Running a program prints an OFFICIAL account and an ACTUAL account of the same events, plus a count of the claims that were false.

It replaces four ordinary compiler ideas with political ones.

- Types become clearance. A value's type is who may read it, and a type error is a disclosure.
- Memory becomes a coalition. An allocation lasts only while it is bribed, and running out of support means elections.
- Halting becomes staying in power. A program stops only when the government falls.
- The output is the diff between the OFFICIAL face and the ACTUAL face.

## Build and run

YahuCode is written in Rust and builds with version 1.78 or later.

```sh
cargo build
cargo run -- examples/02_protective_edge.yahu
cargo test
```

The flags, the syntax and the diagnostics are described in [LANGUAGE.md](LANGUAGE.md). Sources for the real events that the language refers to are in [docs/sources.md](docs/sources.md).
