# React SSR on yavashark

Renders a small React app to HTML with the real `react-dom/server`, running
entirely inside yavashark. The engine strips the TypeScript and compiles the JSX
in `app.tsx` itself, so the app code needs no build step.

```sh
cargo run --release --bin yavashark -- demos/react-ssr/app.tsx > out.html
```

`vendor/react.js` contains React 19.3.1 and `react-dom/server`, prebuilt into a
single file (MIT, see `vendor/LICENSE`). yavashark only resolves relative
imports, so it can't load React from `node_modules`.
