
使用纯C语言实现一个完整的机器学习库，最终在MNIST手写数字识别任务上达到约89%的准确率

帮助理解神经网络、反向传播、自动微分的底层机制。

---

## 1：机器学习框架
### 1.1 三大核心组件

| 组件 | 描述 |
|------|------|
| **`张量数学运算`** | 对任意维度的数据（向量、矩阵等）进行`数学运算` |
| **`自动微分`** | 自动计算任意函数的`梯度` |
| **`胶水层`** | 将运算和梯度更新`连接`起来，形成端到端训练流程 |

### 1.2 机器学习的基本流程

`输入`数据 → 模型（函数） → 输出
              ↑
         `大量可调参数`（θ）
              ↓
         损失函数（Cost/Loss Function）
              ↓
         量化模型表现：预测准确→低损失；预测差→高损失
              ↓
         目标：最小化损失函数
              ↓
         使用梯度：∂C/∂θ 指向损失函数下降最快的方向
              ↓
         调整参数：θ_new = θ_old - 学习率 × 梯度


---

## 2：数学层（Tensor/Math Layer）

### 2.1 Arena Allocator

使用`arena allocator`（内存池）进行高效的内存管理，避免频繁的malloc/free：

```c
// 创建arena，分配700MB内存用于处理大数据集
arena_init(&permanent, malloc(700 * 1024 * 1024));
```

### 2.2 矩阵结构体

```c
typedef struct {
    u32 rows;
    u32 columns;
    f32 *data;  // 行优先存储(row-major)
} Matrix;
```

### 2.3 矩阵操作函数列表

| 函数 | 功能 |
|------|------|
| `mat_create()` | 创建矩阵 |
| `mat_copy()` | 复制矩阵 |
| `mat_clear()` | 清零矩阵 |
| `mat_sum()` | 计算矩阵元素和 |
| `mat_scale()` | 标量乘法（原地操作） |
| `mat_add()` | 矩阵加法：`out = A + B` |
| `mat_sub()` | 矩阵减法：`out = A - B` |
| `mat_mul()` | 矩阵乘法，支持转置选项 |
| `mat_relu()` | ReLU激活函数 |
| `mat_softmax()` | Softmax激活函数 |
| `mat_cross_entropy()` | 交叉熵损失函数 |
| `mat_argmax()` | 返回最大值的索引 |

### 2.4 矩阵乘法实现细节

矩阵乘法函数签名：

```c
bool mat_mul(Arena *arena, Matrix *out, Matrix *A, bool transpose_a, 
             Matrix *B, bool transpose_b, bool zero_out);
```

参数说明：
- `transpose_a` / `transpose_b`：是否转置输入矩阵
- `zero_out`：是否在计算前清零输出矩阵

实现四种转置组合的优化版本以获得最佳缓存局部性：

```c
// 非转置版本（IJK顺序最优）
for (u64 i = 0; i < a_rows; i++) {
    for (u64 k = 0; k < a_columns; k++) {
        f32 a_val = A->data[i * a_columns + k];
        for (u64 j = 0; j < b_columns; j++) {
            out->data[i * b_columns + j] += a_val * B->data[k * b_columns + j];
        }
    }
}
```

优化：==**将K循环放在最外层**以获得最佳缓存命中率==。

### 2.5 激活函数实现

#### ReLU (Rectified Linear Unit)

```c
void mat_relu(Matrix *out, Matrix *in) {
    u64 size = in->rows * in->columns;
    for (u64 i = 0; i < size; i++) {
        out->data[i] = in->data[i] > 0 ? in->data[i] : 0;
        // 或: fmaxf(0, in->data[i])
    }
}
```

数学定义：ReLU(x) = max(0, x)

#### Softmax

将输入向量转换为概率分布：

```c
void mat_softmax(Matrix *out, Matrix *in) {
    // 步骤1：计算每个元素的指数
    f32 sum = 0;
    for (u64 i = 0; i < size; i++) {
        out->data[i] = expf(in->data[i]);
        sum += out->data[i];
    }
    // 步骤2：归一化
    mat_scale(out, 1.0f / sum);  // 确保输出和为1
}
```

数学公式：Softmax(x)_i = e^(x_i) / Σ_j e^(x_j)

