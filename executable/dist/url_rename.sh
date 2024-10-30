#!/bin/env sh

set -x
set -eo

old_url=__RENAME_ME__
new_url=$1
# In old times (this breaks the PWA setup for main pages, but correctly deploys development pages) $CF_PAGES_URL

# Find and replace the URL
find "$PWD/slim" -type f -exec sed -i "s|$old_url|$new_url/slim|Ig" {} +

find "$PWD/fat" -type f -exec sed -i "s|$old_url|$new_url/fat|Ig" {} +
