3D viewer
=====


Rust で作成する 3D グラフィックス  


実行方法
---

test ディレクトリ配下のファイルは実行できます。  
依存するファイル(3Dモデルなど)のパスはtest配下のソースに直接記載されているで、
書き換えればそのファイルを読み込みに行くように変えられます。  

```sh
# Rust 内で Ruby インタプリタを実行する
cargo run --example ruby

# Tensorflow の推論モデルを実行する
cargo run --example tensorflow

# Window を描画し中に 3D グラフィックを描画する
cargo run --example graphics

# WebAssembly で作成されたファイルを読み込み実行する（要 Lucet）
cargo run --example plugin

# .blend ファイルを読み込む（読み込むだけ）
cargo run --example blender

# .blend ファイルを読み込み画面に描画する（blender + graphics の example）
cargo run --example model
```


事前準備
---

- リポジトリのクローン
```sh
# このリポジトリをクローンした後、以下のコマンドで submodule も持ってくる
git submodule update --init --recursive
```

- Lucet
事前に Lucet コンパイラを準備  
[Lucet](https://github.com/bytecodealliance/lucet)  

