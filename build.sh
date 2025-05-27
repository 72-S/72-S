#!/bin/zsh
wasm-pack build --target web --out-dir pkg
rm -rf dist 
mkdir -p dist/js
mkdir -p dist/style
cp static/index.html dist/
cp style/styles.css dist/style/
cp js/index.js dist/js/
cp -r pkg dist/