特性：
- 所有输出为正数
- 输出和为1（有效的概率分布）

#### Cross-Entropy Loss

交叉熵损失函数，专门与Softmax配合使用：

```c
void mat_cross_entropy(Matrix *out, Matrix *P, Matrix *Q) {
    // P: 期望概率分布（one-hot编码）
    // Q: 实际预测分布（Softmax输出）
    for (u64 i = 0; i < size; i++) {
        if (P->data[i] == 0) {
            out->data[i] = 0;  // 避免log(0)
        } else {
            out->data[i] = -P->data[i] * logf(Q->data[i]);
        }
    }
}
```

数学公式：H(P, Q) = -Σ P_i * log(Q_i)

---

## 3：自动微分（Autograd）与计算图

### 3.1 梯度计算的几种方法

| 方法 | 原理 | 缺点 |
|------|------|------|
| **数值微分** | 有限差分：f'(x) ≈ (f(x+h) - f(x))/h | 维度灾难，需要计算n次前向传播 |
| **符号微分** | 解析推导闭式解 | 实现复杂，可能产生符号膨胀 |
| **自动微分** | 构建计算图，利用链式法则 | 无 |

### 3.2 自动微分

1. 构建**计算图**（Computational Graph）
2. 前向传播：按拓扑顺序计算每个节点的值
3. 反向传播：从输出到输入，利用`链式法则累积梯度`

**示例：mx + b**

```
计算图结构：
    M ──┬──→ [Z=A] ←── X
    B ──┘
        ↓
    f = output

反向传播时：
    ∂f/∂f = 1（初始梯度）
    ∂f/∂M = ∂f/∂Z * ∂Z/∂M = 1 * X
    ∂f/∂B = ∂f/∂Z * ∂Z/∂B = 1 * 1
```

### 3.3 模型变量

```c
typedef struct ModelVariable {
    u32 index;           // 变量索引，用于快速访问
    Matrix value;        // 当前值
    Matrix gradient;     // 梯度
    ModelVariable *inputs[2];  // 输入变量（最多2个）
    model_op op;         // 操作类型
    u32 flags;          // 标志位
} ModelVariable;
```

### 3.4 标志位

```c
typedef enum ModelFlag {
    MODEL_FLAG_NONE         = 0,
    MODEL_FLAG_REQUIRE_GRAD = 1 << 0,  // 是否需要计算梯度
    MODEL_FLAG_PARAMETER    = 1 << 1,  // 是否为模型参数（权重/偏置）
    MODEL_FLAG_INPUT        = 1 << 2,  // 输入变量
    MODEL_FLAG_OUTPUT       = 1 << 3,  // 输出变量
    MODEL_FLAG_DESIRED_OUT  = 1 << 4,  // 期望输出
    MODEL_FLAG_COST         = 1 << 5,  // 损失值
} ModelFlag;
```

### 3.5 操作类型枚举

```c
typedef enum ModelOp {
    MODEL_OP_NONE = 0,
    
    MODEL_OP_UNARY_START,    // 一元操作起始标记
    MODEL_OP_RELU,
    MODEL_OP_SOFTMAX,
    
    MODEL_OP_BINARY_START,    // 二元操作起始标记
    MODEL_OP_ADD,
    MODEL_OP_SUBTRACT,
    MODEL_OP_MATMUL,
    MODEL_OP_CROSS_ENTROPY,
} ModelOp;

// 宏：根据操作类型获取输入数量
#define MODEL_OP_NUM_INPUTS(op) \
    ((op) < MODEL_OP_UNARY_START ? 0 : \
     (op) < MODEL_OP_BINARY_START ? 1 : 2)
```

### 3.6 模型上下文结构

```c
typedef struct ModelContext {
    u32 num_vars;                    // 当前变量数量
    Matrix input;                     // 输入引用
    Matrix output;                    // 输出引用
    Matrix desired_output;            // 期望输出引用
    Matrix cost;                      // 损失值引用
    
    struct {
        ModelVariable *out;          // 输出变量
        u32 count;                    // 程序长度
        ModelVariable **vars;         // 拓扑排序后的变量数组
    } forward_program;                // 前向传播程序
    
    struct {
        ModelVariable *out;           // 损失变量
        u32 count;
        ModelVariable **vars;
    } cost_program;                   // 损失计算程序
} ModelContext;
```

