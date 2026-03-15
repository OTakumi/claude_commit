# claude_commit

Rust製のgitコミットメッセージ自動生成CLIツールです。
Claude AIを使用して、ステージング済みの変更から適切なコミットメッセージを自動生成します。

## 機能

- **Git差分の自動取得**: `git diff --cached`でステージング済みの変更を自動取得
- **Claude AIによるメッセージ生成**: Claude CLIを呼び出してコミットメッセージを自動生成
- **インタラクティブな確認フロー**: 生成後に Accept / Edit / Regenerate / Quit を選択可能
- **設定ファイルの自動検索**: `--config` 省略時に標準パスを自動探索
- **カスタマイズ可能なプロンプト**: TOML形式の設定ファイルでプロンプトテンプレートを管理
- **2つの出力モード**:
  - **インタラクティブモード（デフォルト）**: 生成されたメッセージを確認・編集してコミット
  - **JSONモード**: 生成したメッセージをJSON形式で出力（CI/CD連携向け）

## 必要条件

- Rust 2024 Edition
- Git
- [Claude CLI](https://github.com/anthropics/claude-code) がインストールされていること

## インストール

### cargo install（推奨）

```bash
cargo install --git https://github.com/OTakumi/claude_commit.git
```

`~/.cargo/bin/claude_commit` としてインストールされます。

### ソースからビルド

```bash
git clone https://github.com/OTakumi/claude_commit.git
cd claude_commit
cargo build --release
```

ビルド後、`target/release/claude_commit` が実行ファイルとなります。

## 使い方

### セットアップ

1. 設定ファイルを生成する:

   ```bash
   claude_commit init
   ```

   `~/.config/claude_commit/config.toml` が生成されます（ディレクトリも自動作成されます）。
   `prompt` フィールドを編集してコミットメッセージのスタイルをカスタマイズしてください。

2. 変更をステージングする:

   ```bash
   git add <files>
   ```

3. コミットメッセージを生成してコミット:

   ```bash
   claude_commit
   ```

### インタラクティブフロー

メッセージ生成後、以下の選択肢が表示されます:

```
Generated commit message:
─────────────────────────────────────
feat: add user authentication

- Add login/logout endpoints
- Implement JWT token validation
─────────────────────────────────────

[A]ccept  [E]dit  [R]egenerate  [Q]uit >
```

| キー | 動作 |
|------|------|
| `A` | そのままコミット（エディタを開かない） |
| `E` | エディタでメッセージを確認・編集してからコミット |
| `R` | メッセージを破棄して再生成 |
| `Q` | コミットをキャンセル |

### JSONモード

スクリプトやCI/CDで使用する場合:

```bash
claude_commit --json
```

出力例:

```json
{"message":"生成されたコミットメッセージ"}
```

### コマンドラインオプション

```
claude_commit [OPTIONS] [COMMAND]
```

#### コマンド

| コマンド | 説明 |
|---------|------|
| `init` | デフォルトの設定ファイルを生成する |
| `help` | ヘルプを表示する |

#### オプション

| オプション | 説明 |
|-----------|------|
| `--config <PATH>` | TOML形式の設定ファイルパス（省略時は自動検索） |
| `--json` | JSON形式で出力（git commitを実行しない） |

#### init サブコマンドのオプション

| オプション | 説明 |
|-----------|------|
| `--output <PATH>` | 生成先のパス（デフォルト: `.claude_commit.toml`） |
| `--force` | 既存ファイルを上書き |

## 設定ファイル

設定ファイルはTOML形式で記述します。`claude_commit init` で雛形を生成できます。

```toml
# .claude_commit.toml
prompt = """
以下のgit diffの内容を分析して、詳細なコミットメッセージを日本語で生成してください。
1行目にサマリー、その後に変更の詳細を箇条書きで含めてください。
メッセージのみを出力し、説明や追加のテキストは含めないでください。
"""

# オプション: プロンプトテンプレートとgit diffの合計サイズ制限（バイト単位）
# デフォルト: 1,000,000バイト（1MB）
# max_prompt_size = 1000000
```

### 設定ファイルの自動検索

`--config` を省略した場合、以下の順番でファイルを探索します:

1. `~/.config/claude_commit/config.toml`（ユーザー共通設定）← **推奨**
2. `<git root>/.claude_commit.toml`（リポジトリルート）
3. `./.claude_commit.toml`（カレントディレクトリ）

設定は開発者個人が管理するものなので、`claude_commit init` で生成される `~/.config/claude_commit/config.toml` に置くことを推奨します。

### プロンプトのカスタマイズ例

英語でコミットメッセージを生成する場合:

```toml
prompt = """
Analyze the following git diff and generate a commit message in English.
Include a summary on the first line, followed by bullet points describing the changes.
Output only the message without any explanations.
"""
```

Conventional Commits形式:

```toml
prompt = """
Analyze the following git diff and generate a commit message following Conventional Commits format.
Use types like feat, fix, docs, style, refactor, test, chore.
Example: feat(auth): add login functionality
Output only the message without any explanations.
"""
```

## 制限事項

- **入力サイズ制限**: プロンプトテンプレートとgit diffの合計サイズがデフォルトで1MB（1,000,000バイト）に制限されています
  - この制限を超える場合、エラーメッセージが表示されます
  - 設定ファイルで `max_prompt_size` を指定することで上限を変更できます
  - 大規模な変更を一度にコミットする場合は、複数の小さなコミットに分割することを推奨します

## 処理フロー

```
1. CLI引数をパース
2. 設定ファイルをロード（--config 指定 or 自動検索）
3. git diff --cached でステージング済み変更を取得
4. pre-commitフックを実行（存在する場合）
5. git diff --cached を再取得（フォーマッタによる自動修正を反映）
6. 出力モードに応じて処理
   - JSONモード: Claude AIでメッセージ生成 → JSON形式で標準出力
   - インタラクティブモード: スピナー表示しながらメッセージ生成
     → [A]ccept / [E]dit / [R]egenerate / [Q]uit を選択
     → Accept: git commit -F で直接コミット
     → Edit: git commit -v -e -F でエディタを起動してコミット
```

## 依存クレート

| クレート | 用途 |
|---------|------|
| `clap` | CLIアーギュメントの解析 |
| `serde` | シリアライズ/デシリアライズ |
| `serde_json` | JSON形式の出力 |
| `toml` | TOML設定ファイルの解析 |
| `anyhow` | エラーハンドリング |
| `tokio` | 非同期ランタイム（スピナー表示・Claude CLI呼び出し） |
