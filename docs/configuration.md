# 配置

默认文件 `/etc/isarmg/xsos/config.json`，归 `isarmg-xsos:isarmg-xsos` 所有，文件 `0600`、父目录 `0700`。默认文件缺失可使用环境变量和默认值；显式 `--config` 文件缺失、不安全或配置错误则失败。

优先级：CLI > `XSOS_` 环境变量 > JSON > 默认值。未知、重复和错误字段会被拒绝，无旧变量别名。

| JSON | 环境变量 | 默认值 |
|---|---|---|
| bind | XSOS_BIND | 127.0.0.1:18105 |
| data_dir | XSOS_DATA_DIR | /var/lib/isarmg/xsos |
| development | XSOS_DEVELOPMENT | false |
| log_level | XSOS_LOG_LEVEL | info（error/warn/info/debug/trace） |
| trusted_proxies | XSOS_TRUSTED_PROXIES | []（环境值为 JSON 数组，元素是 IP/CIDR） |

`--config`、`--data-dir`、`--bind`、`--development true|false` 为全局参数，可放在子命令前后。development=true 仅允许 loopback 监听。

所有内部状态都在 data_dir：`xsos.sqlite3`、`secrets.json`、`logs/` 和锁文件。无单独 database_url。init 在空实例中自动生成业务密钥并以 0600 原子写入 secrets.json；run/check 只读，缺失或损坏时拒绝，不自动补建。升级保留整个状态目录。管理员密码只在 init 或 admin reset-password 的 stdin 中传入。

日志使用公共 xcss UTC 结构化日志与 logs/ 轮转。生产 HTTPS 由入口负责，管理员认证继续检查 Host/Origin 并设置 Secure Cookie。trusted_proxies 仅控制客户端 IP：默认忽略转发头，只有可信直连代理才按链解析；不会改变 TLS 或认证语义。

`config validate` 只检查配置，无需初始化；`check` 离线检查状态；`status` 查询在线 readiness。

遥测和保留窗口参数仍可通过同名 XSOS_ 大写变量覆盖；development_web_dir / XSOS_DEVELOPMENT_WEB_DIR 仅允许开发构建使用。密钥与数据库属于同一实例，不应单独替换或丢弃。