### 3.7 计算图的构建与编译

#### 3.7.1 创建模型变量

```c
ModelVariable *mv_create(Arena *arena, ModelContext *model, 
                        u32 rows, u32 columns, ModelFlag flags) {
    ModelVariable *mv = push_array(arena, ModelVariable, 1);
    mv->index = model->num_vars++;
    mv->value = mat_create(arena, rows, columns);
    mv->flags = flags;
    
    // 如果需要梯度，为其分配梯度存储
    if (flags & MODEL_FLAG_REQUIRE_GRAD) {
        mv->gradient = mat_create(arena, rows, columns);
    }
    
    return mv;
}
```

#### 3.7.2 创建一元操作（ReLU/Softmax）

```c
ModelVariable *mv_unary_op(Arena *arena, ModelContext *model,
                          ModelVariable *input, model_op op, ModelFlag flags) {
    // 如果输入需要梯度，输出也需要梯度
    if (input->flags & MODEL_FLAG_REQUIRE_GRAD) {
        flags |= MODEL_FLAG_REQUIRE_GRAD;
    }
    
    ModelVariable *out = mv_create(arena, model, 
                                   input->value.rows, 
                                   input->value.columns, 
                                   flags);
    out->op = op;
    out->inputs[0] = input;
    
    return out;
}

ModelVariable *mv_relu(Arena *arena, ModelContext *model,
                       ModelVariable *input, ModelFlag flags) {
    return mv_unary_op(arena, model, input, MODEL_OP_RELU, flags);
}

ModelVariable *mv_softmax(Arena *arena, ModelContext *model,
                          ModelVariable *input, ModelFlag flags) {
    return mv_unary_op(arena, model, input, MODEL_OP_SOFTMAX, flags);
}
```

#### 3.7.3 创建二元操作（Add/Sub/MatMul/CrossEntropy）

```c
ModelVariable *mv_binary_op(Arena *arena, ModelContext *model,
                            ModelVariable *a, ModelVariable *b, 
                            model_op op, ModelFlag flags) {
    // 任一输入需要梯度，输出就需要梯度
    if (a->flags & MODEL_FLAG_REQUIRE_GRAD || 
        b->flags & MODEL_FLAG_REQUIRE_GRAD) {
        flags |= MODEL_FLAG_REQUIRE_GRAD;
    }
    
    u32 out_rows = (op == MODEL_OP_MATMUL) ? a->value.rows : a->value.rows;
    u32 out_cols = (op == MODEL_OP_MATMUL) ? b->value.columns : a->value.columns;
    
    ModelVariable *out = mv_create(arena, model, out_rows, out_cols, flags);
    out->op = op;
    out->inputs[0] = a;
    out->inputs[1] = b;
    
    return out;
}

ModelVariable *mv_add(Arena *arena, ModelContext *model,
                      ModelVariable *a, ModelVariable *b, ModelFlag flags) {
    return mv_binary_op(arena, model, a, b, MODEL_OP_ADD, flags);
}

ModelVariable *mv_matmul(Arena *arena, ModelContext *model,
                         ModelVariable *a, ModelVariable *b, ModelFlag flags) {
    return mv_binary_op(arena, model, a, b, MODEL_OP_MATMUL, flags);
}
```

### 3.8 程序编译：拓扑排序

==将计算图转换为线性的执行顺序，使得每个节点的依赖在其之前被计算==。

#### DFS拓扑排序实现

