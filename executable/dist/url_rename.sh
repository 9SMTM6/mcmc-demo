#!/bin/env sh

set -x
set -eo

sub_page_placeholder=__SUB_PAGE__
root_page_placeholder=__ROOT_PAGE__
stable_page_placeholder=__STABLE_PAGE__
new_release_url=$1
new_stable_url=$2
# In old times (this breaks the PWA setup for main pages, but correctly deploys development pages) $CF_PAGES_URL

# Find and replace the URLs:

# subpages (fat and slim version, links on main page will go to the built release specifically)
find "$PWD/slim" -type f -exec sed -i "s|$sub_page_placeholder|$new_release_url/slim|Ig" {} +
find "$PWD/fat" -type f -exec sed -i "s|$sub_page_placeholder|$new_release_url/fat|Ig" {} +

# root page, will go to current release
find "$PWD/slim" -type f -exec sed -i "s|$root_page_placeholder|$new_release_url|Ig" {} +
find "$PWD/fat" -type f -exec sed -i "s|$root_page_placeholder|$new_release_url|Ig" {} +
find "$PWD/assets" -type f -exec sed -i "s|$root_page_placeholder|$new_release_url|Ig" {} +
sed -i "s|$root_page_placeholder|$new_release_url|Ig" index.html

# currently only used in manifest. This is the `installed` URL, this should lead to users always getting the newest version
find "$PWD/slim" -type f -exec sed -i "s|$stable_page_placeholder|$new_stable_url|Ig" {} +
find "$PWD/fat" -type f -exec sed -i "s|$stable_page_placeholder|$new_stable_url|Ig" {} +
find "$PWD/assets" -type f -exec sed -i "s|$stable_page_placeholder|$new_stable_url|Ig" {} +
sed -i "s|$stable_page_placeholder|$new_stable_url|Ig" index.html
