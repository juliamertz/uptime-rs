#!/usr/bin/env bash

tailwind=./node_modules/.bin/tailwindcss
output_file=./static/_dist.css

echo "Building CSS..."

$tailwind --postcss                   \
          --minify                    \
          --input ./static/styles.css \
          --output $output_file       \
          $@
          
          # &> /dev/null