```c
ModelProgram program_create(Arena *arena, ModelContext *model, 
                            ModelVariable *root) {
    // 分配临时数据结构
    bool *visited = push_array(&scratch, bool, model->num_vars);
    u64 stack_size = 0;
    u64 out_size = 0;
    ModelVariable **stack = push_array(&scratch, ModelVariable*, model->num_vars);
    ModelVariable **out = push_array(&scratch, ModelVariable*, model->num_vars);
    
    // DFS遍历
    stack[stack_size++] = root;
    while (stack_size > 0) {
        ModelVariable *current = stack[--stack_size];
        
        // 检查是否已经访问过
        if (current->index >= model->num_vars) continue;
        
        if (visited[current->index]) {
            // 已访问过，加入输出
            out[out_size++] = current;
            continue;
        }
        
        // 标记并压栈
        visited[current->index] = true;
        stack[stack_size++] = current;
        
        // 遍历子节点（inputs）
        u32 num_inputs = MODEL_OP_NUM_INPUTS(current->op);
        for (u32 i = 0; i < num_inputs; i++) {
            ModelVariable *input = current->inputs[i];
            if (input->index < model->num_vars && 
                !visited[input->index]) {
                // 检查是否已在栈中，避免重复
                bool in_stack = false;
                for (u64 k = 0; k < stack_size; k++) {
                    if (stack[k] == input) {
                        in_stack = true;
                        break;
                    }
                }
                if (!in_stack) {
                    stack[stack_size++] = input;
                }
            }
        }
    }
    
    // 分配最终输出到永久内存
    ModelProgram program;
    program.count = out_size;
    program.vars = push_array(arena, ModelVariable*, out_size);
    memcpy(program.vars, out, sizeof(ModelVariable*) * out_size);
    
    return program;
}
```

### 3.9 前向传播（Forward Pass）

按拓扑顺序执行每个操作，计算输出值：

```c
void program_compute(ModelProgram *program) {
    for (u32 i = 0; i < program->count; i++) {
        ModelVariable *var = program->vars[i];
        
        if (var->op == MODEL_OP_NONE || var->op == MODEL_OP_UNARY_START 
            || var->op == MODEL_OP_BINARY_START) {
            continue;  // 无操作或标记
        }
        
        ModelVariable *a = var->inputs[0];
        ModelVariable *b = var->inputs[1];
        
        switch (var->op) {
            case MODEL_OP_RELU:
                mat_relu(&var->value, &a->value);
                break;
            case MODEL_OP_SOFTMAX:
                mat_softmax(&var->value, &a->value);
                break;
            case MODEL_OP_ADD:
                mat_add(NULL, &var->value, &a->value, &b->value, true);
                break;
            case MODEL_OP_SUBTRACT:
                mat_sub(NULL, &var->value, &a->value, &b->value, true);
                break;
            case MODEL_OP_MATMUL:
                mat_mul(NULL, &var->value, &a->value, false, &b->value, false, true);
                break;
            case MODEL_OP_CROSS_ENTROPY:
                mat_cross_entropy(&var->value, &a->value, &b->value);
                break;
        }
    }
}
```

### 3.10 反向传播（Backward Pass）

#### 链式法则回顾

对于复合函数 f(g(x))：
df/dx = (df/dg) × (dg/dx)

#### Jacobian-Vector Product（雅可比-向量积）

对于向量函数 y = f(x)，梯度传播涉及计算：
∇_x L = J^T × ∇_y L

其中 J 是雅可比矩阵（偏导数矩阵）。

#### 梯度计算规则

**1. 加法操作**
```
f(A, B) = A + B
∂f/∂A = 1, ∂f/∂B = 1

因此：grad_A += current_grad
      grad_B += current_grad
```

**2. 减法操作**
```
f(A, B) = A - B
∂f/∂A = 1, ∂f/∂B = -1

因此：grad_A += current_grad
      grad_B -= current_grad
```

**3. 矩阵乘法**
```
G(A, B) = A × B
∂G/∂A = B^T, ∂G/∂B = A^T

因此：grad_A += current_grad × B^T
      grad_B += A^T × current_grad
```

**4. ReLU激活函数**

```c
void mat_relu_gradient(Matrix *in, Matrix *grad_in, Matrix *grad_out) {
    u64 size = in->rows * in->columns;
    for (u64 i = 0; i < size; i++) {
        if (in->data[i] > 0) {
            grad_in->data[i] += grad_out->data[i];
        }
        // 如果 <= 0，梯度为0，无需操作
    }
}
```

**5. Softmax激活函数**

Softmax的Jacobian不是对角矩阵（因为输出互相依赖）：

J[i,j] = S_i × (δ_ij - S_j)  // δ_ij是Kronecker delta

完整Jacobian向量积：

