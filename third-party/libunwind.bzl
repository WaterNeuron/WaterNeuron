load("@rules_foreign_cc//foreign_cc:defs.bzl", "configure_make")

package(default_visibility = ["//visibility:public"])

configure_make(
    name = "libunwind",
    configure_options = [
        "--enable-shared=yes",
        "--enable-static=yes",
    ],
    lib_source = ".",
    out_static_libs = ["libunwind.a"],
    out_headers_only = True,
    includes = ["include"],
)

cc_library(
    name = "libunwind_headers",
    hdrs = glob(["include/**/*.h"]),
    includes = ["include"],
    visibility = ["//visibility:public"],
)

cc_library(
    name = "libunwind_static",
    srcs = ["libunwind.a"],
    hdrs = glob(["include/**/*.h"]),
    includes = ["include"],
    visibility = ["//visibility:public"],
)
