#!/bin/bash
cd $(dirname $0)
wasm-pack build --target nodejs --scope dr666m1
npm publish pkg --access public