```c
bool mat_softmax_jacobian(Matrix *out, Matrix *grad_out) {
    // out是Softmax的输出（概率分布）
    // 返回的Jacobian矩阵是 N×N
    u64 size = MAX(out->rows, out->columns);
    
    Matrix jacobian = mat_create(&scratch, size, size);
    
    for (u64 i = 0; i < size; i++) {
        for (u64 j = 0; j < size; j++) {
            // J[i][j] = S_i × (δ_ij - S_j)
            f32 delta = (i == j) ? 1.0f : 0.0f;
            jacobian.data[i * size + j] = out->data[i] * (delta - out->data[j]);
        }
    }
    
    // grad_input = J^T × grad_output
    mat_mul(NULL, grad_in, &jacobian, true, grad_out, false, true);
    return true;
}
```

**6. Cross-Entropy损失函数**

设 P 为期望分布（one-hot），Q 为预测分布：

```
C = -Σ P_i × log(Q_i)

∂C/∂P_i = -log(Q_i)
∂C/∂Q_i = -P_i / Q_i
```

```c
void mat_cross_entropy_gradient(Matrix *P, Matrix *Q, 
                                 Matrix *grad_P, Matrix *grad_Q, 
                                 Matrix *incoming_grad) {
    u64 size = P->rows * P->columns;
    
    // ∂C/∂P = -log(Q) × grad
    if (grad_P) {
        for (u64 i = 0; i < size; i++) {
            grad_P->data[i] += -logf(Q->data[i]) * incoming_grad->data[i];
        }
    }
    
    // ∂C/∂Q = -P/Q × grad
    if (grad_Q) {
        for (u64 i = 0; i < size; i++) {
            if (Q->data[i] > 0) {
                grad_Q->data[i] += -(P->data[i] / Q->data[i]) * incoming_grad->data[i];
            }
        }
    }
}
```
#### 反向传播主循环

```c
void program_compute_grads(ModelProgram *program) {
    // 步骤1：清零所有梯度（除了参数的累积梯度）
    for (u32 i = 0; i < program->count; i++) {
        ModelVariable *var = program->vars[i];
        if (!(var->flags & MODEL_FLAG_REQUIRE_GRAD)) continue;
        if (var->flags & MODEL_FLAG_PARAMETER) continue;  // 参数不清零，累积梯度
        mat_clear(&var->gradient);
    }
    
    // 步骤2：设置初始梯度（损失函数对自身的偏导 = 1）
    ModelVariable *final_var = program->vars[program->count - 1];
    mat_fill(&final_var->gradient, 1.0f);
    
    // 步骤3：逆序遍历（从输出到输入）
    for (i64 i = program->count - 1; i >= 0; i--) {
        ModelVariable *var = program->vars[i];
        
        // 获取输入变量
        ModelVariable *a = var->inputs[0];
        ModelVariable *b = var->inputs[1];
        u32 num_inputs = MODEL_OP_NUM_INPUTS(var->op);
        
        // 提前检查：如果没有需要梯度的输入，跳过
        if (num_inputs == 1 && !(a->flags & MODEL_FLAG_REQUIRE_GRAD)) continue;
        if (num_inputs == 2 && 
            !(a->flags & MODEL_FLAG_REQUIRE_GRAD) && 
            !(b->flags & MODEL_FLAG_REQUIRE_GRAD)) continue;
        
        // 根据操作类型计算梯度
        switch (var->op) {
            case MODEL_OP_ADD:
                if (a->flags & MODEL_FLAG_REQUIRE_GRAD) {
                    mat_add(NULL, &a->gradient, &a->gradient, &var->gradient, false);
                }
                if (b->flags & MODEL_FLAG_REQUIRE_GRAD) {
                    mat_add(NULL, &b->gradient, &b->gradient, &var->gradient, false);
                }
                break;
                
            case MODEL_OP_SUBTRACT:
                if (a->flags & MODEL_FLAG_REQUIRE_GRAD) {
                    mat_add(NULL, &a->gradient, &a->gradient, &var->gradient, false);
                }
                if (b->flags & MODEL_FLAG_REQUIRE_GRAD) {
                    mat_sub(NULL, &b->gradient, &b->gradient, &var->gradient, false);
                }
                break;
                
            case MODEL_OP_MATMUL:
                if (a->flags & MODEL_FLAG_REQUIRE_GRAD) {
                    // grad_A += grad × B^T
                    mat_mul(NULL, &a->gradient, &var->gradient, false, &b->value, true, false);
                }
                if (b->flags & MODEL_FLAG_REQUIRE_GRAD) {
                    // grad_B += A^T × grad
                    mat_mul(NULL, &b->gradient, &a->value, true, &var->gradient, false, false);
                }
                break;
                
            case MODEL_OP_RELU:
                mat_relu_gradient(&a->value, &a->gradient, &var->gradient);
                break;
                
            case MODEL_OP_SOFTMAX:
                mat_softmax_jacobian(&var->value, &var->gradient, &a->gradient);
                break;
                
            case MODEL_OP_CROSS_ENTROPY:
                mat_cross_entropy_gradient(&a->value, &b->value, 
                                          &a->gradient, &b->gradient, 
                                          &var->gradient);
                break;
        }
    }
}
```

