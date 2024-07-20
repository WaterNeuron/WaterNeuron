"""
This module defines utilities for building Rust canisters.
"""

load("@rules_rust//rust:defs.bzl", "rust_binary")

def _wasm_rust_transition_impl(_settings, _attr):
    return {
        "//command_line_option:platforms": "@rules_rust//rust/platform:wasm",
    }

wasm_rust_transition = transition(
    implementation = _wasm_rust_transition_impl,
    inputs = [],
    outputs = [
        "//command_line_option:platforms",
    ],
)

def _wasm_binary_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".wasm")
    ctx.actions.run(
        executable = "cp",
        arguments = [ctx.files.binary[0].path, out.path],
        outputs = [out],
        inputs = ctx.files.binary,
    )

    return [DefaultInfo(files = depset([out]), runfiles = ctx.runfiles([out]))]

wasm_rust_binary_rule = rule(
    implementation = _wasm_binary_impl,
    attrs = {
        "binary": attr.label(mandatory = True, cfg = wasm_rust_transition),
    },
)

def rust_canister(name, service_file, **kwargs):
    """Defines a rust program that builds into a WebAssembly module.
    Args:
        name: the name of the target that produces a Wasm module.
        **kwargs: additional arguments to pass a rust_binary rule.
    """

    wasm_name = "_wasm_" + name.replace(".", "_")
    kwargs.setdefault("visibility", ["//visibility:public"])
    kwargs.setdefault("rustc_flags", [
        "-C",
        "link-args=-z stack-size=3145728",
        "-C",
        "linker-plugin-lto",
        "-C",
        "opt-level=3",
        "-C",
        "debug-assertions=no",
        "-C",
        "debuginfo=0",
        "-C",
        "lto",
        "-C",
        "embed-bitcode=yes",
    ])

    rust_binary(
        name = wasm_name,
        crate_type = "bin",
        **kwargs
    )

    wasm_rust_binary_rule(
        name = name,
        binary = ":" + wasm_name,
    )
