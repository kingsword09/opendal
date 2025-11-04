# MoonBit绑定的替代FFI方案

## 当前问题回顾

使用C绑定方式遇到的问题：
- OpenDAL C绑定使用多线程Tokio运行时
- 与MoonBit测试框架的TLS管理冲突
- 导致40-60%的测试失败率

## MoonBit FFI能力分析

根据官方文档，MoonBit支持以下FFI后端：

1. **C** - 生成C文件和可执行文件 ✅ 当前使用
2. **WebAssembly (Wasm)** - 支持bulk-memory-operations等提案 ⭐ 可行
3. **Wasm GC** - 支持GC提案和JS字符串内置函数 ⭐ 可行
4. **JavaScript** - 生成CommonJS、ES模块或IIFE ❌ 不适用
5. **LLVM** - 实验性，**目前不支持FFI** ❌ 不可用

## 方案1: WebAssembly FFI（推荐）⭐⭐⭐⭐⭐

### 概述
将OpenDAL编译为WebAssembly，通过MoonBit的Wasm FFI调用。

### 优势
1. ✅ **避免Tokio多线程问题** - Wasm是单线程的
2. ✅ **跨平台** - 一次编译，到处运行
3. ✅ **内存安全** - Wasm的沙箱环境
4. ✅ **MoonBit原生支持** - 文档完善，支持良好

### 实现步骤

#### 1. 编译OpenDAL为Wasm

```toml
# bindings/wasm/Cargo.toml
[package]
name = "opendal-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
opendal = { path = "../../core", default-features = false, features = ["services-memory"] }
wasm-bindgen = "0.2"
tokio = { version = "1", features = ["rt"] }

[profile.release]
opt-level = "z"  # 优化大小
lto = true
```

```rust
// bindings/wasm/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Operator {
    inner: opendal::Operator,
}

#[wasm_bindgen]
impl Operator {
    #[wasm_bindgen(constructor)]
    pub fn new(scheme: &str) -> Result<Operator, JsValue> {
        let op = opendal::Operator::new(opendal::Scheme::from_str(scheme)?)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .finish();
        Ok(Operator { inner: op })
    }

    pub async fn write(&self, path: &str, data: &[u8]) -> Result<(), JsValue> {
        self.inner.write(path, data).await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn read(&self, path: &str) -> Result<Vec<u8>, JsValue> {
        self.inner.read(path).await
            .map(|b| b.to_vec())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn delete(&self, path: &str) -> Result<(), JsValue> {
        self.inner.delete(path).await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

#### 2. MoonBit Wasm FFI绑定

```moonbit
// src/wasm_ffi.mbt

// 导入Wasm函数
extern "wasm" fn opendal_operator_new(scheme : String) -> Int =
  #|(import "opendal" "operator_new")
  #|(func (param i32 i32) (result i32))

extern "wasm" fn opendal_operator_write(
  op : Int,
  path : String,
  data : Bytes
) -> Int =
  #|(import "opendal" "operator_write")
  #|(func (param i32 i32 i32 i32 i32) (result i32))

extern "wasm" fn opendal_operator_read(
  op : Int,
  path : String
) -> Bytes =
  #|(import "opendal" "operator_read")
  #|(func (param i32 i32 i32) (result i32))

extern "wasm" fn opendal_operator_delete(
  op : Int,
  path : String
) -> Int =
  #|(import "opendal" "operator_delete")
  #|(func (param i32 i32 i32) (result i32))
```

#### 3. 配置moon.pkg.json

```json
{
  "link": {
    "wasm": {
      "exports": [],
      "import-memory": {
        "module": "env",
        "name": "memory"
      },
      "flags": [
        "--import-table",
        "--export-table"
      ]
    }
  }
}
```

### 挑战
1. ⚠️ **异步支持** - Wasm中的异步需要特殊处理
2. ⚠️ **文件系统访问** - 需要WASI支持
3. ⚠️ **性能** - 可能比原生C慢

### 解决方案
- 使用 `wasm-bindgen-futures` 处理异步
- 使用 `wasi` crate 提供文件系统访问
- 对于性能敏感的操作，考虑使用SIMD

## 方案2: 直接Rust FFI（通过静态链接）⭐⭐⭐⭐

### 概述
将OpenDAL编译为静态库，使用单线程Tokio运行时，通过C ABI调用。

### 与当前方案的区别
- **不使用OpenDAL的C绑定**
- **创建MoonBit专用的Rust FFI层**
- **使用单线程Tokio运行时**

### 实现步骤

#### 1. 创建MoonBit专用的Rust FFI

```toml
# bindings/moonbit-rust/Cargo.toml
[package]
name = "opendal-moonbit"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["staticlib"]

[dependencies]
opendal = { path = "../../core", features = ["services-memory"] }
tokio = { version = "1", features = ["rt"] }

[profile.release]
lto = true
```

```rust
// bindings/moonbit-rust/src/lib.rs
use std::sync::LazyLock;

// MoonBit专用：单线程运行时
static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
});

#[repr(C)]
pub struct MoonbitOperator {
    inner: *mut opendal::Operator,
}

