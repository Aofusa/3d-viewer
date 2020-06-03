Hello Sample WebAssembly
=====


Lucet
---


### 準備

事前に Lucet コンパイラを準備しておく  
[Lucet](https://github.com/bytecodealliance/lucet)  


### How to Build

```sh
# 事前に wasm にコンパイルしておく
wasm32-wasi-clang -Ofast -nostartfiles -Wl,--no-entry,--export-all -o hello.wasm hello.c  # ライブラリの作成 基本はこれ
wasm32-wasi-clang -Ofast -o hello.wasm hello.c  # 実行可能なメインファイルの作成 検証で main関数を定義して実行したいならこれ
wasm32-wasi-clang -Ofast -Wl,--no-entry,--export-all -o hello.wasm hello.c  # 実行可能なライブラリファイルの作成 main関数を定義して実行したい場合はこっち

# lucet でビルド
lucetc-wasi -o hello.so hello.wasm

# lucet で実行してみる
lucet-wasi hello.so
```

