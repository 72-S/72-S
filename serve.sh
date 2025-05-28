#!/bin/zsh
wasm-pack build --target web --out-dir pkg
rm -rf dist 
mkdir -p dist/js
mkdir -p dist/style
cp -r static/* dist/
cp -r style/* dist/style/
cp -r js/* dist/js/
cp -r pkg dist/
npx serve dist

