#!/bin/env bash

set -x
set -eo

# will effectively be executed in the dist folder generated with `trunk build --release --public-url $(<"github_url.txt")`

# Assign arguments to variables
old_url=$(<"github_url.txt")
new_url=$CF_PAGES_URL

# Find and replace the URL
find "$PWD" -type f -exec sed -i "s|$old_url|$new_url|Ig" {} +
