# IP Filter 功能說明 / IP Filter Documentation

## 概述 / Overview

本伺服器已整合 [actix-ip-filter](https://github.com/jhen0409/actix-ip-filter) 中介軟體，可根據 IP 位址過濾請求。支援 glob 模式匹配。

This server has integrated the [actix-ip-filter](https://github.com/jhen0409/actix-ip-filter) middleware to filter requests based on IP addresses. Supports glob pattern matching.

## 設定 / Configuration

在 `cofg.yaml` 中設定 IP 過濾器：

Configure the IP filter in `cofg.yaml`:

```yaml
middleware:
  ip_filter:
    enable: false # 啟用/停用 IP 過濾器 / Enable/disable IP filter
    allow:# 允許清單（白名單模式）/ Allow list (whitelist mode)
      # - 127.0.0.1
      # - 192.168.1.*
    block:# 封鎖清單（黑名單模式）/ Block list (blacklist mode)
      # - 10.0.0.*
```

## 使用方式 / Usage

### 白名單模式 / Whitelist Mode

只允許特定 IP 存取：

Only allow specific IPs to access:

```yaml
middleware:
  ip_filter:
    enable: true
    allow:
      - 127.0.0.1 # 允許本機存取 / Allow localhost
      - 192.168.1.* # 允許區域網路 192.168.1.x / Allow LAN 192.168.1.x
      - 172.??.6*.12 # glob 模式匹配 / glob pattern matching
```

### 黑名單模式 / Blacklist Mode

封鎖特定 IP：

Block specific IPs:

```yaml
middleware:
  ip_filter:
    enable: true
    block:
      - 10.0.0.* # 封鎖整個 10.0.0.x 網段 / Block entire 10.0.0.x subnet
      - 192.168.1.100 # 封鎖特定 IP / Block specific IP
```

### 混合模式 / Mixed Mode

可同時使用白名單與黑名單：

Can use both allow and block lists together:

```yaml
middleware:
  ip_filter:
    enable: true
    allow:
      - 192.168.* # 允許所有 192.168.x.x
    block:
      - 192.168.1.100 # 但封鎖 192.168.1.100
```

## Glob 模式 / Glob Patterns

支援的萬用字元 / Supported wildcards:

- `*`: 匹配任意數量的字元 / Match any number of characters
- `?`: 匹配單一字元 / Match a single character

範例 / Examples:

- `192.168.1.*` - 匹配 192.168.1.0 到 192.168.1.255
- `192.168.?.1` - 匹配 192.168.0.1, 192.168.1.1, 192.168.2.1 等
- `172.??.6*.12` - 匹配複雜模式

## 注意事項 / Notes

1. **預設行為** / Default Behavior:
   - 當 `enable: false` 時，不進行任何過濾 / No filtering when `enable: false`
   - 當僅設定 `allow` 時，未列出的 IP 將被拒絕 / When only `allow` is set, unlisted IPs are rejected
   - 當僅設定 `block` 時，未列出的 IP 將被允許 / When only `block` is set, unlisted IPs are allowed

2. **中介軟體順序** / Middleware Order:
   - IP 過濾器在 HTTP 基本認證之後執行 / IP filter runs after HTTP basic authentication
   - 確保安全性檢查的層次性 / Ensures layered security checks

3. **效能考量** / Performance:
   - IP 過濾器使用條件中介軟體，停用時無額外開銷 / Uses conditional middleware, no overhead when disabled
   - 適合用於簡單的存取控制 / Suitable for simple access control

## 測試 / Testing

啟用 IP 過濾器後，可使用以下指令測試：

After enabling IP filter, test with:

```bash
# 測試允許的 IP / Test allowed IP
curl http://127.0.0.1:8080/

# 從其他機器測試（若在封鎖清單中應被拒絕）
# Test from another machine (should be rejected if in block list)
curl http://your-server-ip:8080/
```

被封鎖的請求將收到 HTTP 403 Forbidden 回應。

Blocked requests will receive an HTTP 403 Forbidden response.

## See also

- IP filter implementation summary: ./ip-filter-implementation-summary.md
- Request flow: ./request-flow.md
- Config ↔ Template map: ./config-templating-map.md
