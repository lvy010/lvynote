
前置: 
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/5e5af52f121847e184f0b4b4c21aadda.jpg)
在映射函数矫正几何失真的过程中，如果映射点不落在原有像素点上，需要用重采样来估算它的数值。

不同插值方法就是不同的“取值策略”。

## olive.c 图形库

olive.c 是一个纯 CPU 端的 C 语言图形库，特点：

- **单头文件**：非常便携，可以部署在任何设备上
- **`不依赖图形 API`**：不依赖 OpenGL、X11 等，无需图形界面即可运行
- **工作方式**：接收一块内存区域，在其中绘制图元，输出原始像素数据，用户自行决定如何显示（窗口、文件、终端等）
- **`支持多种渲染后端`**：HTML5 Canvas、SDL、终端 256 色


## 问题：当前纹理映射导致的像素化

### 实现原理

假设有一个 4×4 的小纹理，要映射到 3×2 的目标矩形：

```c
// 当前实现
for (each pixel in target rectangle) {
    // 1. 将目标像素坐标归一化到 [0,1]
    float nx = x / target_width;
    float ny = y / target_height;
    
    // 2. 映射到纹理坐标
    int tex_x = (int)(nx * texture_width);
    int tex_y = (int)(ny * texture_height);
    
    // 3. 取纹理像素颜色
    color = texture[tex_y][tex_x];
}
```

**问题**：当放大纹理时，会直接"复制"像素，造成明显的锯齿和像素化。

### 示例

- 原始纹理大小：120×120 像素
- 拉伸后显示更大，因此产生明显的像素化

## 解决：双线性插值 (Bilinear Interpolation)

### 原理

对于目标矩形内的任意点 $(p_x, p_y)$，它可能落在纹理四个相邻像素形成的矩形中：

```
    Q2 ───────── Q3
      │    P     │
      │   .      │
    Q0 ───────── Q1
```

1. **第一次线性插值**：在上下边界分别做插值
   - 上边界：从 Q0 到 Q1，使用 $p_x$ 权重
   - 下边界：从 Q2 到 Q3，使用 $p_x$ 权重

2. **第二次线性插值**：在垂直方向做插值
   - 从上边界插值结果到下边界插值结果，使用 $p_y$ 权重

### 纯整数实现（避免浮点数）

关键洞察：`x / width` 得到索引，`x % width` 得到像素内位置（相当于小数部分）。

```c
int index = x / width;        // 像素索引
int frac = x % width;         // 像素内位置（0 到 width-1）

// 判断在像素的哪一侧
bool is_left = (frac < width / 2);
```

这样可以在不进行浮点运算的情况下完成插值计算。

## code

### 1. 添加混合两个颜色的函数

```c
// 混合两个颜色
static Color mix_color2(Color c1, Color c2, size_t u1, size_t u2)
{
    // u1 + u2 可能是分母
    size_t u = u2 + (u1 * (c2.argb - c1.argb) / u2);
    return argb_color(u);
}
```

### 2. 双线性插值主函数

```c
static Color blit_sprite_blinear(
    uint32_t *canvas, Rect dst, Size canvas_size,
    Sprite *sprite, Rect src)
{
    Rect nr = normalize_rect(dst, canvas_size);
    if (!nr.width || !nr.height) return 0;
    if (!normalize_rect(src, sprite->size)) return 0;

    Rect sr = normalize_rect(src, sprite->size);
    // sr.x1, sr.y1, sr.x2, sr.y2 是纹理坐标边界

    for (int y = sr.y1; y < sr.y2; y++) {
        for (int x = sr.x1; x < sr.x2; x++) {
            // 计算相对于纹理边界的偏移
            int lx = x - sr.x1;
            int ly = y - sr.y1;
            
            // 计算在目标矩形中的位置（归一化坐标的小数部分）
            int nx = lx * sprite->size.width / (sr.x2 - sr.x1);
            int ny = ly * sprite->size.height / (sr.y2 - sr.y1);
            
            int W = sprite->size.width;
            int H = sprite->size.height;
            
            // 计算像素内位置
            int frac_x = nx % W;
            int frac_y = ny % H;
            
            // 计算纹理索引
            int ix = nx / W;
            int iy = ny / H;
            
            // 计算四个顶点的纹理坐标
            int x1 = ix;
            int x2 = (frac_x >= 0 && ix + 1 < W) ? ix + 1 : ix;
            
            int y1 = iy;
            int y2 = (frac_y >= 0 && iy + 1 < H) ? iy + 1 : iy;
            
            // 获取四个颜色
            Color q0 = sprite->data[iy * W + ix];
            Color q1 = sprite->data[iy * W + (ix + 1 < W ? ix + 1 : ix)];
            Color q2 = sprite->data[(iy + 1 < H ? iy + 1 : iy) * W + ix];
            Color q3 = sprite->data[(iy + 1 < H ? iy + 1 : iy) * W + (ix + 1 < W ? ix + 1 : ix)];
            
            // 计算插值权重
            int px = frac_x + W / 2;
            int py = frac_y + H / 2;
            
            // 第一次插值：水平方向
            Color h1 = mix_color2(q0, q1, px, W);
            Color h2 = mix_color2(q2, q3, px, W);
            
            // 第二次插值：垂直方向
            Color result = mix_color2(h1, h2, py, H);
            
            canvas[dst.y1 + ly * dst.height / (sr.y2 - sr.y1)]
                  [dst.x1 + lx * dst.width / (sr.x2 - sr.x1)] = result;
        }
    }
}
```

### 3. 优化：处理边界情况

当目标像素位于纹理边缘时：

