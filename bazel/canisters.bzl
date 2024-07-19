"""
This module defines utilities for building Rust canisters.
"""

load("@bazel_skylib//rules:copy_file.bzl", "copy_file")
load("@rules_rust//rust:defs.bzl", "rust_binary")
load("//bazel:defs.bzl", "gzip_compress")

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

    # Invokes canister WebAssembly module optimizer and attaches the candid file.
    native.genrule(
        name = name + ".opt",
        srcs = [name + ".raw", service_file],
        outs = [name + ".opt.wasm"],
        testonly = kwargs.get("testonly"),
        message = "Shrinking canister " + name,
        tools = ["@crate_index//:ic-wasm__ic-wasm"],
        cmd_bash = """
        ic-wasm $(location {input_wasm}) -o $@ shrink && \
        ic-wasm $@ -o $@ metadata candid:service --visibility public --file $(location {service_file})
        """.format(input_wasm = name + ".raw", service_file = service_file),
    )

    inject_version_into_wasm(
        name = name + "_with_version.opt",
        src_wasm = name + ".opt",
        # The version file is expected to contain a single line with the git
        version_file = "//version:git_commit_id",
    )

    gzip_compress(
        name = name + ".wasm",
        srcs = [name + "_with_version.opt"],
        testonly = kwargs.get("testonly"),
    )

    copy_file(
        name = name + "-wasm.gz",
        src = name + ".wasm",
        out = name + ".wasm.gz",
        testonly = kwargs.get("testonly"),
    )

def inject_version_into_wasm(*, name, src_wasm, version_file):
    """Generates an output file named `name + '.wasm'`.

    The output file is almost identical to the input (i.e. `src_wasm`), except
    that it has an additional piece of metadata attached to in the form of a
    WASM custom section named `icp:public git_commit_id` (no quotes, of course),
    whose value is the contents of version_file (minus the trailing
    newline character).
    """
    native.genrule(
        name = name,
        srcs = [
            src_wasm,
            version_file,
        ],
        outs = [name + ".wasm"],
        message = "Injecting version into wasm.",
        tools = ["@crate_index//:ic-wasm__ic-wasm"],
        cmd_bash = " ".join([
            "$(location @crate_index//:ic-wasm__ic-wasm)",
            "$(location %s)" % src_wasm,  # Input file.
            "--output $@",  # Output file.
            "metadata",  # Subcommand

            # The name of the custom section will be
            # "icp:public git_commit_id"
            "git_commit_id",
            "--visibility public",

            # Get value to inject from version_file.
            "--file $(location " + version_file + ")",
        ]),
    )
