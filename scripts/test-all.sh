#!/usr/bin/env bash

# This script runs all tests, including model tests. In order to do
# so, models are downloaded and stored in $XDG_CACHE_HOME/syntaxdot.

set -euo pipefail
IFS=$'\n\t'

if ! [ -x "$(command -v curl)" ] ; then
  >&2 echo "'curl' is required for downloading test data"
  exit 1
fi

cache_dir="${XDG_CACHE_HOME:-$HOME/.cache}/syntaxdot-ffi"

declare -A models=(
  ["DUTCH_UD_SMALL"]="https://github.com/stickeritis/sticker2-models/releases/download/nl-ud-small-20200907/nl-ud-small-20200907.tar.gz"
)

if [ ! -d "$cache_dir" ]; then
  mkdir -p "$cache_dir"
fi

for var in "${!models[@]}"; do
  url="${models[$var]}"
  data="${cache_dir}/$(basename "${url}" .tar.gz)"

  # Assumption: if the archive is named foobar.tar.gz, than it contains
  # the directory foobar.
  if [ ! -e "${data}" ]; then
    curl -L "${url}" | tar -C "$cache_dir" -zx
  fi

  declare -x "${var}"="${data}"
done

cargo test --features "model-tests"