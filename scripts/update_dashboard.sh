#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo $SCRIPT_DIR
pushd $SCRIPT_DIR/../tools/evaluation
python ./evaluate.py
pushd ../dashboard
node_modules/.bin/webpack
yes | cp public/bundle.js ../../docs/
popd
popd
