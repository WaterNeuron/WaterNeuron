#!/usr/bin/env bash

set -eEuo pipefail 

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <old_commit> <new_commit>"
    exit 1
fi

REPO_URL="https://github.com/dfinity/ic.git"
OLD_COMMIT="$1"
NEW_COMMIT="$2"


TEMP_DIR=$(mktemp -d)
echo "Working in temporary directory: $TEMP_DIR"

git clone "$REPO_URL" "$TEMP_DIR/ic"
pushd "$TEMP_DIR/ic" > /dev/null
git checkout $NEW_COMMIT

temp_git=$(mktemp)
temp_bazel=$(mktemp)
temp_common=$(mktemp)
temp_commits=$(mktemp)

# Get Git changed files and write directly to temp file
git diff --name-only $OLD_COMMIT | xargs -I {} realpath "{}" | sort > "$temp_git"

# Get Bazel target dependencies and write directly to temp file
bazel query --output=location "filter('^//', kind('source file', deps(//ic-os/guestos/envs/prod:prod)))" | sed 's/:[0-9]*:[0-9]*: .*$//' | sort > "$temp_bazel"

# Use comm to find common files and save to a temporary file
comm -12 "$temp_git" "$temp_bazel" > "$temp_common"

echo "Files that have changed and are dependencies of the Bazel target:"
cat "$temp_common"
echo

echo "Collecting unique commits..."
while IFS= read -r file; do
    git log --oneline $OLD_COMMIT..$NEW_COMMIT -- "$file" >> "$temp_commits"
done < "$temp_common"

echo "Unique commits that modified the relevant files, with PR links:"
sort -u "$temp_commits" | while read -r line; do
    commit_hash=$(echo "$line" | cut -d' ' -f1)
    commit_msg=$(echo "$line" | cut -d' ' -f2-)
    pr_number=$(echo "$commit_msg" | grep -oP '#\K\d+')
    if [ -n "$pr_number" ]; then
        pr_url="$REPO_URL/pull/$pr_number"
        echo "$line"
        echo "PR: $pr_url"
        echo
    else
        echo "$line"
        echo "No PR number found in commit message"
        echo
    fi
done

# Clean up temporary files
rm "$temp_git" "$temp_bazel" "$temp_common" "$temp_commits"
