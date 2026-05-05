# Archive Extractor 🗜️

一个轻量的 Rust 解压工具，支持 **ZIP / 7z / RAR** 三种常见压缩格式的批量解压。

## 功能特性

- 🎯 **三种输入方式**：单文件、单目录、路径列表文件（可混合文件与目录）
- 🔍 自动识别压缩格式（基于文件签名，而非扩展名）
- 📂 **目录递归扫描**：可指定扫描深度
- 📋 **路径列表文件**：支持注释行（`#` 开头），每行一个路径
- 📦 **自定义输出目录**：将所有解压内容集中到指定目录
- 🛡️ 安全的 ZIP 解压（自动跳过路径遍历条目）
- 🔑 **密码解压**：支持加密的 ZIP / 7z / RAR 压缩包

## 支持格式

| 格式 | 后端                |
| ---- | ------------------- |
| ZIP  | `zip` crate         |
| 7z   | `sevenz-rust` crate |
| RAR  | `unrar` crate       |

## 安装

> **环境要求**：Rust 工具链 **1.85.0** 或更高版本。可通过 `rustup update` 升级到最新稳定版。
> 运行 `rustc --version` 可查看当前版本。

### 方式一：全局安装（推荐）

安装后可在任意目录下直接使用 `ae` 命令：

```bash
git clone <repo-url>
cd archive_extractor
cargo install --path .
```

### 方式二：本地编译

```bash
git clone <repo-url>
cd archive_extractor
cargo build --release
```

编译产物位于 `target/release/ae.exe`，建议将其所在目录加入 `PATH`。

## 快速运行（开发 / 调试）

如果你只是想快速测试，不需要安装到系统：

```bash
git clone <repo-url>
cd archive_extractor
cargo run -- <参数>
```

> **注意**：`cargo run` 为 debug 模式，性能较低，且必须在项目目录下执行。

## 使用方法

### 1️⃣ 解压单个文件

```bash
ae game.rar
```

解压到 `game/` 目录（与压缩包同级）。

### 2️⃣ 扫描目录

```bash
# 仅扫描当前目录（默认 depth=1）
ae ./downloads/

# 递归扫描子目录，最大深度 3
ae -d 3 ./downloads/
```

工具会自动跳过非压缩文件。

### 3️⃣ 从列表文件批量解压

```bash
ae -l archive_list.txt
```

列表文件格式示例（`archive_list.txt`）：

```text
# 游戏相关（注释行会自动跳过）
game.rar
./mods/
savegame.zip
./backups/
```

每行一个路径，可以是压缩文件或目录，工具自动识别处理。

### 4️⃣ 指定输出目录

```bash
# 单个文件指定输出目录
ae game.zip -o ./extracted/

# 列表文件 + 输出目录
ae -l archive_list.txt -o ./extracted/
```

使用 `-o` 时，解压结构为 `<输出目录>/<压缩文件名不带后缀>/`。

### 5️⃣ 平铺模式（不建子目录）

```bash
# 跳过 <文件名>/ 子目录，直接解压到指定目录
ae game.zip --flat -o ./extracted/

# 解压到压缩包所在目录（不建子目录）
ae game.zip --flat

# 列表文件 + 平铺模式
ae -l archive_list.txt --flat -o ./out/
```

默认行为会在输出路径中增加一层 `<压缩文件名不带后缀>/` 子目录。使用 `--flat`（或 `-f`）可跳过这层目录，将压缩包内容直接展开到目标根目录。

> **注意**：`--flat` 模式下多个压缩包解压到同目录时，同名文件会互相覆盖。ZIP 格式会打印冲突警告，7z/RAR 格式由外部库管理（静默覆盖）。

## CLI 参数

```
Usage: ae [OPTIONS] <PATH>

Arguments:
  <PATH>  压缩文件路径 / 目录路径 / 列表文件路径（当使用 --list 时）

Options:
  -l, --list             将 PATH 视为路径列表文件（每行一个路径）
  -d, --depth <DEPTH>    目录扫描深度 [default: 1]，只对目录生效
  -o, --output <OUTPUT>  可选输出根目录
  -f, --flat             平铺模式：跳过 <文件名>/ 子目录，直接解压到输出根目录
  -p, --password <PASSWORD>  解压密码（压缩包加密时使用）
  -h, --help                  Print help
  -V, --version               Print version
```

## 示例合集

```bash
# 1. 解压单个文件
ae game.rar

# 2. 扫描目录（默认 depth=1）
ae ./downloads/

# 3. 递归扫描子目录
ae --depth 3 ./downloads/

# 4. 列表文件中混合文件+目录
ae --list archive_list.txt

# 5. 列表文件 + 输出目录
ae --list archive_list.txt --output ./extracted/

# 6. 目录扫描 + 输出目录
ae --depth 2 ./mods/ --output ./extracted/

# 7. 解压加密的压缩包
ae protected.rar --password mypassword

# 8. 扫描目录时统一使用密码
ae --depth 3 ./downloads/ --password mypassword

# 9. 列表文件批量解压 + 密码
ae --list archive_list.txt --password mypassword --output ./extracted/

# 10. 平铺模式：直接解压到输出目录（不建子目录）
ae game.zip --flat -o ./extracted/

# 11. 平铺模式：解压到压缩包所在目录
ae game.zip --flat
```

## 项目结构

```
archive_extractor/
├── Cargo.toml                     # 项目元信息与依赖声明
├── Cargo.lock                     # 依赖版本锁定（精确复现构建）
├── .gitignore                     # Git 忽略规则
├── src/
│   ├── main.rs                    # CLI 入口：参数解析、输入调度、目录扫描
│   ├── lib.rs                     # 库入口，公开 API 导出（extract / extract_to）
│   ├── archive_helper.rs          # 高层解压接口：文件读取 + 委托分发
│   ├── path_utils.rs              # 路径工具（default_dest、ensure_parent_dir）
│   └── formats/
│       ├── mod.rs                 # 格式分发器（infer 检测 + 路由）
│       ├── zip.rs                 # ZIP 解压实现
│       ├── sevenz.rs              # 7z 解压实现
│       └── rar.rs                 # RAR 解压实现
└── tests/
    ├── common/
    │   └── mod.rs                 # 共享测试辅助函数
    ├── path_utils_tests.rs        # 路径工具单元测试
    ├── format_detection_tests.rs  # 格式检测集成测试
    └── zip_tests.rs               # ZIP 解压集成测试（含密码功能）
```

## 依赖

| Crate         | 用途         |
| ------------- | ------------ |
| `clap`        | CLI 参数解析 |
| `infer`       | 文件格式检测 |
| `zip`         | ZIP 解压     |
| `sevenz-rust` | 7z 解压      |
| `unrar`       | RAR 解压     |
