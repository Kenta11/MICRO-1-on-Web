# MICRO-1 on Web

ブラウザ上で動く [MICRO-1](https://github.com/Kenta11/micro1) の
シミュレータ &amp; アセンブラです。シミュレータもアセンブラも
**上流 (Kenta11/micro1, Kenta11/rm1asm, Kenta11/rm1masm) の C / Rust
ソースを WebAssembly にビルドしたものをそのまま動かす** ので、
出力は教科書実装とビット単位で一致します。UI は素の HTML / CSS / JS
で書かれていて Pages にそのまま置けます。

馬場敬信『マイクロプログラミング』(昭晃堂, 1985) で扱われる MICRO-1 を
教材として WEB ブラウザだけで触れるようにすることが目的です。

## 構成

ページは 3 つのタブに分かれています。

### Simulator タブ

すべての実行はここで起こります。CPU は 1 つだけで、Macro asm の出力は
MM に、Micro asm の出力は CM に、それぞれロードされます。
[Kenta11/micro_alpha_veryl](https://github.com/Kenta11/micro_alpha_veryl)
に同梱されている **教科書相当のマイクロプログラム MICROONE**
(`samples/microone.cm`) が起動時に CM へ自動ロードされるので、
追加で何も書かなくても macro 命令が `m1sim.c` の上で正しく解釈実行されます。

- **表示**: R0–R7, CMAR, CMDR (40-bit), PC, IR, MAR, C, FSR,
  バス LBUS / RBUS / ABUS / SBUS / IOBUS, フラグ ZER / NEG / CRY /
  OV / CZ / T。CM (4K × 40-bit) は逆アセンブル付きで CMAR を追従。
  MM (64K × 16-bit) も 1 word/行 + 逆アセンブルで PC を追従。
- **操作**: Microstep / Macrostep (CMAR が 0 に戻るまで = 次の FETCH
  まで進める) / Run / Stop / Reset / Load MICROONE。
- **編集**: レジスタ・制御レジスタ・フラグ・MM・CM のすべてのセル
  がクリックで in-place 編集できます (`m1sim.c` の `change` コマンド相当)。
- **ブレークポイント**: CM (CMAR) と MM (PC) それぞれに任意個。
  メモリビューでアドレスを選んで Toggle BP、または BP マーカ `●`
  で識別。Run / Macrostep が当たると停止。
- **I/O**: 入力 (CR) と出力 (LPT)。

### Macro asm タブ — `rm1asm` (Rust → WASM)

`.asm` ソースを書いて Assemble すると `MM <title>` / `addr  word`
形式の `.b` テキストを生成。Load into MM で Simulator の MM に
展開し、Entry PC で実行開始アドレスを指定できます (`ORG X"100`
で配置したプログラムも問題なく走ります)。

サンプル: `hello.asm`, `echo.asm`, `sum.asm`, `subroutine.asm`,
`group1.asm`。

### Micro asm タブ — `rm1masm` (Rust → WASM)

`.mas` ソースを書いて Assemble すると `CM <title>` / `addr  ctrl_word`
形式の `.cm` テキストを生成。Load into CM で Simulator の CM に
展開し、Entry CMAR で実行開始アドレスを指定できます。
`* LABEL: ADDR` の命令ヘッダ、`GOTO / CALL / RETURN /
IF FLAG = 0|1 THEN label [ELSE FETCH] / IRA / IAB / IOP`,
`READ / WRITE`, `R0 := R1 + R2 [: shift]`, `SET BY ...`,
`SET HLT / WITH ONE / IR := LBUS / C := RBUS / ...` 等を認識。

サンプル: `load.mas`, `count.mas`, `mem.mas` (2-microstep の MM 書き/読み),
`flag.mas`。

## 既知の上流挙動

- `rm1masm` の `Eol` 正規表現は `(;[^\r]*\r)?\n` でコメント付き行末に
  `\r\n` を要求します。JS ラッパで入力を CRLF に正規化してから呼んでいるので
  ユーザは気にしなくて OK。
- `rm1asm` は **算術系 `disp(ra)` の disp 位置にラベル参照を許容しません**
  (上流仕様)。サンプル `sum.asm` は数値オフセットで書いてあります。
- `rm1asm` は **u16 を超える整数リテラル** (例: `O"314232` = 0x19C9A) を
  エラーにします。C 版 `asm.c` は黙って下位 8 bit に切る挙動ですが、
  本ページは上流 Rust 実装のままです。

## ローカルで動かす

WASM 成果物 (`js/wasm/`) と教科書マイクロプログラム
(`samples/microone.cm`) はリポジトリにコミット済みなので、ビルド不要で
起動できます。

```sh
python3 -m http.server 8000
# → http://localhost:8000
```

`file://` で `index.html` を直接開いても動きます (外部リソース無し)。
Google Fonts の Noto Sans JP だけは CDN から読みますが、ロード失敗時は
システムの日本語フォントにフォールバックします。

## WASM をビルドし直す

```sh
# Emscripten
git clone https://github.com/emscripten-core/emsdk
emsdk/emsdk install latest
emsdk/emsdk activate latest
source emsdk/emsdk_env.sh

# Rust + wasm-pack
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# 3 つまとめてビルド
wasm_src/build.sh
# → js/wasm/ に m1sim.{js,wasm} と rm1asm/, rm1masm/ が再生成される
```

## 公開 (GitHub Pages)

`main` ブランチへ push すると `.github/workflows/pages.yml` が
emsdk + wasm-pack のセットアップ → WASM ビルド → Pages デプロイ
までやります。リポジトリ Settings → Pages の "Build and deployment
source" を **GitHub Actions** に設定してください。

## ファイル構成

```
index.html                    エントリ (Simulator / Macro asm / Micro asm タブ)
css/style.css                 スタイル + モバイル対応
js/
  assembler.js                rm1asm wasm のラッパ (window.MICRO1Assembler)
  microasm.js                 rm1masm wasm のラッパ (window.MICRO1Microasm)
  microsim.js                 m1sim wasm のラッパ (window.MICRO1Microsim)
  app.js                      UI / サンプル / 編集 / BP
  wasm/                       ビルド済み WASM 成果物
    m1sim.js, m1sim.wasm          m1sim.c (C) → Emscripten
    rm1asm/                       rm1asm (Rust) → wasm-pack
    rm1masm/                      rm1masm (Rust) → wasm-pack
samples/
  microone.cm                 教科書マイクロプログラム (micro_alpha_veryl から抽出)
wasm_src/
  m1sim_core.c                m1sim.c の WASM 用パッチ版 (I/O のみ書き換え)
  m1sim_api.c                 Emscripten 用 API (reset/load/step/set_*/get_state)
  rm1asm_wasm/                wasm-bindgen 用クレート (rm1asm vendored)
  rm1masm_wasm/               wasm-bindgen 用クレート (rm1masm vendored)
  build.sh                    3 つまとめてビルド
.github/workflows/pages.yml   CI: WASM ビルド + Pages デプロイ
```

## 参考

- [Kenta11/micro1](https://github.com/Kenta11/micro1) — C 版 (`m1sim.c`,
  `asm.c`, `masm.c`)
- [Kenta11/rm1asm](https://github.com/Kenta11/rm1asm) — Rust マクロ
  アセンブラ
- [Kenta11/rm1masm](https://github.com/Kenta11/rm1masm) — Rust マイクロ
  アセンブラ
- [Kenta11/micro-alpha](https://github.com/Kenta11/micro-alpha) /
  [Kenta11/micro_alpha_veryl](https://github.com/Kenta11/micro_alpha_veryl)
  — MICRO-alpha (MICRO-1 互換 FPGA 実装) と MICROONE マイクロプログラム
- [Kenta11/micro1-as](https://github.com/Kenta11/micro1-as) — C++17 版
  マクロアセンブラ (テストフィクスチャのみ参照)
- 馬場敬信『マイクロプログラミング』昭晃堂, 1985

## License

MIT。ベンダリングした上流ソース (`wasm_src/` 配下) もすべて MIT。
