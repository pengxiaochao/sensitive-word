# 敏感词过滤系统

基于Rust实现的高性能敏感词过滤系统，使用AC自动机（Aho-Corasick算法）进行高效的敏感词匹配。

## 功能特点

- 高性能：基于AC自动机算法，提供高效的敏感词匹配
- 持久化：支持将敏感词索引保存到文件，重启服务时快速加载
- 自动重建：当索引文件不存在时，自动从原始词典文件构建索引
- RESTful API：提供HTTP接口，方便集成到各种系统
- 命令行配置：支持通过命令行参数自定义服务配置

## 项目结构

```
sensitive-word/
├── Cargo.toml         # 项目依赖配置
├── src/
│   ├── ac.rs          # AC自动机实现
│   ├── filter.rs      # 敏感词过滤服务
│   └── main.rs        # 主程序入口和API实现
└── models/
    ├── ac_index.bin   # 生成的敏感词索引文件(自动生成)
    └── source/
        └── dic.txt    # 原始敏感词列表文件
```

## 安装说明

### 前提条件

- 安装Rust开发环境（[rustup](https://rustup.rs/)）

### 构建项目

```bash
# 克隆项目
git clone <repository-url>
cd sensitive-word

# 构建项目（开发版本）
cargo build

# 构建项目（发布版本）
cargo build --release
```

## 使用方法

### 准备敏感词列表

默认敏感词列表取自[Sensitive_topic_identification](https://github.com/llzbat/Sensitive_topic_identification)

在`models/source/dic.txt`文件中添加敏感词，每行一个词：

```
敏感词1
敏感词2
不良内容
违禁品
赌博
毒品
```

### 启动服务

```bash
# 使用默认配置启动
cargo run --release

# 自定义配置启动
cargo run --release -- --host 0.0.0.0 --port 8084 --path ./

# 启动时重建索引
cargo run --release -- --rebuild
```

### 命令行参数

| 参数 | 短选项 | 描述 | 默认值 |
|------|--------|------|--------|
| `--port` | `-p` | 服务监听端口 | 3000 |
| `--host` | `-h` | 服务绑定地址 | 127.0.0.1 |
| `--path` | `-p` | 模型目录路径 | ./ |
| `--rebuild` | `-r` | 启动时重建索引 | false |

## API 文档

### 1. 过滤文本

**请求**:

```
POST /filter
Content-Type: application/json

{
  "text": "这是一个包含敏感词1的文本"
}
```

**响应**:

```json
{
  "filtered": "这是一个包含****的文本"
}
```

### 2. 检查文本是否包含敏感词

**请求**:

```
POST /check
Content-Type: application/json

{
  "text": "这是一个包含敏感词1和敏感词2的文本"
}
```

**响应**:

```json
{
  "contains_sensitive": true,
  "words": ["敏感词1", "敏感词2"]
}
```

### 3. 重建索引

**请求**:

```
POST /rebuild
```

**响应**:

```json
{
  "status": "Index rebuilt successfully"
}
```

### 4. 检查服务状态

**请求**:

```
GET /status
```

**响应**:

```json
{
  "status": "Service is running"
}
```

## 许可证
在 MIT 许可证下发布。