---

## 4：模型构建 | MNIST网络架构

### 4.1 MNIST数据集说明

- 输入：28×28像素灰度图像（展平为784维向量）
- 输出：10分类（数字0-9的概率分布）

### 4.2 网络架构


输入 (784×1)
    ↓
Dense1: W1 (16×784) × input + b1 (16×1)
    ↓ (ReLU)
Hidden1 (16×1)
    ↓
Dense2: W2 (16×16) × Hidden1 + b2 (16×1)
    ↓ (ReLU)
    ⊕ (残差连接: Hidden1 + Hidden2)
    ↓
Residual (16×1)
    ↓
Dense3: W3 (10×16) × Residual + b3 (10×1)
    ↓ (Softmax)
输出 (10×1) - 概率分布
    ↓
Cross-Entropy Loss


### 4.3 残差连接的重要性

残差连接允许梯度直接流向后层，这对于训练深层网络至关重要

这种架构只能用计算图实现，线性实现无法支持。

### 4.4 Xavier初始化

权重初始化对于训练稳定性非常重要：

```c
// Xavier/Glorot初始化
// 范围: [-sqrt(6/(fan_in + fan_out)), +sqrt(6/(fan_in + fan_out))]

f32 bound = sqrtf(6.0f / (rows + columns));
mat_uniform(out, -bound, bound);
```

---

## 第五部分：训练流程

### 5.1 训练超参数

```c
typedef struct TrainingDescription {
    Matrix *train_images;      // 训练图像 (60000×784)
    Matrix *train_labels;      // 训练标签 (60000×10, one-hot)
    Matrix *test_images;       // 测试图像
    Matrix *test_labels;       // 测试标签
    u32 epochs;                // 训练轮数
    u32 batch_size;            // 批次大小
    f32 learning_rate;         // 学习率
} TrainingDescription;
```

### 5.2 随机梯度下降（SGD）

