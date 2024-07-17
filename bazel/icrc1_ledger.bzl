LEDGER_BUILD = """
package(default_visibility = ["//visibility:public"])
exports_files(["ic-icrc1-ledger.wasm.gz"])
"""

LEDGER_VERSION = "922a89e6b521529fac3d131c3922cd13011dc600"
LEDGER_SHA256 = "9282046dc95ea6af0c12a74779b224247850c727de3999053ff67a8aa2aba281"

LEDGER_URL = "https://download.dfinity.systems/ic/{version}/canisters/ic-icrc1-ledger.wasm.gz"

def _icrc1_ledger_impl(repository_ctx):
    repository_ctx.report_progress("Fetching ic-icrc1-ledger.wasm.gz")
    repository_ctx.download(
        url = LEDGER_URL.format(version = LEDGER_VERSION),
        output = "ic-icrc1-ledger.wasm.gz",
        sha256 = LEDGER_SHA256
    )

    repository_ctx.file("BUILD.bazel", LEDGER_BUILD, executable = True)

_icrc1_ledger = repository_rule(
    implementation = _icrc1_ledger_impl,
    attrs = {},
)

def icrc1_ledger(name):
    _icrc1_ledger(name = name)
