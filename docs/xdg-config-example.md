# XDG 配置目录支持 / XDG Config Directory Support

## 概述 / Overview

my-http-server 现在支持 XDG Base Directory 规范，允许在标准位置存储配置文件。

my-http-server now supports the XDG Base Directory specification, allowing configuration files to be stored in standard locations.

## 配置文件位置优先级 / Config File Precedence

配置按以下顺序加载（后面的会覆盖前面的）：

Configuration is loaded in the following order (later overrides earlier):

1. **内建预设** / Built-in defaults
   - 嵌入在二进制文件中的默认配置
   - Default configuration embedded in the binary

2. **XDG 配置目录** / XDG config directory
   - **Linux/macOS**: `$XDG_CONFIG_HOME/my-http-server/cofg.yaml`
     - 默认：`~/.config/my-http-server/cofg.yaml`
   - **Windows**: `%LOCALAPPDATA%\my-http-server\config\cofg.yaml`
     - 例如：`C:\Users\YourName\AppData\Local\my-http-server\config\cofg.yaml`

3. **本地配置文件** / Local config file
   - `./cofg.yaml` 或通过 `--config-path` 指定
   - `./cofg.yaml` or specified via `--config-path`

4. **环境变量** / Environment variables
   - 前缀 `MYHTTP_`，分隔符 `_`
   - Prefix `MYHTTP_`, separator `_`
   - 例如 / Example: `MYHTTP_ADDRS_IP=192.168.1.1`

5. **命令行参数** / CLI arguments (最高优先级 / highest priority)
   - 例如 / Example: `--ip 0.0.0.0 --port 9090`

## 使用示例 / Usage Examples

### Linux/macOS

```bash
# 创建 XDG 配置目录
# Create XDG config directory
mkdir -p ~/.config/my-http-server

# 创建配置文件
# Create config file
cat > ~/.config/my-http-server/cofg.yaml << 'EOF'
addrs:
  ip: localhost
  port: 8080

public_path: ./public/

templating:
  hot_reload: false
EOF

# 运行服务器（会自动使用 XDG 配置）
# Run server (will automatically use XDG config)
./my-http-server
```

### Windows (PowerShell)

```powershell
# 创建配置目录
# Create config directory
New-Item -ItemType Directory -Force -Path "$env:LOCALAPPDATA\my-http-server\config"

# 创建配置文件
# Create config file
@"
addrs:
  ip: localhost
  port: 8080

public_path: ./public/

templating:
  hot_reload: false
"@ | Out-File -Encoding UTF8 "$env:LOCALAPPDATA\my-http-server\config\cofg.yaml"

# 运行服务器（会自动使用 LOCALAPPDATA 配置）
# Run server (will automatically use LOCALAPPDATA config)
.\my-http-server.exe
```

## 配置覆盖示例 / Override Examples

### 使用本地配置覆盖 XDG 配置
### Override XDG config with local config

```bash
# XDG 配置设置 port: 8080
# XDG config sets port: 8080

# 创建本地配置覆盖端口
# Create local config to override port
cat > ./cofg.yaml << 'EOF'
addrs:
  port: 9090
EOF

# 运行服务器 - 将使用端口 9090
# Run server - will use port 9090
./my-http-server
```

### 使用环境变量覆盖
### Override with environment variables

```bash
# XDG 配置存在，但想临时改变 IP
# XDG config exists, but want to temporarily change IP
export MYHTTP_ADDRS_IP=0.0.0.0
export MYHTTP_ADDRS_PORT=3000

./my-http-server
```

### 使用 CLI 参数覆盖所有配置
### Override all config with CLI args

```bash
# 所有配置源都存在，但 CLI 参数优先级最高
# All config sources exist, but CLI args have highest priority
./my-http-server --ip 127.0.0.1 --port 8888
```

### 跳过配置文件
### Skip config files

```bash
# 跳过 XDG 和本地配置文件，仅使用内建默认值
# Skip XDG and local config files, use only built-in defaults
./my-http-server --no-config
```

## 调试配置 / Debugging Config

如果不确定配置从哪里加载，可以启用调试日志：

If you're unsure where config is loaded from, enable debug logging:

```bash
# Linux/macOS
RUST_LOG=debug ./my-http-server

# Windows (PowerShell)
$env:RUST_LOG="debug"
.\my-http-server.exe
```

调试输出会显示：
Debug output will show:

- `Loading config from XDG path: ...` - 如果使用 XDG 配置
- `Loading config from: ./cofg.yaml` - 如果使用本地配置
- 应用的环境变量和 CLI 覆盖

## 优势 / Benefits

1. **跨平台一致性** / Cross-platform consistency
   - Linux/macOS 使用 XDG 标准
   - Windows 使用 LOCALAPPDATA 标准

2. **用户级配置** / User-level configuration
   - 不需要 root/管理员权限
   - No root/admin permissions needed

3. **多实例支持** / Multiple instance support
   - 用户配置 + 项目配置
   - User config + project config

4. **向后兼容** / Backward compatible
   - 现有的 `./cofg.yaml` 继续工作
   - Existing `./cofg.yaml` continues to work

## 注意事项 / Notes

- XDG 配置是**可选的** / XDG config is **optional**
  - 如果文件不存在，会跳过此层
  - If file doesn't exist, this layer is skipped

- `--no-config` 会跳过 XDG 和本地配置
  - `--no-config` skips both XDG and local config
  - 仅使用内建默认值、环境变量和 CLI 参数
  - Uses only built-in defaults, env vars, and CLI args

- 配置合并是**深度合并** / Config merge is **deep merge**
  - 你不需要在 XDG 配置中复制所有字段
  - You don't need to copy all fields in XDG config
  - 只需覆盖你想改变的部分
  - Only override the parts you want to change

## 相关文档 / Related Documentation

- [配置映射文档](./config-templating-map.md)
- [开发者指南](./developer-guide.md)
- [架构文档](../architecture.md)
