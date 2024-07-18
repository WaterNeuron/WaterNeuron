"""
Compute the SHA-256 hash of a file.
"""

def _hash_impl(ctx):
    output = ctx.actions.declare_file(ctx.label.name + ".sha256")
    ctx.actions.run_shell(
        outputs = [output],
        inputs = [ctx.file.artifact],
        command = "sha256sum {} | tee {} >&2".format(ctx.file.artifact.path, output.path),
    )
    return [DefaultInfo(files = depset([output]))]

hash_rule = rule(
    implementation = _hash_impl,
    attrs = {
        "src": attr.label(mandatory = True, allow_single_file = True),
    },
)
