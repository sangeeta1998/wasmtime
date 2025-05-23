<div align="center">
<<<<<<< HEAD
  <h1><code>wasmtime</code></h1>

  <p>
    <strong>A standalone runtime for
    <a href="https://webassembly.org/">WebAssembly</a></strong>
  </p>

  <strong>A <a href="https://bytecodealliance.org/">Bytecode Alliance</a> project</strong>

  <p>
    <a href="https://github.com/bytecodealliance/wasmtime/actions?query=workflow%3ACI"><img src="https://github.com/bytecodealliance/wasmtime/workflows/CI/badge.svg" alt="build status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/217126-wasmtime"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <img src="https://img.shields.io/badge/rustc-stable+-green.svg" alt="supported rustc stable" />
    <a href="https://docs.rs/wasmtime"><img src="https://docs.rs/wasmtime/badge.svg" alt="Documentation Status" /></a>
  </p>

  <h3>
    <a href="https://bytecodealliance.github.io/wasmtime/">Guide</a>
    <span> | </span>
    <a href="https://bytecodealliance.github.io/wasmtime/contributing.html">Contributing</a>
    <span> | </span>
    <a href="https://wasmtime.dev/">Website</a>
    <span> | </span>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/217126-wasmtime">Chat</a>
  </h3>
</div>

## Installation

The Wasmtime CLI can be installed on Linux and macOS (locally) with a small install
script:

```sh
curl https://wasmtime.dev/install.sh -sSf | bash
```
This script installs into `$WASMTIME_HOME` (defaults to `$HOME/.wasmtime`), and executable is placed in `$WASMTIME_HOME/bin`.

