#!/bin/zsh
wasm-pack build --target web --out-dir pkg
rm -rf dist 
mkdir -p dist/js
cp static/index.html dist/
cp js/index.js dist/js/
cp -r pkg dist/
npx serve dist

