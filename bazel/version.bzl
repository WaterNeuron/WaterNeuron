BUILD_VERSION = """
package(default_visibility = ["//visibility:public"])

filegroup(
    name = "git_commit_id",
    srcs = ["git_commit_id.txt"],
)
"""

def _git_version_impl(repository_ctx):
    repository_ctx.report_progress("Fetching git commit ID")

    result = repository_ctx.execute(["git", "rev-parse", "HEAD"])
    if result.return_code != 0:
        fail("Failed to get git commit ID: " + result.stderr)

    commit_id = result.stdout.strip()

    repository_ctx.file("git_commit_id.txt", commit_id)
    repository_ctx.file("BUILD.bazel", BUILD_VERSION, executable = False)

git_version = repository_rule(
    implementation = _git_version_impl,
    attrs = {},
)
