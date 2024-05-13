# uLLVM

[Demo](https://tan.ge/ullvm)

Complete rewrite of [my bachelor's project](https://tan.ge/portfolio/llvm--2/)
in Rust! 🦀

Targeting the web with Dioxus for visualizing register allocation of any given
LLVM module as opposed to a (very) limited subset of LLVM referred to as LLVM--.

The LLVM parser is compiled to WASM from [a stripped down LLVM 17.0.6.](https://github.com/s1gtrap/llvm-project/tree/lean-17)

## Development

Run the following command in the root of the project to start the Dioxus dev server:

```bash
dx serve --hot-reload
```

and

```bash
npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch
```

- Open the browser to http://localhost:8080
