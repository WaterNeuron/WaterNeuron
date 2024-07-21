CANISTER_BUILD = """
package(default_visibility = ["//visibility:public"])
filegroup(
    name = "wasm",
    srcs = ["{wasm_file}"],
)

"""

CANISTER_URL_TEMPLATE = "https://download.dfinity.systems/ic/{version}/canisters/{wasm_file}"

def _ic_canister_impl(repository_ctx):
    wasm_file = repository_ctx.attr.wasm_file
    version = repository_ctx.attr.version
    sha256 = repository_ctx.attr.sha256

    repository_ctx.report_progress("Fetching %s" % wasm_file)
    url = CANISTER_URL_TEMPLATE.format(version = version, wasm_file = wasm_file)
    repository_ctx.report_progress("URL: %s" % url)

    repository_ctx.download(
        url = url,
        output = wasm_file,
        sha256 = sha256,
    )

    build_file = "BUILD.bazel"
    repository_ctx.file(build_file, CANISTER_BUILD.format(wasm_file = wasm_file), executable = True)

ic_canister = repository_rule(
    implementation = _ic_canister_impl,
    attrs = {
        "wasm_file": attr.string(mandatory = True),
        "version": attr.string(mandatory = True),
        "sha256": attr.string(mandatory = True),
    },
)
