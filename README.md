# claude_commit

Rust製のgitコミットメッセージ自動生成CLIツールです。　　
Claude AIを使用して、ステージング済みの変更から適切なコミットメッセージを自動生成します。

## 機能

- **Git差分の自動取得**: `git diff --cached`でステージング済みの変更を自動取得
- **Claude AIによるメッセージ生成**: Claude CLIを呼び出してコミットメッセージを自動生成
- **カスタマイズ可能なプロンプト**: TOML形式の設定ファイルでプロンプトテンプレートを管理
- **2つの出力モード**:
  - **エディタモード（デフォルト）**: 生成されたメッセージをエディタで確認・編集してコミット
  - **JSONモード**: 生成したメッセージをJSON形式で出力（CI/CD連携向け）

## 必要条件

- Rust 2024 Edition
- Git
- [Claude CLI](https://github.com/anthropics/claude-code) がインストールされていること

## インストール

```bash
git clone https://github.com/OTakumi/claude_commit.git
cd claude_commit
cargo build --release
```

ビルド後、`target/release/claude_commit`が実行ファイルとなります。

## 使い方

### 基本的な使い方

1. 変更をステージングする:

   ```bash
   git add <files>
   ```

2. コミットメッセージを生成してコミット:

   ```bash
   cargo run -- --config=commit-config.toml
   ```

   エディタが開き、生成されたコミットメッセージを確認・編集できます。

### JSONモード

スクリプトやCI/CDで使用する場合:

```bash
cargo run -- --config=commit-config.toml --json
```

出力例:

```json
{"message":"生成されたコミットメッセージ"}
```

### コマンドラインオプション

| オプション | 説明 |
|-----------|------|
| `--config <PATH>` | **必須**: TOML形式の設定ファイルパス |
| `--json` | JSON形式で出力（git commitを実行しない） |

## 設定ファイル

設定ファイルはTOML形式で記述します。`commit-config.toml.example`を参考にしてください。

```toml
# commit-config.toml
prompt = """
以下のgit diffの内容を分析して、詳細なコミットメッセージを日本語で生成してください。
1行目にサマリー、その後に変更の詳細を箇条書きで含めてください。
メッセージのみを出力し、説明や追加のテキストは含めないでください。
"""
```

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

## 処理フロー

```
1. CLI引数をパース
2. 設定ファイルをロード
3. git diff --cached でステージング済み変更を取得
4. Claude CLIでコミットメッセージを生成
5. 出力モードに応じて処理
   - JSONモード: JSON形式で標準出力
   - エディタモード: .git/COMMIT_MSG_GENERATED に保存 → git commit -v -e -F で確認
```

## 依存クレート

| クレート | 用途 |
|---------|------|
| `clap` | CLIアーギュメントの解析 |
| `serde` | シリアライズ/デシリアライズ |
| `serde_json` | JSON形式の出力 |
| `toml` | TOML設定ファイルの解析 |
| `anyhow` | エラーハンドリング |
