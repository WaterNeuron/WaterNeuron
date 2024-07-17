"""
Utilities for building IC replica and canisters.
"""

_COMPRESS_CONCURRENCY = 16

def _compress_resources(_os, _input_size):
    """ The function returns resource hints to bazel so it can properly schedule actions.

    Check https://bazel.build/rules/lib/actions#run for `resource_set` parameter to find documentation of the function, possible arguments and expected return value.
    """
    return {"cpu": _COMPRESS_CONCURRENCY}

def _gzip_compress(ctx):
    """GZip-compresses source files.
    """
    out = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.run_shell(
        command = "{pigz} --processes {concurrency} --no-name {srcs} --stdout > {out}".format(pigz = ctx.file._pigz.path, concurrency = _COMPRESS_CONCURRENCY, srcs = " ".join([s.path for s in ctx.files.srcs]), out = out.path),
        inputs = ctx.files.srcs,
        outputs = [out],
        tools = [ctx.file._pigz],
        resource_set = _compress_resources,
    )
    return [DefaultInfo(files = depset([out]), runfiles = ctx.runfiles(files = [out]))]

gzip_compress = rule(
    implementation = _gzip_compress,
    attrs = {
        "srcs": attr.label_list(allow_files = True),
        "_pigz": attr.label(allow_single_file = True, default = "@pigz"),
    },
)