```c
void model_train(ModelContext *model, TrainingDescription *desc) {
    u32 num_train = desc->train_images->rows;
    u32 input_size = desc->train_images->columns;
    
    // 准备训练顺序（打乱）
    u32 *training_order = push_array(&permanent, u32, num_train);
    for (u32 i = 0; i < num_train; i++) {
        training_order[i] = i;
    }
    
    u32 num_batches = num_train / desc->batch_size;
    
    // 训练循环
    for (u32 epoch = 0; epoch < desc->epochs; epoch++) {
        // 打乱训练顺序
        for (u32 i = num_train - 1; i > 0; i--) {
            u32 j = random_u64(&gen) % (i + 1);
            u32 temp = training_order[i];
            training_order[i] = training_order[j];
            training_order[j] = temp;
        }
        
        f32 epoch_cost = 0;
        
        for (u32 batch = 0; batch < num_batches; batch++) {
            // 步骤1：清零所有参数的梯度
            for (u32 i = 0; i < model->cost_program.count; i++) {
                ModelVariable *var = model->cost_program.vars[i];
                if (var->flags & MODEL_FLAG_PARAMETER) {
                    mat_clear(&var->gradient);
                }
            }
            
            // 步骤2：遍历批次中的每个样本
            for (u32 i = 0; i < desc->batch_size; i++) {
                u32 sample_idx = training_order[batch * desc->batch_size + i];
                
                // 复制输入
                memcpy(model->input.value.data,
                       desc->train_images->data + sample_idx * input_size,
                       input_size * sizeof(f32));
                
                // 复制期望输出
                memcpy(model->desired_output.value.data,
                       desc->train_labels->data + sample_idx * 10,
                       10 * sizeof(f32));
                
                // 前向传播
                program_compute(&model->forward_program);
                program_compute(&model->cost_program);
                
                // 反向传播
                program_compute_grads(&model->cost_program);
                
                // 累积损失
                epoch_cost += mat_sum(&model->cost.value);
            }
            
            // 步骤3：更新参数（批次结束后）
            f32 scale = desc->learning_rate / (f32)desc->batch_size;
            for (u32 i = 0; i < model->cost_program.count; i++) {
                ModelVariable *var = model->cost_program.vars[i];
                if (var->flags & MODEL_FLAG_PARAMETER) {
                    // θ_new = θ_old - lr × 平均梯度
                    mat_scale(&var->gradient, scale);
                    mat_sub(NULL, &var->value, &var->value, &var->gradient, true);
                }
            }
            
            printf("Epoch %u/%u | Batch %u/%u | Avg Cost: %.4f\r",
                   epoch + 1, desc->epochs, batch + 1, num_batches,
                   epoch_cost / (f32)((batch + 1) * desc->batch_size));
            fflush(stdout);
        }
        printf("\n");
    }
}
```

### 5.3 评估

```c
void model_evaluate(ModelContext *model, Matrix *test_images, 
                    Matrix *test_labels) {
    u32 num_test = test_images->rows;
    u32 correct = 0;
    f32 total_cost = 0;
    
    for (u32 i = 0; i < num_test; i++) {
        // 加载测试样本
        memcpy(model->input.value.data,
               test_images->data + i * 784,
               784 * sizeof(f32));
        memcpy(model->desired_output.value.data,
               test_labels->data + i * 10,
               10 * sizeof(f32));
        
        // 前向传播（使用cost program同时计算输出和损失）
        program_compute(&model->cost_program);
        
        total_cost += mat_sum(&model->cost.value);
        
        // 计算准确率：比较argmax(output) vs argmax(desired)
        u32 pred = mat_argmax(&model->output.value);
        u32 actual = mat_argmax(&model->desired_output.value);
        
        if (pred == actual) correct++;
    }
    
    printf("Test Accuracy: %.2f%% (%u/%u)\n", 
           100.0f * correct / num_test, correct, num_test);
    printf("Average Cost: %.4f\n", total_cost / num_test);
}
```

---

## 6：MNIST数据加载

### 6.1 Python数据准备脚本

使用`TensorFlow Datasets`下载并导出MNIST数据：

```python
import tensorflow_datasets as tfds
import tensorflow as tf

# 加载MNIST数据集
(ds_train, ds_test), info = tfds.load(
    'mnist',
    split=['train', 'test'],
    as_supervised=True,
    with_info=True
)

# 转换并保存为numpy数组文件
def ds_to_numpy(ds):
    images = []
    labels = []
    for img, label in tfds.as_numpy(ds):
        images.append(img.flatten())
        labels.append(label)
    return np.array(images, dtype=np.float32), np.array(labels, dtype=np.float32)

train_images, train_labels = ds_to_numpy(ds_train)
test_images, test_labels = ds_to_numpy(ds_test)

# 归一化：0-255 → 0-1
train_images = train_images / 255.0
test_images = test_images / 255.0

# 保存为二进制文件
train_images.tofile('train_images.bin')
train_labels.tofile('train_labels.bin')
test_images.tofile('test_images.bin')
test_labels.tofile('test_labels.bin')

# 保存形状信息
print(f"Train: {train_images.shape}, {train_labels.shape}")
print(f"Test: {test_images.shape}, {test_labels.shape}")
```

### 6.2 C语言数据加载