Windows or otherwise interested users can download installers and
binaries directly from the [GitHub
Releases](https://github.com/bytecodealliance/wasmtime/releases) page.

Documentation on Wasmtime's currently supported versions can be found [in the
online book
documentation](https://docs.wasmtime.dev/stability-release.html#current-versions).

## Example

If you've got the [Rust compiler
installed](https://www.rust-lang.org/tools/install) then you can take some Rust
source code:

```rust
fn main() {
    println!("Hello, world!");
}
```

and compile/run it with:

```sh
$ rustup target add wasm32-wasip1
$ rustc hello.rs --target wasm32-wasip1
$ wasmtime hello.wasm
Hello, world!
```

(Note: make sure you installed Rust using the `rustup` method in the official
instructions above, and do not have a copy of the Rust toolchain installed on
your system in some other way as well (e.g. the system package manager). Otherwise, the `rustup target add...`
command may not install the target for the correct copy of Rust.)

## Features

* **Fast**. Wasmtime is built on the optimizing [Cranelift] code generator to
  quickly generate high-quality machine code either at runtime or
  ahead-of-time. Wasmtime is optimized for efficient instantiation, low-overhead
  calls between the embedder and wasm, and scalability of concurrent instances.

* **[Secure]**. Wasmtime's development is strongly focused on correctness and
  security. Building on top of Rust's runtime safety guarantees, each Wasmtime
  feature goes through careful review and consideration via an [RFC
  process]. Once features are designed and implemented, they undergo 24/7
  fuzzing donated by [Google's OSS Fuzz]. As features stabilize they become part
  of a [release][release policy], and when things go wrong we have a
  well-defined [security policy] in place to quickly mitigate and patch any
  issues. We follow best practices for defense-in-depth and integrate
  protections and mitigations for issues like Spectre. Finally, we're working to
  push the state-of-the-art by collaborating with academic researchers to
  formally verify critical parts of Wasmtime and Cranelift.

* **[Configurable]**. Wasmtime uses sensible defaults, but can also be
  configured to provide more fine-grained control over things like CPU and
  memory consumption. Whether you want to run Wasmtime in a tiny environment or
  on massive servers with many concurrent instances, we've got you covered.

* **[WASI]**. Wasmtime supports a rich set of APIs for interacting with the host
  environment through the [WASI standard](https://wasi.dev).

* **[Standards Compliant]**. Wasmtime passes the [official WebAssembly test
  suite](https://github.com/WebAssembly/testsuite), implements the [official C
  API of wasm](https://github.com/WebAssembly/wasm-c-api), and implements
  [future proposals to WebAssembly](https://github.com/WebAssembly/proposals) as
  well. Wasmtime developers are intimately engaged with the WebAssembly
  standards process all along the way too.

[Wasmtime]: https://github.com/bytecodealliance/wasmtime
[Cranelift]: https://cranelift.dev/
[Google's OSS Fuzz]: https://google.github.io/oss-fuzz/
[security policy]: https://bytecodealliance.org/security
[RFC process]: https://github.com/bytecodealliance/rfcs
[release policy]: https://docs.wasmtime.dev/stability-release.html
[Secure]: https://docs.wasmtime.dev/security.html
[Configurable]: https://docs.rs/wasmtime/latest/wasmtime/struct.Config.html
[WASI]: https://docs.rs/wasmtime-wasi/latest/wasmtime_wasi/
[Standards Compliant]: https://docs.wasmtime.dev/stability-tiers.html

## Language Support

You can use Wasmtime from a variety of different languages through embeddings of
the implementation.

Languages supported by the Bytecode Alliance:

* **[Rust]** - the [`wasmtime` crate]
* **[C]** - the [`wasm.h`, `wasi.h`, and `wasmtime.h` headers][c-headers], [CMake](crates/c-api/CMakeLists.txt) or [`wasmtime` Conan package]
* **C++** - the [`wasmtime.hh` header][c-headers] or the [`wasmtime-cpp` Conan package]
* **[Python]** - the [`wasmtime` PyPI package]
* **[.NET]** - the [`Wasmtime` NuGet package]
* **[Go]** - the [`wasmtime-go` repository]
* **[Ruby]** - the [`wasmtime` gem]

Languages supported by the community:

* **[Elixir]** - the [`wasmex` hex package]
* **Perl** - the [`Wasm` Perl package's `Wasm::Wasmtime`]

[Rust]: https://bytecodealliance.github.io/wasmtime/lang-rust.html
[C]: https://bytecodealliance.github.io/wasmtime/lang-c.html
[`wasmtime` crate]: https://crates.io/crates/wasmtime
[c-headers]: https://bytecodealliance.github.io/wasmtime/c-api/
[Python]: https://bytecodealliance.github.io/wasmtime/lang-python.html
[`wasmtime` PyPI package]: https://pypi.org/project/wasmtime/
[.NET]: https://bytecodealliance.github.io/wasmtime/lang-dotnet.html
[`Wasmtime` NuGet package]: https://www.nuget.org/packages/Wasmtime
[Go]: https://bytecodealliance.github.io/wasmtime/lang-go.html
[`wasmtime-go` repository]: https://pkg.go.dev/github.com/bytecodealliance/wasmtime-go
[`wasmtime` Conan package]: https://conan.io/center/wasmtime
[`wasmtime-cpp` Conan package]: https://conan.io/center/wasmtime-cpp
[Ruby]: https://bytecodealliance.github.io/wasmtime/lang-ruby.html
[`wasmtime` gem]: https://rubygems.org/gems/wasmtime
[Elixir]: https://docs.wasmtime.dev/lang-elixir.html
[`wasmex` hex package]: https://hex.pm/packages/wasmex
[`Wasm` Perl package's `Wasm::Wasmtime`]: https://metacpan.org/pod/Wasm::Wasmtime

## Documentation

[📚 Read the Wasmtime guide here! 📚][guide]

The [wasmtime guide][guide] is the best starting point to learn about what
Wasmtime can do for you or help answer your questions about Wasmtime. If you're
curious in contributing to Wasmtime, [it can also help you do
that][contributing]!

[contributing]: https://bytecodealliance.github.io/wasmtime/contributing.html
[guide]: https://bytecodealliance.github.io/wasmtime

---

It's Wasmtime.
=======
  <h1><code>wasi-nn</code></h1>

  <strong>A <a href="https://bytecodealliance.org/">Bytecode Alliance</a> project</strong>

  <p><strong>High-level bindings for writing wasi-nn applications</strong></p>

  <p>
    <a href="https://github.com/bytecodealliance/wasi-nn/actions?query=workflow%3ACI">
      <img src="https://github.com/bytecodealliance/wasi-nn/workflows/CI/badge.svg" alt="CI status"/>
    </a>
    <a href="https://crates.io/crates/wasi-nn">
      <img src="https://img.shields.io/crates/v/wasi-nn.svg"/>
    </a>
    <a href="https://www.npmjs.com/package/as-wasi-nn">
      <img src="https://img.shields.io/npm/v/as-wasi-nn.svg"/>
    </a>
  </p>

</div>

### Introduction

This project provides high-level wasi-nn bindings for Rust and AssemblyScript. The basic idea: write
your machine learning application in a high-level language using these bindings, compile it to
WebAssembly, and run it in a WebAssembly runtime that supports the [wasi-nn] proposal, such as
[Wasmtime] and [WasmEdge].

[Wasmtime]: https://wasmtime.dev
[wasi-nn]: https://github.com/WebAssembly/wasi-nn
[WasmEdge]: https://github.com/WasmEdge/WasmEdge

> __NOTE__: These bindings are experimental (use at your own risk) and subject to upstream changes
> in the [wasi-nn] specification.


### Use

 - In Rust, download the [crate from crates.io][crates.io] by adding `wasi-nn = "0.6.0"` as a Cargo
   dependency; more information in the [Rust README].
 - In AssemblyScript, download the [package from npmjs.com][npmjs.com] by adding `"as-wasi-nn":
   "^0.3.0"` as an NPM dependency; more information in the [AssemblyScript README].
 - When you call Wasmtime, you'll need to pass the flag `--wasi-modules=experimental-wasi-nn` to
   enable the use use of wasi-nn.
 - For WasmEdge, you should install the [wasi-nn plugin] first.

[crates.io]: https://crates.io/crates/wasi-nn
[Rust README]: rust/README.md
[npmjs.com]: https://www.npmjs.com/package/wasi-nn
[AssemblyScript README]: assemblyscript/README.md
[wasi-nn plugin]: https://wasmedge.org/docs/category/ai-inference

### Examples

This repository includes examples of using these bindings. See the [Rust example] and
[AssemblyScript example] to walk through an end-to-end image classification using an AlexNet model.
Currently the example uses OpenVino as the backend. If you are running Ubuntu, you can simply run
the script to install the supported version`.github/actions/install-openvino/install.sh`. Otherwise
you'll need to visit the [Installation Guides] and follow the instructions for your OS. The version
of OpenVino currently supported is openvino_2022.1.0.643.

Once you have OpenVino installed, run them with:
 - `./build.sh rust` runs the [Rust example]
 - `./build.sh as` runs the [AssemblyScript example]

[Rust example]: rust/examples/classification-example
[AssemblyScript example]: assemblyscript/examples/object-classification.ts
[Installation Guides]: https://docs.openvino.ai/2023.3/openvino_docs_install_guides_overview.html

To run examples in WasmEdge, consult this article: [WasmEdge wasi-nn examples].

[WasmEdge wasi-nn examples]: https://github.com/second-state/WasmEdge-WASINN-examples

### Related Links

- [WASI]
- [wasi-nn]
- [Wasmtime]
- [WasmEdge]
- [AssemblyScript]
- [OpenVino]

[WASI]: https://github.com/WebAssembly/WASI
[AssemblyScript]: https://www.assemblyscript.org/
[OpenVino]: https://docs.openvinotoolkit.org/latest/index.html

### License

This project is licensed under the Apache 2.0 license. See [LICENSE] for more details.

[LICENSE]: LICENSE


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this project by you, as defined in the Apache-2.0 license, shall be licensed as above, without any
additional terms or conditions.
>>>>>>> 629c15c7a3de2e3c918f2a7b0612c5be32a53268
