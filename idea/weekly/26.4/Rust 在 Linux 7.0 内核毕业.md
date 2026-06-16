
Driver Core 的三层改造

Linux 7.0 完成了一件内核社区争论 6 年的里程碑事件：**Rust 从 EXPERIMENTAL 毕业成为稳定特性**。首个受益的子系统是内核中最庞大也最脆弱的 **Driver Core**
本次改造的核心约束是：**C 代码一行不改**，Rust 通过三层抽象改造成功嵌入 30 年历史的老代码基座。

---

## 一、背景

### 1.1 Driver Core 的现状

将 Driver Core 比作一栋运行了 30 年的老楼：

| 组件 | 比喻 | Rust 的解决方案 |
|------|------|----------------|
| `kobject`/`kref` | 楼内水管系统 | 外层套 `Arc<Device>` 自动记账 |
| `sysfs` 属性 | 布告栏 | 加装电子屏，类型安全 |
| `probe`/`remove` | 房客入住退房 | 门上换智能锁（RAII） |

老楼的三大问题：
- **水管要手动记账**：漏水没人管，漏一次加内存泄露，漏一次减 use-after-free
- **布告栏谁都能乱贴**：类型不安全，裸指针操作
- **房客退房钥匙经常收不回来**：资源清理代码复杂，易出错

### 1.2 三位人物

#### Miguel Ojeda
- **角色**：Rust for Linux 项目发起人、核心维护者
- **贡献**：2020 年发出第一封 RFC，6 年后负责编写每一行 C 封装代码
- **铁律**：Rust 代码里除 `unsafe` 块外，不允许出现任何未定义行为

#### Greg KH
- **角色**：Driver Core 维护者，`struct device` 守护人（20 年）
- **转变**：从最初保留态度 → 最终点头同意
- **关键作用**：7.0 合并时亲自 review 每一行 Rust binding，扮演守门人角色

#### Linus Torvalds
- **态度转变**：从"允许试试看" → 7.0 发布邮件首次使用"graduated（毕业）"描述 Rust
- **铁律**：
  1. Rust 代码绝不能阻碍 C 代码的重构
  2. C 侧维护者想动结构就动结构，Rust 侧重新生成 binding
  3. 责任边界清清楚楚

---

## 二、架构分层设计

### 2.1 四层架构总览

```
┌─────────────────────────────────────────────────────────┐
│          驱动开发者面向的世界（Safe API）                │
├─────────────────────────────────────────────────────────┤
│     rust/kernel（手写 Safe 封装层）                      │
│     Arc<Device> | Attribute | Device 等等                │
├─────────────────────────────────────────────────────────┤
│     rust/bindings（bindgen 自动生成，约 10 万行）       │
│     从 include/linux/device.h 自动生成                   │
├─────────────────────────────────────────────────────────┤
│     C 侧 Driver Core（原封不动）                         │
│     struct device | kobject | sysfs 核心代码             │
└─────────────────────────────────────────────────────────┘
```

### 2.2 架构设计原则

| 层级 | 特性 | 说明 |
|------|------|------|
| 越往下 | 越 `unsafe` | C 互操作层 |
| 越往上 | 越 `safe` | 开发者友好 |
| 边界 | 零成本抽象 | `#[repr(transparent)]` 保证 |

### 2.3 bindgen 的作用机制

```
内核 build 时：
  include/linux/device.h
           ↓
     bindgen 自动解析
           ↓
  rust/bindings/devices.rs（~10万行）
           ↓
  Rust 侧开发者不碰此文件
```

**特性**：C 侧结构体加字段 → bindgen 次日自动同步 → Rust 侧编译报错 → Rust 维护者修 binding

---

## 三、第一层改造：引用计数自动化

### 3.1 C 侧的古老问题

```c
// C 代码中的手动引用计数管理
struct device *dev = kobject_get(&parent->kobj);
// ... 使用 dev ...
kobject_put(&parent->kobj);
```

常见 bug 类型：
- `kobject_get` 后忘记 `kobject_put` → 内存泄露
- `kobject_put` 后继续使用 → use-after-free

### 3.2 Rust 的解决方案

