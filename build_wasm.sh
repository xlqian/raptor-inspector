#!/bin/bash
(cd wasm && wasm-pack build --target web --out-dir ../frontend/src/wasm/wasm_pkg)