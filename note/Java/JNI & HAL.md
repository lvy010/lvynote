前文：[rplidar_sdk/6多平台os抽象原语HAL.md](https://github.com/lvy010/lvynote/blob/main/note/Robot/rplidar_sdk/6%E5%A4%9A%E5%B9%B3%E5%8F%B0os%E6%8A%BD%E8%B1%A1%E5%8E%9F%E8%AF%ADHAL.md)
源于对rplidar_sdk代码的研究
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/d144a8382620428982279d12ef737dce.png)
下文是对于HAL的扩展内容

# Android HAL（硬件抽象层）

Android 的==硬件抽象层（Hardware Abstraction Layer, HAL==）是 Android 系统架构中的关键组件，它在 Linux 内核驱动和上层应用框架之间架起了一座桥梁。

本文将从计算机专家的视角，解析 Android HAL 的设计理念、实现原理及完整的调用链路。

## 一、为什么需要 HAL？设计初衷

### 1.1 GPL 许可证的"绕行"策略

Android HAL 的诞生有着明确的商业目的。

传统 Linux 内核驱动遵循 GPL（GNU General Public License）许可证，这意味着任何基于 GPL 代码的衍生作品都必须开源。对于硬件厂商而言，这无疑是一个巨大的挑战——他们的核心技术和专有算法可能被迫公开。

Android 采用了一个巧妙的解决方案：

- **内核层极简化**：Linux 内核驱动仅保留最基础的硬件寄存器读写操作
- **逻辑层上移**：==将体现硬件特性的控制逻辑、算法实现全部移至用户空间==（User Space）
- **许可证切换**：HAL 层采用 Apache 许可证，允许厂商提供闭源的二进制库

这种设计使得 Android 成为一个**开放平台**而非完全的**开源平台**，硬件厂商可以在保护知识产权的同时参与 Android 生态。

### 1.2 架构优势

```css
应用层 (Java/Kotlin)
        ↓
应用框架层 (Java Framework)
        ↓
JNI 桥接层
        ↓
HAL 层 (C/C++ 动态库) ← 厂商可闭源
        ↓
内核驱动 (仅基础读写) ← GPL 开源
        ↓
硬件设备
```

## 二、技术实现：JNI 的关键作用

### 2.1 JNI（Java Native Interface）简介

==JNI 是连接 Java 世界和 C/C++ 世界的桥梁==，允许：

- Java 程序调用 C/C++ 编写的本地代码（Native Code）
- 本地代码访问 Java 对象和方法

在 Android HAL 中，JNI 的作用至关重要：

- **性能优化**：硬件控制逻辑用 C/C++ 实现，执行效率更高
- **代码复用**：可以直接使用现有的 C/C++ 硬件控制库
- **闭源保护**：==编译为 `.so` 动态链接库，不暴露源代码==

### 2.2 两种调用方式对比

**方式一：直接调用（简单但不推荐）**
```css
应用 → .so 动态库 → HAL → 内核驱动
```

**方式二：标准框架调用（推荐）**
```css
应用 → Manager → Service(Java) → Service(JNI) → HAL → 内核驱动
```

第二种方式虽然看似复杂，但==更符合 Android 的分层架构设计，便于权限管理、资源调度和系统维护==。

## 三、实现案例：LED 控制系统