```c
Matrix load_mnist(Arena *arena, char *filename, u32 rows, u32 cols) {
    FILE *file = fopen(filename, "rb");
    
    Matrix mat = mat_create(arena, rows, cols);
    
    fread(mat.data, sizeof(f32), rows * cols, file);
    
    fclose(file);
    return mat;
}

// 加载示例
Matrix train_images = load_mnist(&permanent, "train_images.bin", 60000, 784);
Matrix train_labels = load_mnist(&permanent, "train_labels.bin", 60000, 10);
Matrix test_images = load_mnist(&permanent, "test_images.bin", 10000, 784);
Matrix test_labels = load_mnist(&permanent, "test_labels.bin", 10000, 10);
```

### 6.3 One-Hot编码转换

原始标签文件包含的是单个数字（如"4"），需要转换为one-hot编码：

```c
void convert_to_onehot(Matrix *labels, u32 num_samples) {
    // labels当前格式：60000×1，每行是一个数字
    // 目标格式：60000×10，每行是one-hot向量
    // 例如：4 → [0,0,0,0,1,0,0,0,0,0]
    
    u32 *raw_labels = (u32*)labels->data;  // 假设数据仍是u32
    
    for (u32 i = 0; i < num_samples; i++) {
        u32 digit = raw_labels[i];
        labels->data[i * 10 + digit] = 1.0f;
    }
}
```

---

## 7：完整训练结果

### 7.1 训练配置

```c
TrainingDescription desc = {
    .train_images = &train_images,
    .train_labels = &train_labels,
    .test_images = &test_images,
    .test_labels = &test_labels,
    .epochs = 3,
    .batch_size = 50,
    .learning_rate = 0.1f
};
```

### 7.2 训练过程输出

训练开始时损失函数较高（约2.3），随着训练进行逐渐下降：
- Epoch 1/3结束时：损失已显著降低
- `Epoch 3/3结束时：损失收敛到较低水平`

### 7.3 测试

```
Test Accuracy: 89.xx% (约8900/10000)
Average Cost: 0.xxxx
```

### 7.4 输出验证

训练前：模型输出接近均匀分布（随机初始化）

训练后：对于手写数字"4"的输入
- `模型以约89%置信度预测为"4"`
- 其他类别的概率显著降低

---

## ⭕8：设计与优化

### 8.1 `Arena Allocator`的优势

1. **避免内存碎片化**：一次性分配大块内存
2. **简化内存释放**：只需重置arena指针，无需逐个释放
3. **提升分配性能**：O(1)时间复杂度

### 8.2 `计算图` vs 线性实现

计算图实现的优势：
- 支持任意复杂的操作连接
- 支持残差连接
- 梯度计算自动完成

### 8.3 拓扑排序算法选择

使用`DFS`而非Kahn算法的原因：
- 更容易处理图的子集（损失图是输出图的超集）
- 只需一次遍历

### 8.4 矩阵乘法的四种转置变体

针对不同转置组合分别优化，通过调整循环顺序获得最佳缓存局部性：
- `mat_mul_nn`：标准乘
- `mat_mul_nt`：B转置
- `mat_mul_tn`：A转置
- `mat_mul_tt`：都转置

### 8.5 参数梯度不清零

在训练循环中：
- **非参数变量**：每次迭代清零梯度
- **参数变量**：累积整个batch的梯度，结束后平均并更新

这种设计`支持批量梯度累积`，适用于大batch训练。

---

## 9：改进方向

| 改进项 | 说明 |
|--------|------|
| **动量优化** | 添加momentum到SGD中加速收敛 |
| **学习率衰减** | 随训练进行降低学习率 |
| **Adam优化器** | 实现自适应矩估计 |
| **Batch Normalization** | 加速训练、提升稳定性 |
| **卷积层** | 处理图像的空间结构（当前是全连接层） |
| **Dropout** | 防止过拟合 |
| **GPU加速** | 将矩阵运算移至GPU执行 |
| **内存优化** | 中间变量复用内存 |
| **错误处理** | 添加完善的错误检查 |

---

## 总结

本项目完整实现了一个机器学习框架的三个核心组件：

1. **数学层**：矩阵运算、激活函数、损失函数
2. **自动微分层**：计算图构建、拓扑排序、反向传播
3. **胶水层**：模型定义、训练循环、参数更新

最终在MNIST数据集上达到约89%的准确率，验证了框架的正确性和教育价值。代码量`约1000行`，便于理解深度学习框架内部工作原理