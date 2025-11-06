#!/bin/bash
(cd wasm && wasm-pack build --debug --target web --out-dir ../frontend/src/wasm/wasm_pkg)