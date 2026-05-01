# 本周开源精选：Agent 自主化 + 工具平替化

> 本期 8 个项目，两条主线：Agent 开始能自己进化；付费工具正在被开源复刻。

---

## 1. Agent 可以从种子代码自己长大了

> [GenericAgent](https://github.com/genericagent) — 3300 行种子代码 + 9 个原子工具 + ~100 行 Agent Loop，赋予任意 LLM 系统级本地控制能力，可自主生长技能树，Token 消耗比同类方案低 6 倍。

极简架构、自进化能力、低 Token——这是目前 Agent 框架里最接近「能真正跑起来」的方案。3K 行代码你能读完，黑盒最少。

---

## 2. 大模型入门终于有了一套不水的中文教程

> [dive-into-LLMs](https://github.com/dive-into-llms) — 上海交大张倬胜老师主导，覆盖大模型原理、微调、安全，配套完整代码与 PPT，从课程讲义拓展而来。

大多数「大模型教程」是 API 调用包装。这个是真的从原理讲——配代码、配 PPT、有安全视角，适合想搞懂「为什么」的人，而不是「怎么调」的人。

---

## 3. ElevenLabs 的开源平替，声音克隆几秒搞定

> [Voicebox](https://github.com/voicebox) — 支持 Qwen3-TTS、Chatterbox 等 5 款 TTS 引擎，几秒语音样本完成声音克隆，内置类 DAW 多轨编辑器，全程本地运行，提供 REST API。

ElevenLabs 订阅不便宜。这个本地跑、数据不上云、还有 API——对播客创作者来说，这是目前最值得试的平替。声音克隆这件事的门槛已经低到几秒钟了。

---

## 4. Claude Code 太贵？免费版来了

> [free-claude-code](https://github.com/free-claude-code) — 支持终端、VSCode 插件、Discord，部分版本支持语音输入，完全开源免费。

Anthropic 刚禁止第三方用包月套餐跑 Claude Code，这个项目就冒出来了。AI 工具的平替速度比官方涨价速度快——这本身就是一条值得关注的规律。

---

## 5. Agent 上下文窗口被用滥了，这个工具来优化它

> [context-mode](https://github.com/context-mode) — 针对 AI 编程场景上下文利用率低的问题，通过压缩和优先级排序，让 Agent 在有限窗口内塞入更多有效信息。

Context 就是 Agent 的工作记忆。工作记忆乱，什么都做不好。这个工具解决的是一个被大多数人忽视的效率瓶颈——你买了更大的上下文窗口，但实际用到的有效信息可能只有一半。

---

## 6. 让 AI 真正读懂你的整个项目，而不是孤立的代码片段

> [claude-context](https://github.com/claude-context) — 将完整代码库结构化注入 AI 上下文，使模型在回答问题、生成代码时具备全局项目视野。

AI 编程最大的问题不是生成能力弱，是「看不全」。这个工具解决的是：让 AI 知道你的项目里有什么，而不是每次都从零开始猜。大型项目尤其值得试。

---

## 7. DeepSeek 出手做 GEMM 内核，FP8 推理加速的基础组件

> [DeepGEMM](https://github.com/deepseek-ai/DeepGEMM) — 专注 FP8 精度下的矩阵乘法加速，GEMM 是深度学习推理训练的核心计算瓶颈，DeepSeek 团队做了极致内核优化。

矩阵乘法是整个深度学习的地基。DeepSeek 把这个做成开源——意味着推理加速不再只是大厂私有基础设施的优势。关注 AI infra 的人，这个库值得跟踪。

---

## 8. 把 AI Agent 接进运维，On-call 压力能降多少？

> [opensre](https://github.com/opensre) — 将 AI Agent 引入站点可靠性工程，通过自然语言实现告警分析、故障排查、日志诊断自动化，帮助团队构建自己的 AI 运维助手。

SRE 是最适合 Agent 落地的领域之一：任务重复、规则明确、错误代价高。真正的挑战不是 Agent 能不能用，而是**在什么情况下你敢让 Agent 自动操作生产环境**。这是边界问题，不是技术问题。

---

**本期两条主线：**
- **Agent 自主化**：GenericAgent、opensre — Agent 开始接管执行，不只是辅助
- **工具平替化**：free-claude-code、Voicebox — 付费工具的开源复刻速度超过想象
