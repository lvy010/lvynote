



哈喽大家好这个视频主要分为



开源项目 

十篇文章

一些论文与时事







Why Your Next Mobile App Is Probably Headless

新范式：后端提供数据 + 用途描述，UI 由 AI 实时生成

前端本质上是同一件事：向用户展示数据。如果 AI 助手接管了这一步，前端就只剩数据层

这不是科幻，Ruby Is the Best Language for AI Apps

这类文章已经在讨论配套的语言栈了。



三件事同时发生，印证着算力的不足
OpenAI 关闭 Sora，GitHub 频繁故障，Anthropic 禁止包月套餐用于 OpenClaw、OpenCode 等第三方，因为足额使用的算力成本远超套餐费用

算力有上限，而会分配算力的人，比算力本身更稀缺。

拆解整个需求，思考状态的流转，设计模块之间的关系，在脑中提前模拟整个执行路径——这是编程本质，不会被 AI 替代，因为 AI 本身也需要有人给它做这件事。



惠普 Customer Support 强制 15 分钟等待

感觉以后免费就只有 AI 客服，额外付费才有真人客服，马太效应加剧



We Used AI to Build a JS Engine in 6 Weeks，作者六周生成了一个 100% 通过 test262 测试集的 JavaScript 引擎

AI 编程只是流程的一部分，能接管中间的编码部分。人的工作是：找出可以用代码解决的问题，解决它，验证解决方案有效。







我们来看开源项目部分

dive-into-LLMs

上交张倬胜老师主导的大模型教程。

大多数「大模型教程」是 API 调用包装，这个是真的从原理讲，适合想搞懂「为什么」的人，而不只是「怎么调」的人。





GenericAgent

一个极简架构、低 Token的自进化框架。

从寒假的evomap到现在，agent的自动进化一直是一个热门话题，[GenericAgent]3000多行代码很易于上手学习



Voicebox

几秒语音样本完成声音克隆

没错这个视频我的声音就是通过这个克隆来的，ElevenLabs 订阅不便宜，大家有需要的可以试一下这个平替



context-mode
 AI 编程场景上下文利用率低的问题

Context 就是 Agent 的工作记忆。通过压缩和优先级排序，让 Agent 在有限窗口内塞入更多有效信息。





DeepGEMM

专注 FP8 精度下的矩阵乘法加速

GEMM 是深度学习推理训练的核心计算瓶颈，DeepSeek 团队做了极致内核优化。



opensre

运维是最适合 Agent 落地的领域之一：任务重复、规则明确、错误代价高

真正的挑战是在什么情况下你敢让 Agent 自动操作生产环境。这是边界问题，不是技术问题。