以 [Mokoid 开源项目](https://github.com/moko365/mokoid-stub)为例，展示一个完整的 HAL 实现。

### 3.1 项目结构树

```css
mokoid/
├── apps/                          # 应用层
│   ├── LedClient/                 # 直接调用 Service
│   │   ├── AndroidManifest.xml
│   │   └── src/com/mokoid/LedClient/
│   │       └── LedClient.java
│   └── LedTest/                   # 通过 Manager 调用
│       ├── AndroidManifest.xml
│       └── src/com/mokoid/LedTest/
│           ├── LedSystemServer.java
│           └── LedTest.java
│
├── frameworks/base/               # 框架层
│   ├── core/java/mokoid/hardware/
│   │   ├── ILedService.aidl      # 服务接口定义（AIDL）
│   │   └── LedManager.java       # Manager 实现
│   └── service/
│       ├── com.mokoid.server.xml
│       ├── java/com/mokoid/server/
│       │   └── LedService.java   # Service Java 实现
│       └── jni/
│           └── com_mokoid_server_LedService.cpp  # JNI 实现
│
└── hardware/modules/              # HAL 层
    ├── include/mokoid/
    │   └── led.h                 # HAL 接口定义
    └── led/
        └── led.c                 # HAL 实现（硬件控制）
```

### 3.2 核心代码

#### 第一层：内核驱动

```c
// 内核驱动仅提供基础接口
static int led_open(struct inode *inode, struct file *file) { ... }
static ssize_t led_write(struct file *file, const char __user *buf, 
                         size_t count, loff_t *ppos) 
{
    // 仅执行寄存器写入操作
    writel(value, LED_REGISTER_ADDR);
    return count;
}
```

**关键特点**：没有控制逻辑，只有硬件操作。

#### 第二层：HAL 层（核心实现）

```c
// led.h - HAL 接口定义
struct led_module_t {
    struct hw_module_t common;  // 标准硬件模块结构
};

struct led_control_device_t {
    struct hw_device_t common;  // 标准硬件设备结构
    
    int fd;  // 设备文件描述符
    
    // 硬件控制接口
    int (*set_on)(struct led_control_device_t *dev, int32_t led);
    int (*set_off)(struct led_control_device_t *dev, int32_t led);
};
```

```c
// led.c - HAL 实现
int led_on(struct led_control_device_t *dev, int32_t led) {
    LOGI("LED Stub: set %d on.", led);
    // 这里实现复杂的控制逻辑
    // 例如：PWM 调制、亮度控制、序列控制等
    write(dev->fd, &led_cmd, sizeof(led_cmd));
    return 0;
}

int led_off(struct led_control_device_t *dev, int32_t led) {
    LOGI("LED Stub: set %d off.", led);
    write(dev->fd, &led_cmd, sizeof(led_cmd));
    return 0;
}

static int led_device_open(const struct hw_module_t* module, 
                          const char* name,
                          struct hw_device_t** device) {
    struct led_control_device_t *dev;
    
    dev = (struct led_control_device_t *)malloc(sizeof(*dev));
    memset(dev, 0, sizeof(*dev));
    
    // 打开内核驱动设备节点
    dev->fd = open("/dev/led", O_RDWR);
    
    // 绑定硬件操作函数
    dev->set_on = led_on;
    dev->set_off = led_off;
    
    *device = &dev->common;
    return 0;
}

// HAL 模块方法表
static struct hw_module_methods_t led_module_methods = {
    .open = led_device_open
};

// HAL 模块信息符号（动态库加载入口）
const struct led_module_t HAL_MODULE_INFO_SYM = {
    .common = {
        .tag = HARDWARE_MODULE_TAG,
        .version_major = 1,
        .version_minor = 0,
        .id = LED_HARDWARE_MODULE_ID,
        .name = "Sample LED Stub",
        .author = "The Mokoid Open Source Project",
        .methods = &led_module_methods,
    }
};
```

**编译输出**：==`libled.so`，安装路径：`/system/lib/hw/`==

#### 第三层：JNI 桥接层

```cpp
// com_mokoid_server_LedService.cpp
static jint mokoid_setOn(JNIEnv* env, jobject thiz, jint led) {
    led_control_device_t* device = 
        (led_control_device_t*)env->GetIntField(thiz, gDeviceField);
    
    if (!device) {
        LOGE("Device not open");
        return -1;
    }
    
    return device->set_on(device, led);
}

static jint mokoid_setOff(JNIEnv* env, jobject thiz, jint led) {
    led_control_device_t* device = 
        (led_control_device_t*)env->GetIntField(thiz, gDeviceField);
    
    return device->set_off(device, led);
}

// JNI 方法注册表
static JNINativeMethod gMethods[] = {
    {"_init",   "()Z", (void*)mokoid_init},
    {"setOn",   "(I)I", (void*)mokoid_setOn},
    {"setOff",  "(I)I", (void*)mokoid_setOff},
};

// JNI 库加载入口
jint JNI_OnLoad(JavaVM* vm, void* reserved) {
    JNIEnv* env = NULL;
    
    if (vm->GetEnv((void**)&env, JNI_VERSION_1_4) != JNI_OK) {
        return -1;
    }
    
    jclass clazz = env->FindClass("com/mokoid/server/LedService");
    if (env->RegisterNatives(clazz, gMethods, 
                            sizeof(gMethods)/sizeof(gMethods[0])) < 0) {
        return -1;
    }
    
    return JNI_VERSION_1_4;
}
```

#### 第四层：Java Service 层

```java
// LedService.java
package com.mokoid.server;

public class LedService extends ILedService.Stub {
    private static final String TAG = "LedService";
    private int mNativePointer;  // 指向 HAL 设备的指针
    
    public LedService() {
        // 加载 JNI 动态库
        System.loadLibrary("mokoid_runtime");
        _init();  // 调用 JNI 初始化方法
    }
    
    // Native 方法声明
    private native boolean _init();
    private native int setOn(int led);
    private native int setOff(int led);
    
    // AIDL 接口实现
    @Override
    public boolean setLedOn(int led) {
        Log.d(TAG, "setLedOn: " + led);
        return setOn(led) == 0;
    }
    
    @Override
    public boolean setLedOff(int led) {
        Log.d(TAG, "setLedOff: " + led);
        return setOff(led) == 0;
    }
}
```

#### 第五层：Manager 层

```java
// LedManager.java
package mokoid.hardware;

public class LedManager {
    private static final String TAG = "LedManager";
    private ILedService mService;
    
    public LedManager(ILedService service) {
        mService = service;
    }
    
    public boolean turnOn(int led) {
        try {
            return mService.setLedOn(led);
        } catch (RemoteException e) {
            Log.e(TAG, "RemoteException in turnOn", e);
            return false;
        }
    }
    
    public boolean turnOff(int led) {
        try {
            return mService.setLedOff(led);
        } catch (RemoteException e) {
            Log.e(TAG, "RemoteException in turnOff", e);
            return false;
        }
    }
}
```

#### 第六层：应用层

```java
// LedTest.java
package com.mokoid.LedTest;

public class LedTest extends Activity {
    private LedManager mLedManager;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // 获取 LED Manager
        mLedManager = (LedManager)getSystemService("led");
        
        // 控制 LED
        findViewById(R.id.btn_on).setOnClickListener(v -> {
            mLedManager.turnOn(0);  // 打开 LED 0
        });
        
        findViewById(R.id.btn_off).setOnClickListener(v -> {
            mLedManager.turnOff(0);  // 关闭 LED 0
        });
    }
}
```

## 四、HAL 加载机制详解

### 4.1 动态库加载流程

当 Service 调用 `hw_get_module()` 时（定义在 `hardware/libhardware/hardware.c`），系统会：

1. **构造库文件名**：根据硬件 ID 生成文件名，如 `led.default.so`
2. **搜索路径**：按顺序在以下路径查找：
   ```css
   /vendor/lib/hw/
   /system/lib/hw/
   ```
3. **加载动态库**：==使用 `dlopen()` 加载找到的 `.so` 文件==
4. **查找符号**：使用 `dlsym()` 查找 `HAL_MODULE_INFO_SYM` 符号
5. **返回模块**：==将模块结构体指针返回给调用者==

```c
// hardware.c 简化实现
int hw_get_module(const char *id, const struct hw_module_t **module) {
    char path[PATH_MAX];
    void *handle;
    struct hw_module_t *hmi;
    
    // 构造库文件路径
    snprintf(path, sizeof(path), "/system/lib/hw/%s.default.so", id);
    
    // 加载动态库
    handle = dlopen(path, RTLD_NOW);
    if (!handle) {
        return -ENOENT;
    }
    
    // 查找模块信息符号
    hmi = (struct hw_module_t *)dlsym(handle, HAL_MODULE_INFO_SYM_AS_STR);
    if (!hmi) {
        dlclose(handle);
        return -EINVAL;
    }
    
    *module = hmi;
    return 0;
}
```

### 4.2 设备打开流程

```c
// 应用调用流程
hw_get_module(LED_HARDWARE_MODULE_ID, &module);  // 1. 加载模块
module->methods->open(module, LED_HARDWARE_DEVICE_ID, &device);  // 2. 打开设备
led_device->set_on(led_device, 0);  // 3. 调用硬件操作
```

## 五、调用链路
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/e5849341be4b4e79b82d90648f7e2e04.png)