```rust
// 灵魂代码：所有权与引用计数的缝合
#[repr(transparent)]
pub struct Device {
    // C 的 struct device 在内存中与此 Rust struct 完全重叠
    _private: [u8; 0],
}

unsafe impl RefCounted for Device {
    // 告诉编译器：此类型天生带引用计数
    // 克隆时自动调 kobject_get
    // Drop 时自动调 kobject_put
}
```

### 3.3 解释

| 特性 | 作用 |
|------|------|
| `#[repr(transparent)]` | Rust `Device` 与 C `struct device` 内存布局完全等价，指针可零成本互转 |
| `unsafe impl RefCounted` | 声明此类型使用引用计数机制 |
| 自动行为 | 克隆时调用 `kobject_get`，Drop 时调用 `kobject_put` |

**效果**：开发者写 Rust 代码==完全不用关心 `kref`，底层走的还是 C 那套机制==，但编译期保证不错漏。

---

## 四、第二层改造：sysfs 属性类型安全

### 4.1 C 侧的问题

```c
// C 的 show 函数签名
ssize_t (*show)(struct device *dev, struct device_attribute *attr,
                char *buf);

// 问题：kobj 是裸指针，类型不明确
// 问题：buf 需要手动处理 buffer overflow
// 问题：device_attribute 回调地狱
```

### 4.2 Rust 的解决方案

```rust
// sysfs 属性的 trait 定义
pub trait Attribute {
    type Parent;  // 类型由编译器推导，防止混淆
    
    fn show(&self, parent: &Self::Parent) -> FormatterResult;
    fn store(&self, parent: &mut Self::Parent, value: &[u8]) -> Result;
}

// FormatterResult 自动处理 buffer 长度
// &[u8] 替代裸 char*，编译期检查边界
```

### 4.3 改进效果

| C 侧 | Rust 侧 |
|------|---------|
| `struct kobject *` 裸指针 | `&Device` 类型安全引用 |
| `char *buf` 手动管理 | `FormatterResult` 自动 buffer 处理 |
| 整类 SYSFS 相关 bug | **从语言层面消失** |

---

## 五、第三层改造：probe/remove 生命周期 RAII 化

### 5.1 C 侧的典型问题

```c
// C 的 probe 函数常见结构
int probe(...) {
    if (alloc_a() != 0) goto err_a;
    if (alloc_b() != 0) goto err_b;
    if (alloc_c() != 0) goto err_c;
    return 0;

err_c:
    free_b();
err_b:
    free_a();
err_a:
    return -ENOMEM;
}

// 资源越多，goto 标签越多
// 30 年驱动中最经典的 bug 来源
```

### 5.2 Rust 的 RAII 机制

```rust
// probe 函数
pub fn probe(pdev: &mut PlatformDriver) -> Result<()> {
    let resource_a = ResourceA::new()?;
    let resource_b = ResourceB::new()?;
    let resource_c = ResourceC::new()?;
    
    // 注册资源...
    Ok(())
    // 函数结束，所有局部变量按后进先出顺序 drop
    // 编译器自动生成回滚代码
}

// remove 函数 - 仅两行有效代码！
fn remove(dev: &mut Device) {
    let boxed = dev.data::<Box<MyDriverState>>().remove();
    // boxed 在函数作用域结束时自动 drop
    // 所有持有的资源全部释放
}
```

### 5.3 效果对比

| C 侧 | Rust 侧 |
|------|---------|
| probe 中 N 个资源 = N 个 goto 标签 + N 个清理函数 | probe 中声明即管理，作用域结束自动清理 |
| remove 中手写几十行清理代码 | **两行有效代码**（取回 Box + 自动 drop） |
| 清理顺序依赖程序员保证 | **后进先出（LIFO）** 编译期保证 |

**意义**：不是语法糖，`是把语义级别的清理上升为**语言机制**`。

---

## 六、C/Rust 互操作机制

### 6.1 共存原理

**q**：Rust 驱动和 C 驱动能共存吗？

**a**：`能`，注册到同一条 `platform_bus`。

```
platform_bus
    │
    ├── C 驱动（传统）
    │       └── 符号表导出函数指针
    │
    └── Rust 驱动
            └── 通过 extern "C" 暴露符合 C ABI 的接口
            └── 函数指针注册到 bus
```

### 6.2 技术

