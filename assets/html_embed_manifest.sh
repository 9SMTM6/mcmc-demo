#!/bin/bash

# Load the JSON content from a file
json_data=$(cat "./assets/manifest.json")

# URL encode the JSON string
url_encoded_json=$(echo "$json_data" | jq -sRr @uri)

# Create the Data URL
data_url="data:application/json,${url_encoded_json}"

manifest_link_el="<link rel=\"manifest\" href='$data_url' />"

echo "$manifest_link_el" > ./assets/manifest.html
