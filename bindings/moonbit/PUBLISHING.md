# OpenDAL MoonBit 绑定发布指南

## 概述

本文档说明如何发布 OpenDAL MoonBit 绑定到 mooncakes.io（MoonBit 包注册中心）。

## 前提条件

### 1. 账号注册

```bash
# 注册 mooncakes.io 账号
moon register

# 登录
moon login
```

### 2. 依赖准备

确保 OpenDAL C 库已构建：

```bash
cd ../c
cargo build --release
cd ../moonbit
```

## 发布前检查清单

### 1. 更新版本号

编辑 `moon.mod.json`：

```json
{
  "name": "opendal/lib",
  "version": "0.1.0",  // 更新版本号
  "readme": "README.md",
  "repository": "https://github.com/apache/opendal",
  "license": "Apache-2.0",
  "keywords": [
    "opendal",
    "storage",
    "ffi"
  ],
  "description": "MoonBit bindings for Apache OpenDAL",
  "source": "src",
  "preferred-target": "native",
  "deps": {}
}
```

**版本号规范：**
- 遵循语义化版本 (Semantic Versioning)
- 格式：`MAJOR.MINOR.PATCH`
- 例如：`0.1.0` → `0.1.1` (bug 修复) 或 `0.2.0` (新功能)

### 2. 清理依赖

**重要：** 移除 `moonbitlang/async` 依赖（已移除异步 API）：

```json
{
  "deps": {}  // 确保为空
}
```

### 3. 验证代码

```bash
# 检查代码
make check

# 格式化代码
make format

# 构建库
make build

# 运行测试
make test
```

### 4. 更新文档

确保以下文档是最新的：
- `README.md` - 主要文档
- `CHANGELOG.md` - 变更日志（如果有）
- `ASYNC_REMOVAL_SUMMARY.md` - 异步移除说明

## 发布流程

### 方法 1：使用 moon publish（推荐）

```bash
# 1. 确保在正确的目录
cd /path/to/opendal/bindings/moonbit

# 2. 打包
moon package

# 3. 发布（需要先登录）
moon publish
```

### 方法 2：使用 Makefile

添加发布目标到 Makefile：

```makefile
# Publish to mooncakes.io
publish: check format build test
	@echo "$(COLOR_BLUE)Publishing to mooncakes.io...$(COLOR_RESET)"
	@echo "$(COLOR_YELLOW)Current version: $(shell grep version moon.mod.json | cut -d'"' -f4)$(COLOR_RESET)"
	@read -p "Continue? [y/N] " confirm && [ "$$confirm" = "y" ] || exit 1
	moon publish
	@echo "$(COLOR_GREEN)✓ Published successfully!$(COLOR_RESET)"

# Dry run publish
publish-dry-run:
	@echo "$(COLOR_BLUE)Dry run publish...$(COLOR_RESET)"
	moon publish --dry-run
```

然后运行：

```bash
# 测试发布（不实际发布）
make publish-dry-run

# 正式发布
make publish
```

## 发布注意事项

### 1. C 库依赖

**重要：** MoonBit 包本身不包含 C 库文件。用户需要：

1. 自己构建 OpenDAL C 库
2. 或者从 OpenDAL 发布页面下载预编译的 C 库

在 README 中明确说明：

```markdown
## 安装

### 1. 安装 MoonBit 包

\`\`\`bash
moon add opendal/lib
\`\`\`

### 2. 构建 OpenDAL C 库

\`\`\`bash
# 克隆 OpenDAL 仓库
git clone https://github.com/apache/opendal.git
cd opendal/bindings/c

# 构建 C 库
cargo build --release

# 库文件位置：
# - macOS: target/release/libopendal_c.dylib
# - Linux: target/release/libopendal_c.so
# - Windows: target/release/opendal_c.dll
\`\`\`

### 3. 配置库路径

在你的项目中设置库路径：

\`\`\`bash
# macOS
export DYLD_LIBRARY_PATH=/path/to/opendal/bindings/c/target/release

# Linux
export LD_LIBRARY_PATH=/path/to/opendal/bindings/c/target/release

# Windows
set PATH=C:\\path\\to\\opendal\\bindings\\c\\target\\release;%PATH%
\`\`\`
```

### 2. 平台支持

明确说明支持的平台：

```markdown
## 支持的平台

- ✅ macOS (x86_64, arm64)
- ✅ Linux (x86_64, arm64)
- ✅ Windows (x86_64)
```

### 3. 版本兼容性

说明与 OpenDAL C 库的版本兼容性：

```markdown
## 版本兼容性

| MoonBit 绑定版本 | OpenDAL C 库版本 |
|------------------|------------------|
| 0.1.0            | 0.50.x           |
| 0.2.0            | 0.51.x           |
```

## 发布后验证

### 1. 检查包是否可用

```bash
# 搜索包
moon search opendal

# 查看包信息
moon info opendal/lib
```

### 2. 测试安装

在新项目中测试：

```bash
# 创建测试项目
mkdir test-opendal
cd test-opendal
moon init

# 添加依赖
moon add opendal/lib

# 验证
moon check
```

## 发布检查清单

在发布前，确保完成以下检查：

- [ ] 版本号已更新
- [ ] 移除了 `moonbitlang/async` 依赖
- [ ] 代码通过 `make check`
- [ ] 代码已格式化 `make format`
- [ ] 库构建成功 `make build`
- [ ] 所有测试通过 `make test`
- [ ] README.md 是最新的
- [ ] 文档说明了 C 库依赖
- [ ] 文档说明了平台支持
- [ ] 已登录 mooncakes.io
- [ ] 已测试 `moon publish --dry-run`

## 常见问题

### Q1: 发布失败，提示权限错误

**A:** 确保已登录：

```bash
moon login
```

### Q2: 如何撤回已发布的版本？

**A:** MoonBit 包注册中心通常不允许撤回已发布的版本。如果有问题，发布一个新的修复版本。

### Q3: 如何发布预发布版本？

**A:** 使用预发布版本号：

```json
{
  "version": "0.2.0-alpha.1"
}
```

### Q4: C 库文件应该包含在包中吗？

**A:** 不应该。原因：
- C 库文件很大
- 不同平台需要不同的库文件
- 用户应该自己构建或下载预编译版本

### Q5: 如何处理 C 库的版本依赖？

**A:** 在文档中明确说明：
- 推荐的 OpenDAL C 库版本
- 最低兼容版本
- 已知的不兼容版本

## 自动化发布

### GitHub Actions 示例

创建 `.github/workflows/publish.yml`：

```yaml
name: Publish MoonBit Package

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install MoonBit
        run: |
          # 安装 MoonBit

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build C Library
        run: |
          cd ../c
          cargo build --release

      - name: Check and Test
        run: |
          cd bindings/moonbit
          make check
          make format
          make build
          make test

      - name: Login to mooncakes.io
        run: |
          echo "${{ secrets.MOONCAKES_TOKEN }}" | moon login

      - name: Publish
        run: |
          cd bindings/moonbit
          moon publish
```

## 参考资源

- [MoonBit 官方文档](https://docs.moonbitlang.com/)
- [mooncakes.io](https://mooncakes.io/)
- [OpenDAL 文档](https://opendal.apache.org/)
- [语义化版本规范](https://semver.org/)

## 联系方式

如有问题，请：
- 提交 Issue: https://github.com/apache/opendal/issues
- 加入讨论: https://github.com/apache/opendal/discussions
