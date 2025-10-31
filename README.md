# raptor-viewer Starter

This is a browser-based data visualization project using Rust (compiled to WebAssembly), TypeScript, and HTML/CSS. Heavy processing is handled in Rust (WASM), UI and interactivity in TypeScript. The minimal version lets you echo strings through Rust for TS↔WASM round-trip verification.

## Prerequisites

- Node.js (preferably v18+, but at least matching Vite/Leaflet requirements)
- Rust + Cargo
- wasm-pack (`cargo install wasm-pack`)

## Project Structure

```
raptor-viewer/
  README.md           # This file
  build_wasm.sh       # Helper script to build WASM bundle
  wasm/               # Rust crate for core processing (compiled to WASM)
    src/lib.rs        # Rust entry; exposes JS-bindgen interfaces
    tests/            # Rust tests for core logic
  frontend/
    index.html        # Main HTML page
    src/
      main.ts         # Main TypeScript app entry
      wasm/echo.ts    # TS interface for WASM
      wasm_pkg/       # Output from wasm-pack build
    test/main.test.ts # TypeScript unit tests
    style.css         # App styles
```

## Setup & Run

1. **Install dependencies**
   - Rust: https://www.rust-lang.org/tools/install
   - wasm-pack: `cargo install wasm-pack`
   - Node.js / npm: https://nodejs.org/

2. **Build the Rust WASM package:**
   ```bash
   ./build_wasm.sh
   # Or, manually:
   # cd wasm && wasm-pack build --target web --out-dir ../frontend/src/wasm/wasm_pkg
   ```

3. **Install frontend dependencies:**
   ```bash
   (cd frontend && npm install)
   ```

4. **Run the development server:**
   ```bash
   npm run dev
   ```
   - Open the browser at the local address shown (http://localhost:5173 by default).

5. **Try the basic echo demo:**
   - Enter text, click 'Echo via WASM'.
   - The string is sent from TypeScript to Rust, echoed back (via WASM).

6. **Run unit tests:**
   - Rust (core logic):
     ```bash
     cd ../wasm
     cargo test
     ```
   - TypeScript (frontend):
     ```bash
     cd ../frontend
     npm run test
     ```

## Extending This Project
- Replace the echo function in Rust with actual data processing logic
- Use the echo pathway as a template for passing user input from TS to Rust/WASM
- Add map display and interactivity in the frontend
- Write more tests in both languages as you add functionality

---

**Troubleshooting:**
- If WASM import fails in the browser, ensure you ran `./build_wasm.sh` and the pkg is in `frontend/src/wasm/wasm_pkg`.
- If Vite gives compatibility complaints, check Node.js version and clean `node_modules`.
