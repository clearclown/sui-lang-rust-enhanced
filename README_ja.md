# 粋 (Sui) - LLMのためのプログラミング言語

[![Crates.io](https://img.shields.io/crates/v/sui-lang.svg)](https://crates.io/crates/sui-lang)
[![Documentation](https://docs.rs/sui-lang/badge.svg)](https://docs.rs/sui-lang)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/clearclown/sui-lang-rust-enhanced/actions/workflows/ci.yml/badge.svg)](https://github.com/clearclown/sui-lang-rust-enhanced/actions)

> LLMが最も正確にコードを生成できるように設計された行ベース言語

[English README](README.md) | [オンラインプレイグラウンド](https://clearclown.github.io/sui-lang-rust-enhanced/)

## 概要

**粋 (Sui)** は「洗練」「無駄を削ぎ落とす」という日本語の美意識から名付けられた、LLM（大規模言語モデル）が正確にコードを生成することを最優先に設計されたプログラミング言語である。

**本リポジトリはRust実装版**であり、高性能インタプリタ、REPL、複数のトランスパイラ（JavaScript、WebAssembly）を提供する。

## 特徴

### コア機能
- LLM生成に最適化された行ベース構文
- 高性能Rustインタプリタ
- 複数のトランスパイラ（Python、JavaScript）
- 対話型REPLモード
- ブラウザ実行用WebAssemblyサポート
- ステップデバッガ
- モジュール/インポートシステム

### 設計原則

1. **行単位独立性** - 各行が完全に自己完結
2. **括弧問題の最小化** - ネストは関数ブロック `{}` のみ
3. **1文字命令** - トークン効率最大化
4. **連番変数** - 意味のある名前は不要（v0, v1, g0, a0）
5. **明示的制御フロー** - ラベルとジャンプ

## インストール

### crates.io から（推奨）

```bash
cargo install sui-lang
```

### ソースから

```bash
git clone https://github.com/clearclown/sui-lang-rust-enhanced.git
cd sui-lang-rust-enhanced
cargo install --path .
```

### 追加機能付き

```bash
# フル機能（REPL、カラー出力）
cargo install sui-lang --features full

# 最小インストール
cargo install sui-lang --no-default-features
```

### レガシーPython版

```bash
# PyPI
pip install sui-lang

# Homebrew (macOS/Linux)
brew tap TakatoHonda/sui
brew install sui-lang
```

## クイックスタート

### インタプリタ

```bash
# サンプル実行（デモを表示）
sui

# ファイル実行
sui examples/fibonacci.sui

# 引数付き実行
sui examples/fib_args.sui 15

# バリデーション
sui --validate examples/fibonacci.sui

# REPLモード
sui --repl
```

### トランスパイラ（Sui → Python）

```bash
# 変換結果を表示
sui2py examples/fibonacci.sui

# ファイルに出力
sui2py examples/fibonacci.sui -o fib.py

# 変換して即実行
sui2py examples/fibonacci.sui --run
```

### トランスパイラ（Sui → JavaScript）

```bash
# 変換結果を表示
sui2js examples/fibonacci.sui

# ファイルに出力
sui2js examples/fibonacci.sui -o fib.js

# 変換してNode.jsで実行
sui2js examples/fibonacci.sui --run

# ブラウザ互換コード生成
sui2js examples/fibonacci.sui --browser
```

### トランスパイラ（Python → Sui）

```bash
# PythonをSuiに変換
py2sui script.py

# ファイルに出力
py2sui script.py -o script.sui

# 変換例
echo 'x = 10
y = x + 5
print(y)' | py2sui

# 出力:
# = v0 10
# + v1 v0 5
# . v1
```

### REPLモード

```bash
sui --repl

# REPL内コマンド:
# :help  - ヘルプ表示
# :reset - インタプリタ状態リセット
# :vars  - 変数表示
# :quit  - 終了
```

### デバッガ

```bash
# デバッガ起動
sui-debug examples/fibonacci.sui

# 初期ブレークポイント付き
sui-debug examples/fibonacci.sui -b 5,10

# デバッガコマンド:
# step, s        - 1命令実行
# continue, c    - ブレークポイントまで続行
# break N, b N   - N行目にブレークポイント設定
# delete N, d N  - ブレークポイント削除
# list, l        - 現在行周辺のソース表示
# locals         - ローカル変数表示
# globals        - グローバル変数表示
# print E, p E   - 式の検査
# backtrace, bt  - コールスタック表示
# quit, q        - デバッガ終了
```

## 構文

### 命令一覧

| 命令 | 形式 | 説明 |
|------|------|------|
| `=` | `= var value` | 代入 |
| `+` | `+ result a b` | 加算 |
| `-` | `- result a b` | 減算 |
| `*` | `* result a b` | 乗算 |
| `/` | `/ result a b` | 除算 |
| `%` | `% result a b` | 剰余 |
| `<` | `< result a b` | 小なり（結果0/1） |
| `>` | `> result a b` | 大なり（結果0/1） |
| `~` | `~ result a b` | 等価（結果0/1） |
| `!` | `! result a` | NOT |
| `&` | `& result a b` | AND |
| `\|` | `\| result a b` | OR |
| `?` | `? cond label` | 条件ジャンプ |
| `@` | `@ label` | 無条件ジャンプ |
| `:` | `: label` | ラベル定義 |
| `#` | `# id argc {` | 関数定義開始 |
| `}` | `}` | 関数定義終了 |
| `$` | `$ result func args...` | 関数呼び出し |
| `^` | `^ value` | return |
| `[` | `[ var size` | 配列作成 |
| `]` | `] result arr idx` | 配列読取 |
| `{` | `{ arr idx value` | 配列書込 |
| `.` | `. value` | 出力 |
| `,` | `, var` | 入力 |
| `R`/`P` | `R result "func" args...` | FFI呼び出し |
| `_` | `_ "path/to/module.sui"` | モジュールインポート |

### 変数

| 形式 | 意味 |
|------|------|
| `v0`, `v1`, ... | ローカル変数 |
| `g0`, `g1`, ... | グローバル変数 |
| `a0`, `a1`, ... | 関数引数 |
| `g100` | argc（コマンドライン引数の数） |
| `g101`, `g102`, ... | argv（コマンドライン引数） |

## 例

### フィボナッチ

```sui
# 0 1 {
< v0 a0 2
! v1 v0
? v1 1
^ a0
: 1
- v2 a0 1
$ v3 0 v2
- v4 a0 2
$ v5 0 v4
+ v6 v3 v5
^ v6
}
= g0 10
$ g1 0 g0
. g1
```

**出力**: `55`

### FizzBuzz

```sui
= v0 1
: 0
> v1 v0 100
? v1 9
% v2 v0 15
~ v3 v2 0
? v3 1
% v4 v0 3
~ v5 v4 0
? v5 2
% v6 v0 5
~ v7 v6 0
? v7 3
. v0
@ 4
: 1
. "FizzBuzz"
@ 4
: 2
. "Fizz"
@ 4
: 3
. "Buzz"
@ 4
: 4
+ v0 v0 1
@ 0
: 9
```

### モジュールとインポート

別ファイルに関数を定義して再利用可能なモジュールを作成：

**modules/math.sui:**
```sui
; 数学ユーティリティ関数

; 関数100: double(x) - x * 2 を返す
# 100 1 {
* v0 a0 2
^ v0
}

; 関数101: square(x) - x * x を返す
# 101 1 {
* v0 a0 a0
^ v0
}
```

**main.sui:**
```sui
; 数学モジュールをインポート
_ "modules/math.sui"

; インポートした関数を使用
= v0 5
$ v1 100 v0
. v1
; 出力: 10

$ v2 101 v0
. v2
; 出力: 25
```

**特徴:**
- インポート元ファイルからの相対パス解決
- ネストしたインポート対応（モジュールが他のモジュールをインポート可能）
- 循環インポートの自動検出
- 効率化のためのモジュールキャッシング

## ライブラリとしての使用（Rust）

```rust
use sui_lang::Interpreter;

fn main() {
    let code = r#"
= v0 10
+ v1 v0 5
. v1
"#;

    let mut interpreter = Interpreter::new();
    let output = interpreter.run(code, &[]).unwrap();
    println!("出力: {:?}", output);  // ["15"]
}
```

## FFI（外部関数インターフェース）

`R`（または`P`）コマンドを使用して組み込み関数を呼び出し：

```sui
; 数学関数
R v0 "math.sqrt" 16        ; v0 = 4.0
R v1 "pow" 2 10            ; v1 = 1024.0
R v2 "sin" 0               ; v2 = 0.0
R v3 "cos" 0               ; v3 = 1.0

; 文字列/配列関数
R v4 "len" "hello"         ; v4 = 5
R v5 "abs" -42             ; v5 = 42
R v6 "max" 10 20 5 30      ; v6 = 30
R v7 "min" 10 20 5 30      ; v7 = 5

; 型変換
R v8 "int" "123"           ; v8 = 123
R v9 "float" "3.14"        ; v9 = 3.14
R v10 "str" 42             ; v10 = "42"
R v11 "round" 3.14159 2    ; v11 = 3.14

; 乱数
R v12 "random.randint" 1 100  ; v12 = 1-100のランダム値
```

## WebAssemblyサポート

ネイティブに近いパフォーマンスでブラウザ実行するためにWebAssemblyにコンパイル：

```bash
# WASMモジュールをビルド
wasm-pack build --target web
```

```html
<script type="module">
import init, { run_sui } from './pkg/sui_lang.js';

await init();
const output = run_sui(`
= v0 10
+ v1 v0 5
. v1
`);
console.log(output); // ["15"]
</script>
```

[Rust + WebAssembly](https://rustwasm.github.io/book/) の利点：
- **小さなバイナリサイズ**: ~50KB（Goの2MB+最小に対して）
- **ランタイムオーバーヘッドなし**: WASMへの直接コンパイル
- **メモリ安全性**: Rustの保証がWASMに引き継がれる

## ロードマップ

### 完了
- [x] 高性能Rustインタプリタ
- [x] トランスパイラ（Sui → Python）
- [x] トランスパイラ（Sui → JavaScript）
- [x] トランスパイラ（Python → Sui）
- [x] 対話型REPLモード
- [x] WebAssemblyバインディング
- [x] FFIサポート（組み込み関数）
- [x] 包括的テストスイート（118+テスト）

### 進行中
- [ ] オンラインプレイグラウンド（WASM版） - GitHub Pagesにデプロイ

### 最近追加
- [x] VS Code拡張（シンタックスハイライトとスニペット）
- [x] [LSP（Language Server Protocol）](https://microsoft.github.io/language-server-protocol/) - [tower-lsp](https://github.com/ebkalderon/tower-lsp)使用
- [x] ブレークポイント付きステップデバッガ
- [x] コード再利用のためのモジュール/インポートシステム

### 予定
- [ ] [LLVM IR](https://mcyoung.xyz/2023/08/01/llvm-ir/) 出力（ネイティブコンパイル用）
- [ ] 型注釈（オプショナル静的型付け）
- [ ] Suiモジュール用パッケージマネージャ
- [ ] Jupyterカーネル統合

## なぜSuiか

### 名前の由来

**粋（すい/いき）** - 日本語で「洗練されている」「無駄がない」という意味。余計なものを削ぎ落とし、本質だけを残す美意識を表す。

### LLMコード生成の問題

現在のLLMは特定のコード生成パターンで苦労する。[ACM TOSEM](https://dl.acm.org/doi/10.1145/3770084)の研究によると、ドメイン固有言語（DSL）は独自の構文とデータ不足により追加の課題がある。

| LLMの弱点 | 従来のコード | Suiの対策 |
|----------|-------------|----------|
| 括弧の不一致 | `if (x) { if (y) { ... } }` | 括弧は関数の `{}` のみ |
| 長距離依存 | 100行以上後で使用される変数 | 各行が独立 |
| 変数名タイポ | `userCount` vs `userCont` | 連番（v0, v1...） |
| 複雑なネスト | ネストされたコールバック/条件 | ラベル付きフラット構造 |
| コンテキストウィンドウ制限 | 大規模コードベースの理解 | 最小トークン使用 |

### パフォーマンス（Rust vs Python）

| ベンチマーク | Python | Rust | 高速化 |
|------------|--------|------|-------|
| Fibonacci(20) | ~50ms | ~1ms | **~50倍** |
| ループ10000 | ~100ms | ~2ms | **~50倍** |
| 配列操作 | ~80ms | ~1ms | **~80倍** |
| WASMバイナリ | N/A | ~50KB | 即時ロード |

## コントリビューション

コントリビューションを歓迎します！Pull Requestをお気軽に送ってください。

### 開発環境セットアップ

```bash
# クローンとビルド
git clone https://github.com/clearclown/sui-lang-rust-enhanced.git
cd sui-lang-rust-enhanced
cargo build --release

# テスト実行
cargo test

# ベンチマーク実行
cargo bench

# フル機能でビルド
cargo build --features full
```

### コントリビューション対象

- **LSP実装** - [tower-lsp](https://github.com/ebkalderon/tower-lsp)を使用したIDE対応の改善
- **LLVMバックエンド** - [llvm-sys](https://crates.io/crates/llvm-sys)経由のネイティブコンパイル
- **ドキュメント** - サンプルとチュートリアルの改善
- **テスト** - エッジケーステストの追加

## ライセンス

MIT License

---

<p align="center">
  <b>粋 (Sui)</b> - AI時代の洗練されたコード生成
</p>
