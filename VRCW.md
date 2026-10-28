# VRCW recovery fork

`vrcw-2` adds an opt-in UdonSharp source emitter to the `vrcw-1` base
(`5249fc1d501ddeb0dedf4af7fb076fd0e7c0c281`). The existing `dc`, `dasm`
and raw-byte dumper retain their output.

```sh
udon-decompiler us --module-info UdonModuleInfo.json program.hex ClassName.cs
```

Run `tools/Editor/UdonModuleInfoExtractor.cs` in the target VRChat SDK project.
Beside module metadata it exports `UdonEvents.json` (builtin events and argument
symbols/types) and `UdonSyncModes.json` (the SDK's behaviour sync values).
The emitter reads the event table beside module metadata, or through `--event-info`.
Use `--sync-mode None|Continuous|Manual|NoVariableSync` only when all behaviours
using the program agree on that mode.

The emitter preserves the recorded class identity, exported fields, sync metadata
and local names. Typed enum fields retain heap enum types and values. Unsupported
heap values fail with `unsupported constant <type>` instead of a null placeholder.
The output can still fail the SDK compiler or differ from the original program;
source emission alone never authorizes replacing a running program. VRCW compiles
and compares lifts in an isolated SDK-only project before installation.

Validation:

```sh
cargo test -q
cargo clippy --all-targets -- -D warnings -A clippy::chunks_exact_to_as_chunks -A clippy::question_mark
```

The two Clippy allowances cover existing upstream warnings on newer Rust versions.
Use the checksum-pinned clang-format-node 3.0.0 formatter (LLVM 22.1.1) for the
golden tests. SDK verification used VRChat Worlds 3.10.4 and Unity 2022.3.22f1.

Release archives contain the native Rust executable, self-contained .NET 10
UdonProgramDumper, Editor scripts, clang-format and licenses. Build Linux archives
on Ubuntu 22.04/glibc 2.35; build Windows archives on Windows. The private VRCW
`linux-forks.yml` workflow builds both platform archives. Publishing requires
separate approval.