## 六、总结

### 6.1 HAL 设计原则

1. **接口标准化**：所有 HAL 模块都继承自 `hw_module_t` 和 `hw_device_t`
2. **动态加载**：运行时根据需要加载对应的 `.so` 库
3. **版本管理**：通过 `version_major` 和 `version_minor` 管理接口兼容性
4. **多实例支持**：同一硬件可以有多个实现（如 `led.default.so`、`led.vendor.so`）

### 6.2 性能优化考虑

- **JNI 调用开销**：频繁的 JNI 调用会有性能损耗，应批量处理数据
- **内存管理**：注意 Java 和 Native 层的内存生命周期管理
- **线程安全**：HAL 层需要考虑多线程并发访问

### 6.3 与 Linux 主线的分歧

Android 的这种 HAL 设计导致了与 Linux 主线内核的分歧：

- **功能不完整**：内核驱动功能被人为削弱，无法独立使用
- **平台绑定**：==驱动强依赖 Android HAL，无法移植到其他 Linux 发行版==
- **维护困难**：内核和用户空间的职责划分不清晰

这也是为什么 Linux 内核社区一度将 Android 相关代码移出主线的原因之一。

## 七、建议

### 7.1 开发新 HAL 的步骤

1. **定义接口**：在 `hardware/libhardware/include/` 下定义头文件
2. **实现 HAL**：在 `hardware/modules/` 下实现 C/C++ 代码
3. **编写 JNI**：在 `frameworks/base/services/jni/` 下实现 JNI 桥接
4. **创建 Service**：在 `frameworks/base/services/java/` 下实现 Java Service
5. **提供 Manager**：在 `frameworks/base/core/java/` 下实现 Manager API
6. **编写测试应用**：验证完整调用链路

### 7.2 调试技巧

```bash
# 查看 HAL 库加载日志
adb logcat | grep "hw_get_module"

# 检查动态库是否存在
adb shell ls -l /system/lib/hw/*.so

# 查看 JNI 调用日志
adb logcat | grep "JNI"

# 使用 strace 跟踪系统调用
adb shell strace -p <pid>
```

## 结语

Android HAL 是一个精妙的设计，它在开源理念和商业利益之间找到了平衡点。通过将硬件控制逻辑从内核空间移至用户空间，Android 不仅保护了硬件厂商的知识产权，也为自己赢得了更广泛的硬件生态支持。

`理解 HAL 的实现原理，对于 Android 系统开发者、驱动工程师以及应用开发者都具有重要意义`。它不仅是技术实现的范例，更是软件架构设计和商业策略结合的典范。

---

**参考资源**：
- Android 官方文档：[Hardware Abstraction Layer (HAL)](https://source.android.com/devices/architecture/hal)
- Mokoid 开源项目：提供完整的 HAL 实现示例
- Linux 内核文档：理解传统驱动模型

**注**：本文基于 Android 早期 HAL 架构（HAL 1.0），现代 Android 已演进至 HIDL/AIDL HAL，但核心设计理念仍然相通。