LEDGER_BUILD = """
package(default_visibility = ["//visibility:public"])
exports_files(["ic-icrc1-ledger.wasm.gz"])
"""

# this is not the latest
LEDGER_VERSION = "7eace38b7580dc35af53b9180ea40480af4977dc"
LEDGER_SHA256 = "54aa8fd9172e7e59a39e55c70b3c8a159cf3d8f4c1b2b1284aa3b80cf82ff207"

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
