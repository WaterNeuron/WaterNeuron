load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

CANDID_BUILD = """
package(default_visibility = ["//visibility:public"])
filegroup(
    name = "candid",
    srcs = ["{file_name}"],
)
"""

GITHUB_RAW_URL_TEMPLATE = "https://raw.githubusercontent.com/dfinity/ic/{commit}/{file_path}"

def _ic_candidfile_impl(repository_ctx):
    commit = repository_ctx.attr.commit
    file_path = repository_ctx.attr.file_path
    sha256 = repository_ctx.attr.sha256
    file_name = file_path.split("/")[-1]

    repository_ctx.report_progress("Fetching %s" % file_name)
    url = GITHUB_RAW_URL_TEMPLATE.format(commit = commit, file_path = file_path)
    repository_ctx.report_progress("URL: %s" % url)

    repository_ctx.download(
        url = url,
        output = file_name,
        sha256 = sha256,
    )

    build_file = "BUILD.bazel"
    repository_ctx.file(build_file, CANDID_BUILD.format(file_name = file_name), executable = False)

ic_candidfile = repository_rule(
    implementation = _ic_candidfile_impl,
    attrs = {
        "commit": attr.string(mandatory = True),
        "file_path": attr.string(mandatory = True),
        "sha256": attr.string(mandatory = True),
    },
)
