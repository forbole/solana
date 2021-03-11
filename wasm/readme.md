# BD Solana WASM

## Intro

The signer part of solana sdk for WASM.

## Development

1. Run `wasm-pack build --target web` to generate `pkg` folder including wasm file and js file.
2. Replace `solana_wasm.js` `import * as __wbg_star0 from 'env';` with `const __wbg_star0 = { now: () => {} }` if the situation exists. ( There is a non-surported function in ed25519-dalek pkg )
3. Go to pkg folder and build index.html to import and init wasm.
4. Run `npm run serve` to start the test server on localhost:5000.

## Quick start

Initialize project

```
npm init wasm-app
npm install bd-solana-wasm
```

Import package
```javascript
index.js

import * as wasm from "bd-solana-wasm"

console.log(wasm.generateKey())
```

Run start
```
npm run start
```