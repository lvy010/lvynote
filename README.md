## Notebook backup(Work In Progress & Pending Integration)

(目录归档开始于12.9，有时间按如下规范组织之前8-12月[note](./note)中笔记in progress...）



- Box64:  Linux用户态x86_64模拟器，在ARM64/RISC-V等非x86架构上运行x86_64 Linux程序，支持动态库翻译、JIT编译和系统调用转换，为==跨架构软件兼容提供高性能解决方案==_12.15
  - note: [lvynote/note/C Advanced/box64 at main · lvy010/lvynote](https://github.com/lvy010/lvynote/tree/main/note/C%20Advanced/box64)
  - **Box64 核心特性：**
    - 🔄 指令翻译：x86_64指令实时转换为目标架构指令
    - ⚡ JIT编译：动态编译优化，提升执行性能
    - 📚 库包装：自动处理x86_64动态库的加载和调用
    - 🎮 游戏支持：兼容Steam、Wine等游戏平台
    - 🖥️ 系统调用：完整的Linux系统调用转换层
    - 🏗️ 多架构：支持ARM64、RISC-V、LoongArch等目标平台
    - 🔧 调试友好：提供详细的执行日志和性能分析工具
    - 🆓 开源免费：MIT许可证，社区驱动开发

----------

- Slamtec RPLIDAR SDK 为激光雷达产品创建机器人导航和三维扫描等应用_12.9
    - note:https://github.com/lvy010/lvynote/tree/main/note/Robot/rplidar_sdk
    - code:https://github.com/lvy010/Cpp-Lib-test/tree/main/rplidar_demo
- RouteLLM 智能路由在不同LLM之间选择，在保持响应质量的同时降低成本_12.10
  - note：[lvynote/note/机器学习/RouteLLM at main · lvy010/lvynote](https://github.com/lvy010/lvynote/tree/main/note/机器学习/RouteLLM)
- OpenAI Glasses for Navigation AI智能眼镜系统，通过视频音频传感器数据为视障人士提供实时导航辅助*_12.11*
  - note: [lvynote/note/AI_apps/AI_glasses at main · lvy010/lvynote](https://github.com/lvy010/lvynote/tree/main/note/AI_apps/AI_glasses)
- Drogon: C++14/17的高性能HTTP应用框架，支持WebSocket/异步非阻塞IO/ORM/微服务_12.12  
  - note: [lvynote/note/C++/Drogon_net at main · lvy010/lvynote](https://github.com/lvy010/lvynote/tree/main/note/C%2B%2B/Drogon_net)
  - code:
  - **Drogon 核心特性：**
    - 🚀 高性能：基于非阻塞IO和epoll/kqueue
    - 🔌 支持HTTP/1.1、WebSocket协议
    - 💾 内置ORM，支持PostgreSQL/MySQL/SQLite
    - 🎯 RESTful API友好
    - ⚡ 支持协程（C++20）
    - 🔧 跨平台（Linux/macOS/Windows）

- OpenTitan: 开源安全芯片项目，提供透明可审计的硬件信任根(Root of Trust)实现，包含加密加速器、密钥管理、安全启动等模块，采用SystemVerilog开发，为嵌入式系统和数据中心提供可信的硬件安全基础_12.13
  - note: [lvynote/note/C Advanced/OpenTitan at main · lvy010/lvynote](https://github.com/lvy010/lvynote/tree/main/note/C%20Advanced/OpenTitan)
  - **OpenTitan 核心特性：**
    - 🔐 硬件信任根：提供安全启动和固件验证
    - 🔑 密码学引擎：AES/HMAC/KMAC等加密加速器
    - 🛡️ 安全隔离：独立的安全处理器和内存保护
    - 📖 完全开源：硬件设计、固件、工具链全部开放
    - ✅ 工业级质量：符合安全认证标准(Common Criteria)
    - 🌐 社区驱动：lowRISC主导，Google/多家企业参与