#[no_mangle]
pub extern "C" fn moonbit_operator_new(
    scheme: *const u8,
    scheme_len: usize
) -> *mut MoonbitOperator {
    let scheme_str = unsafe {
        std::str::from_utf8_unchecked(
            std::slice::from_raw_parts(scheme, scheme_len)
        )
    };

    let _guard = RUNTIME.enter();

    match opendal::Operator::new(opendal::Scheme::from_str(scheme_str).unwrap()) {
        Ok(op) => {
            let boxed = Box::new(MoonbitOperator {
                inner: Box::into_raw(Box::new(op.finish())),
            });
            Box::into_raw(boxed)
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn moonbit_operator_write(
    op: *const MoonbitOperator,
    path: *const u8,
    path_len: usize,
    data: *const u8,
    data_len: usize,
) -> i32 {
    let op = unsafe { &*(*op).inner };
    let path_str = unsafe {
        std::str::from_utf8_unchecked(
            std::slice::from_raw_parts(path, path_len)
        )
    };
    let data_slice = unsafe {
        std::slice::from_raw_parts(data, data_len)
    };

    let _guard = RUNTIME.enter();

    match RUNTIME.block_on(op.write(path_str, data_slice)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

// ... 其他函数类似
```

#### 2. MoonBit绑定

```moonbit
// src/ffi.mbt
type MoonbitOperator

extern "C" fn moonbit_operator_new(
  scheme : Bytes,
  scheme_len : UInt
) -> MoonbitOperator = "moonbit_operator_new"

extern "C" fn moonbit_operator_write(
  op : MoonbitOperator,
  path : Bytes,
  path_len : UInt,
  data : Bytes,
  data_len : UInt
) -> Int = "moonbit_operator_write"
```

#### 3. 配置

```json
{
  "link": {
    "native": {
      "cc-link-flags": "-L../moonbit-rust/target/release -lopendal_moonbit"
    }
  }
}
```

### 优势
1. ✅ **完全控制** - 我们控制Tokio运行时配置
2. ✅ **单线程运行时** - 避免TLS问题
3. ✅ **性能** - 原生性能，无Wasm开销
4. ✅ **不影响其他绑定** - 独立的库

### 挑战
1. ⚠️ **维护成本** - 需要维护独立的FFI层
2. ⚠️ **平台兼容性** - 需要为每个平台编译

## 方案3: Wasm Component Model（未来）⭐⭐⭐

### 概述
使用WebAssembly Component Model，这是Wasm的下一代标准。

### 优势
1. ✅ **标准化接口** - WIT (WebAssembly Interface Types)
2. ✅ **更好的互操作性** - 组件之间可以直接通信
3. ✅ **类型安全** - 强类型接口定义

### 状态
⚠️ **尚未成熟** - MoonBit和OpenDAL都需要支持

### 示例WIT定义

```wit
// opendal.wit
interface opendal {
  record operator {
    handle: u32
  }

  new: func(scheme: string) -> result<operator, string>
  write: func(op: operator, path: string, data: list<u8>) -> result<_, string>
  read: func(op: operator, path: string) -> result<list<u8>, string>
  delete: func(op: operator, path: string) -> result<_, string>
}
```

## 方案对比

| 方案 | 复杂度 | 性能 | 稳定性 | 跨平台 | 推荐度 |
|------|--------|------|--------|--------|--------|
| 当前C绑定 | 低 | 高 | ⚠️ 不稳定 | ✅ | ⭐⭐ |
| Wasm FFI | 中 | 中 | ✅ 稳定 | ✅✅ | ⭐⭐⭐⭐⭐ |
| 直接Rust FFI | 中 | 高 | ✅ 稳定 | ✅ | ⭐⭐⭐⭐ |
| Wasm Component | 高 | 中 | ⚠️ 未成熟 | ✅✅ | ⭐⭐⭐ |

## 推荐方案

### 短期（立即实施）：方案2 - 直接Rust FFI
**原因**：
1. 最快实现
2. 完全控制运行时配置
3. 原生性能
4. 不影响其他绑定

### 中期（1-2个月）：方案1 - Wasm FFI
**原因**：
1. 更好的跨平台支持
2. 避免所有原生代码问题
3. MoonBit对Wasm支持很好
4. 未来趋势

### 长期（6个月+）：方案3 - Wasm Component Model
**原因**：
1. 标准化
2. 更好的互操作性
3. 等待生态成熟

## 实施建议

### 第一步：实现方案2（直接Rust FFI）
1. 创建 `bindings/moonbit-rust/` 目录
2. 实现单线程Tokio运行时的FFI层
3. 更新MoonBit绑定使用新的FFI
4. 测试稳定性

### 第二步：探索方案1（Wasm FFI）
1. 研究OpenDAL的Wasm支持
2. 创建概念验证
3. 评估性能和功能完整性
4. 如果可行，逐步迁移

### 第三步：关注方案3（Component Model）
1. 跟踪Wasm Component Model发展
2. 参与MoonBit和OpenDAL的Wasm支持
3. 在标准成熟时迁移

## 结论

**最佳路径**：
1. 立即实施**方案2（直接Rust FFI）**解决当前问题
2. 并行探索**方案1（Wasm FFI）**作为长期方案
3. 关注**方案3（Component Model）**的发展

这样既能快速解决问题，又为未来的改进留有空间，且完全不影响其他语言绑定。
