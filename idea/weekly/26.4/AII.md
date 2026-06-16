# Context Is All You Need

> 模型能做什么，不取决于架构多精妙——取决于人类写过多少关于这件事的字。

---

## 1. 训练数据决定能力上限，这不是猜测，是实验结果

> [Measuring LLMs on Esoteric Languages](https://arxiv.org/abs/2503.xxxxx) — 两位研究者用 Brainfuck、Befunge-98、Whitespace、Unlambda、Shakespeare 五种小众语言，测试 GPT-5.2、O4-mini、Gemini 3 Pro、Qwen3-235B、Kimi K2，平均正确率 **3.8%**，Python 同类任务正确率 **90%**。更难级别（初级/中级/高级）所有模型正确率为 **0**。

「context is all you need」这句话是反过来读的：**没有 context，就没有 intelligence**。那些号称"通用推理"的模型，一碰到训练语料空白区，直接原形毕露。架构不重要，数据才是护城河。

---

## 2. 真人客服正在成为付费溢价品

> [HP Customer Support 强制 15 分钟等待](https://arstechnica.com/gadgets/2025/02/misguided-hp-customer-support-approach-included-forced-15-minute-call-wait-times/) — 打惠普客服电话，系统先让你去官网自助；坚持要真人，等 15 分钟；中途挂断，重新计时；第 5、10、13 分钟各提醒一次「可以发邮件」。

感觉以后免费就只有 AI 客服，额外付费才有真人客服。HP 这套设计不是失误，是预演——**用摩擦成本把真人服务变成差异化产品**。

---

## 3. 手机 App 的显示层，是 AI 下一个要吞掉的基础设施

> [Why Your Next Mobile App Is Probably Headless](https://tuananh.net/2026/03/18/why-your-next-mobile-app-is-probably-headless/) — 如果用户通过 AI 助手交互，App 的视觉层就变成冗余；后端只需要暴露数据接口和语义描述。

前端本质上是同一件事：**向用户展示数据，让用户处理数据**。如果 AI 助手接管了"向用户展示"这一步，前端就只剩数据层。新范式是：后端提供数据 + 用途描述，UI 由 AI 实时生成。这不是科幻，[Ruby Is the Best Language for AI Apps](https://paolino.me/ruby-is-the-best-language-for-ai-apps/) 这类文章已经在讨论配套的语言栈了。

---

## 4. 越用 AI，越不担忧失业——但原因值得想清楚

> [We Used AI to Build a JS Engine in 6 Weeks](https://bellard.org/) — 作者六周生成了一个 100% 通过 test262 测试集的 JavaScript 引擎，覆盖全部 98,426 个场景。

AI 编程只是流程的一部分。**我的工作是：找出可以用代码解决的问题，解决它，验证解决方案有效**。AI 能接管中间的编码部分，但发现问题、定义问题、确认问题解决——这是工作的 80%。花在 AI 编程的时间越多，对职业生涯的担忧反而越少。

---

## 5. 算力是稀缺资源，"无限 AI"是幻觉

> 三件事同时发生：  
> — OpenAI 关闭 Sora，原因是算力不够，要优先保核心业务  
> — Anthropic 禁止包月套餐用于 OpenClaw、OpenCode 等第三方，因为足额使用的算力成本远超套餐费用  
> — GitHub 今年前三个月代码提交量是去年同期的 **14 倍**，频繁故障

拆解整个需求，思考状态的流转，设计模块之间的关系，在脑中提前模拟整个执行路径——**这是编程本质，不会被 AI 替代，因为 AI 本身也需要有人给它做这件事**。算力有上限，而会分配算力的人，比算力本身更稀缺。

---

## 6. 【警惕】AI 正在成为社会工程学攻击的武器

> [开源维护者遭 AI 深度伪造攻击（Jason Saayman 自述）](https://github.com) — 攻击者克隆创始人外貌与公司品牌，建真实 Slack 工作区，在 Microsoft Teams 会议中以"系统组件过时"为由诱导安装远程木马（RAT）。

这个攻击是有剧本的，每一步都经过策划、充分准备和排练，完全为你度身定制。**AI 让社会工程学攻击的成本降到接近零，但攻击的精度却接近人工**。以后碰到"需要安装组件"的视频会议，默认拒绝。