```rust
// Rust 驱动暴露给 C 的接口
#[no_mangle]
pub extern "C" fn my_rust_driver_probe(
    dev: *mut c_void,
    id: *const c_void
) -> c_int {
    // FFI 边界，unsafe 块内操作
    // 调用 safe 封装层
}
```

**关键**：
- `extern "C"` 确保函数签名兼容 C ABI
- ==底层 bus 只看到函数指针==，不知道下游是 C 还是 Rust
- 事件流：`bus_call_probe` → 函数指针 → Rust/C 各自处理

### 6.3 设备生命周期

```
0ms ─── 总线发现设备
2ms ─── C 侧 probe 入口
4ms ─── Rust 侧获取 Device 引用
6ms ─── 注册 sysfs 属性
8ms ─── 设备 Ready

... 设备稳态运行 ...

拔除触发：
remove() ──→ Box::drop() ──→ 资源逐层回收 ──→ kref_put() ──→ C 侧 release()
```

**本质**：Rust 接管的是**`状态转换的时机`**——从人脑管理 → 编译器管理。

---

## 七、工具链要求：Rust 1.95

### 7.1 原因

Linux 7.0 将 Rust 工具链要求提升到 **1.95**，原因是有三个 unstable 特性在该版本才稳定：

| 特性 | 用途 |
|------|------|
| `offset_of` | 计算 C 结构体字段偏移量，用于 unsafe 代码 |
| `unsafe_fields` attributes | 放宽 `unsafe` 字段使用场景 |
| `unsafe extern` 语法改进 | FFI 边界更安全 |

### 7.2 对发行版的影响

- **CI 必须升级 Rust**：否则 7.0 内核编译不通过
- **发行版提前适配**：Fedora、Debian、Arch 等需同步升级工具链

---

## 八、案例

### 8.1 Nova GPU 驱动

- **定位**：NVIDIA GPU 的**纯 Rust 驱动**
- **意义**：Rust for Linux 的**样板工程**
- **性能**：热路径性能与等价 C 实现差距在 **1% 以内**

### 8.2 Synology NAS 驱动

- **定位**：首个**非实验性**的商用 Rust 代码
- **意义**：商业公司愿意在生产环境使用
- **里程碑**：证明 Rust 在企业级场景可用

### 8.3 Asahi Linux GPU 驱动

- **定位**：Apple Silicon 显卡的 Rust 实现
- **意义**：最早的**压力测试用户**之一

### 8.4 `为什么从驱动切入`？

选择 Driver Core 作为 Rust "毕业" 的第一个子系统，原因有三：

1. **API 最稳定**：20 年没大改，适合做 binding
2. **驱动数量最多**：能最快验证 Rust 生态
3. **`漏洞`重灾区**：内核 C 代码一半安全漏洞来自驱动，堵口投入产出比最高

---

## 九、价值总结

### 9.1 范式转变

> 这场改造的本质，不是把 C 翻译成 Rust，而是把**内存安全这件事从程序员的纪律**上升为**`编译器的保证`**。

| 过去 30 年 | Rust 引入后 |
|------------|------------|
| 靠文档规范 | 靠类型系统 |
| 靠 code review | 靠编译器检查 |
| 靠静态分析工具 | 靠 RAII 生命周期 |
| 人类记忆易出错 | 编译器零失误 |

### 9.2 责任边界设计

```
Greg KH 明天想删 struct device 一个字段：
    ↓
他不需要问任何 Rust 维护者
    ↓
他动 C 代码 → 编译错误自然落在 Rust 侧
    ↓
由 Rust 维护者去修 binding
    ↓
C 不用迁就 Rust，Rust 也不能绑架 C
```

这是 Linus 定下的铁律，也是改造能落地的**核心前提**。

### 9.3 性能代价

- **失去的**：一些 `unsafe` 代码块（最小必要使用）
- **得到的**：整类 bug 在编译期消失（内存泄露、use-after-free、资源泄露）
- **实测数据**：Nova 驱动热路径性能差距 **< 1%**

---

## 十、展望

Driver Core 只是第一个毕业的系统。接下来：

- 网络子系统
- 文件系统
- 调度器

都会逐步长出 Rust 抽象层。

**历史意义**：这是 Linux 内核史上第一次迎接一门新的系统语言。6 年前 Miguel Ojeda 发第一封 RFC 时没几个人相信能成，今天 Rust 已正式成为内核一等公民。