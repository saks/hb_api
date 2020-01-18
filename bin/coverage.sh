#!/usr/bin/env bash

# check cargo installation:
type cargo &>/dev/null
if [[ $? -gt 0 ]]; then
  echo "Please install rust:"
  echo "https://www.rust-lang.org/tools/install"
  echo
  exit 1
fi

# check jq installation:
type jq &>/dev/null
if [[ $? -gt 0 ]]; then
  echo "Please install jq:"
  echo "https://stedolan.github.io/jq/download/"
  echo
  exit 1
fi

# check kcov installation:
type kcov &>/dev/null
if [[ $? -gt 0 ]]; then
  echo "Please install kcov:"
  echo "https://github.com/SimonKagstrom/kcov/blob/master/INSTALL.md"
  echo
  exit 1
fi

set -e

export RUST_TEST_THREADS=1
WORKSPACE_ROOT=$(pwd)
COVERAGE_DIR=$WORKSPACE_ROOT/coverage
TEST_INPUT=$(tempfile)

rm -rf $COVERAGE_DIR
mkdir $COVERAGE_DIR

cargo build --all --tests --message-format=json \
  | jq -s '.[] | select(.profile.test == true)' \
  | jq -s 'map({name: .target.name, file: .executable})' \
  | jq -c '.[]' > $TEST_INPUT

for crate in $(cat $TEST_INPUT); do
  eval $(echo $crate | jq -r '@sh "name=\(.name) test_file=\(.file)"')
  crate_dir="${WORKSPACE_ROOT}/${name}"
  cd ${crate_dir}
  kcov --verify --include-path=$WORKSPACE_ROOT --exclude-pattern=tests.rs $COVERAGE_DIR $test_file;
done

unlink $TEST_INPUT

