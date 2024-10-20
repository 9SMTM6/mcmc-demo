#!/bin/env sh

# set -x
# set -eo

# will effectively be executed in the dist folder generated with `trunk build --release --public-url $(<"github_url.txt")`

old_url=__RENAME_ME__
new_url=$1
# In old times (this breaks caching for main pages) $CF_PAGES_URL

# Find and replace the URL
find "$PWD" -type f -exec sed -i "s|$old_url|$new_url|Ig" {} +
