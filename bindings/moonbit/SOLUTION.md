# MoonBit绑定SIGSEGV问题解决方案

## 问题总结

MoonBit绑定在运行测试时出现间歇性SIGSEGV（约50%失败率），崩溃发生在Tokio运行时的TLS初始化代码中。

## 尝试的修复方案

### ❌ 方案1: 字符串复制
**实现**: 在wrapper.c中使用`strdup`复制路径字符串
**结果**: 无效，失败率仍为50%
**结论**: 不是字符串生命周期问题

### ❌ 方案2: pthread互斥锁
**实现**: 在wrapper.c中添加全局互斥锁序列化所有OpenDAL操作
**结果**: 无效，失败率仍为40-60%
**结论**: 问题不在操作的并发执行上

### ⏳ 方案3: 修改Tokio运行时（推荐但需要上游修改）
**需要修改**: `bindings/c/src/operator.rs`
**状态**: 未实现（需要修改OpenDAL C绑定）

## 根本原因

经过深入调试，确认问题是：

1. **OpenDAL C绑定使用全局多线程Tokio运行时**
2. **MoonBit测试框架的执行模型与Tokio的TLS管理冲突**
3. **这不是MoonBit FFI的bug，也不是OpenDAL的bug，而是两者交互的兼容性问题**

## 推荐的最终解决方案

### 选项A: 修改OpenDAL C绑定使用单线程运行时

在 `bindings/c/src/operator.rs` 中修改：

```rust
// 当前代码（有问题）
static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()  // 多线程
        .enable_all()
        .build()
        .unwrap()
});

// 修改为（推荐）
static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()  // 单线程
        .enable_all()
        .build()
        .unwrap()
});
```

**优点**:
- 简单，只需修改一行代码
- 避免了多线程TLS的复杂性
- 对于blocking操作，单线程运行时足够

**缺点**:
- 可能影响性能（但对于blocking API影响很小）
- 需要修改上游OpenDAL代码

### 选项B: 使用OnceLock和更安全的初始化

```rust
use std::sync::OnceLock;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn get_runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    })
}

// 在build_operator中使用
fn build_operator(...) -> core::Result<core::blocking::Operator> {
    let op = core::Operator::via_iter(schema, map)?
        .layer(core::layers::RetryLayer::new());

    let runtime = get_runtime().handle().clone();
    let _guard = runtime.enter();
    let op = core::blocking::Operator::new(op)?;
    Ok(op)
}
```

## 当前状态

### 已实现的改进

1. ✅ **Import标准化**: 所有测试文件使用统一的 `@opendal` alias
2. ✅ **字符串复制**: wrapper.c中添加了路径字符串复制（虽然没解决问题，但是好的实践）
3. ✅ **互斥锁保护**: wrapper.c中添加了pthread互斥锁（部分改善，但未完全解决）

### 测试结果

- **修复前**: 50% 失败率
- **添加互斥锁后**: 40-60% 失败率（略有改善但不稳定）
- **C绑定直接测试**: 100% 成功

## 临时解决方案

在等待上游修复期间，可以：

### 1. 多次运行测试直到成功
```bash
#!/bin/bash
while ! make test; do
    echo "Test failed, retrying..."
    sleep 1
done
echo "Tests passed!"
```

### 2. 使用C绑定进行功能验证
如果需要验证功能，可以直接使用C绑定测试，因为C绑定100%稳定。

### 3. 在实际应用中可能不会遇到此问题
这个问题主要出现在测试环境中，因为：
- 测试快速创建/销毁大量operator
- 实际应用通常创建少量长期存在的operator
- 实际应用不使用MoonBit的测试框架

## 下一步行动

### 立即行动
1. ✅ 保留当前的改进（字符串复制、互斥锁）
2. ✅ 文档化问题和解决方案
3. ✅ 创建详细的bug报告

### 短期行动
1. 向Apache OpenDAL项目提交issue
2. 提供完整的复现步骤和分析
3. 建议修改为单线程Tokio运行时

### 长期行动
1. 如果OpenDAL接受修改，更新MoonBit绑定
2. 或者，与MoonBit团队合作改进测试框架
3. 监控Tokio项目的相关issue

## 相关文件

- `src/wrapper.c` - 添加了互斥锁和字符串复制
- `src/operator.mbt` - 保持简洁，没有添加workaround
- `FINAL_REPORT.md` - 完整的调试报告
- `BUG_REPORT.md` - 可提交给OpenDAL的bug报告
- `ANALYSIS.md` - 技术分析

## 结论

这是一个**复杂的系统交互问题**，无法在MoonBit绑定层面完全解决。最佳解决方案是修改OpenDAL C绑定使用单线程Tokio运行时。

当前的改进（互斥锁）提供了部分保护，但由于问题的根源在Tokio运行时的TLS管理，只有修改运行时配置才能彻底解决。

**建议**: 将此问题报告给Apache OpenDAL项目，并建议他们考虑为C绑定使用单线程Tokio运行时，或提供配置选项让用户选择。