```c
// 边界检查
int x1 = ix;
int x2 = (ix + 1 < W) ? ix + 1 : ix;

int y1 = iy;
int y2 = (iy + 1 < H) ? iy + 1 : iy;
```

如果邻居像素不存在，使用当前像素颜色作为替代（相当于"外推"边缘颜色）。

### 4. 权重计算

```c
// 判断在像素的哪一侧
int px = frac_x + W / 2;
int py = frac_y + H / 2;

// 等效于：
// 如果在左侧（像素左半边），权重增加 W/2
// 如果在右侧（像素右半边），权重减少 W/2
// 这样确保插值方向正确
```

##  Bug：mix_color 实现错误

### 原始 mix_color3 实现（用于三角形渐变）

```c
Color mix_color3(Color c1, Color c2, Color c3, size_t u1, size_t u2, size_t u3)
{
    size_t u = u2 + u1*(c2.argb - c1.argb)/u3;
    u += u1 + u2 + u3;
    // 实际使用 (u1 + u2 + u3) 作为分母
}
```

### 问题

```c
Color mix_color2(Color c1, Color c2, size_t u1, size_t u2)
{
    size_t u = u2 + (u1 * (c2.argb - c1.argb) / u2);
    return argb_color(u);
}
```

如果 $c_1$ 颜色值是 100，$c_2$ 是 200，插值比例应该是 0.3（更接近 $c_1$）：

- 错误计算：`100 + 0.3 * (200-100) = 130`（30% 在 c1 和 c2 之间）
- 但实际上是颜色值相加，不是比例计算

### 正解

在 UV 坐标系中，距离某个顶点越近，该顶点的颜色权重越大。因此：

```c
// 正确：距离越近，权重越大，混合时该颜色贡献越多
Color mix_color2(Color c1, Color c2, size_t u1, size_t u2)
{
    size_t weight1 = u2;  // 距离远，权重小
    size_t weight2 = u1;  // 距离近，权重小 → 等等，这个理解有问题
    
    // 正确理解：
    // 假设 u1 是距离 c1 的"逆距离"，u2 是距离 c2 的"逆距离"
    // 如果 u1 = 70, u2 = 30，意味着点更靠近 c1
    // 所以结果应该更多是 c1 的颜色
    size_t u = (c1.argb * u2 + c2.argb * u1) / (u1 + u2);
}
```

## 构建系统改进

### 支持选择性构建

```makefile
# 构建所有演示
demos:
    ./build.py demos

# 构建单个演示
demos squish:
    ./build.py demos squish

# 按平台构建
demos squish SDL:
    ./build.py demos squish SDL

# 按平台构建所有该平台的演示
demos SDL:
    ./build.py demos SDL
```

构建脚本改进：

```python
def build(platform, name):
    # 支持并行构建
    processes = []
    for platform in platforms:
        p = subprocess.Popen([...])
        processes.append(p)
    
    # 等待所有进程完成
    for p in processes:
        p.wait()
```

### 添加测试

```c
test_case("blit_sprite_blinear", test_blit_sprite_blinear);

// 测试函数
static void test_blit_sprite_blinear(Canvas *canvas)
{
    Sprite *src = &olive.texture_back;
    
    // 创建放大后的目标（10倍）
    Canvas *canvas1 = ol_cnew(Canvas, 1);
    *canvas1 = make_canvas(src->size.width * 10, src->size.height * 10);
    
    ol_fill(*canvas1, ARGBA_ZERO);
    
    // 使用无插值的版本
    ol_blit_sprite_copy(canvas1,
        ol_pt(0, 0),
        ol_sz(canvas1->size.width, canvas1->size.height),
        src,
        ol_sz(src->size.width, src->size.height));
    
    // 使用有插值的版本（右侧）
    ol_blit_sprite_blinear(canvas1,
        ol_pt(canvas1->size.width/2, 0),
        ol_sz(canvas1->size.width/2, canvas1->size.height),
        src,
        ol_sz(src->size.width, src->size.height));
}
```

## 数学细节

### 纹理坐标映射

```
目标像素位置 (x, y)
    ↓
偏移量: lx = x - src.x1, ly = y - src.y1
    ↓
归一化: nx = lx * sprite_width / src_width
        ny = ly * sprite_height / src_height
    ↓
纹理索引: ix = nx / W, iy = ny / H
像素内偏移: frac_x = nx % W, frac_y = ny % H
```

### 插值权重

```c
// px 表示在当前像素内的位置偏移
int px = frac_x + W/2;  // W/2 是像素中心
int py = frac_y + H/2;

// px 范围：0 到 W
// py 范围：0 到 H
// px=0 表示在最左侧
// px=W 表示在最右侧
// px=W/2 表示正好在中心
```

### 为什么 +W/2

```
像素位置分布：
0 ─── W/2 ─── W
|---左--|--右--|

如果 frac_x = 0（正好在整数索引），加上 W/2 后：
- px = W/2，正好在中心
- 插值会均匀混合左右两个像素

如果 frac_x 很小（接近左边界），加上 W/2 后：
- px < W/2，偏向左边
- 最终颜色更接近左侧像素
```

## 对比

| 方式 | 效果 | 性能 |
|------|------|------|
| 最近邻（无插值） | 明显像素化 | 快 |
| 双线性插值 | 平滑过渡 | 较慢（但 CPU 可接受） |

`双线性插值在纹理放大时能有效消除明显的像素锯齿`，但在纹理缩小时效果可能不那么明显。

## 附录

- **仿射变换** (Affine Transformation)：双线性插值可扩展到任意维度
- **三线性插值** (Trilinear Interpolation)：3D 版本，需要在立方体的所有边和面上进行插值
- **各向异性过滤** (Anisotropic Filtering)：更高级的纹理滤波技术