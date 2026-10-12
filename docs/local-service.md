# 本地管理服务

仅启动 xsos 及其管理 Web，不自动安装或启动采集 Client。
开发服务监听 `http://127.0.0.1:18105`，使用当前认证和 loopback HTTP Cookie。
不用于生产部署，不修改系统服务，也不占用 xscs 本地管理服务的 18104 端口。

在仓库根执行以下命令：安装锁定的 Web 构建依赖，构建 Web 和开发二进制，显式初始化私有数据，随后启动、查看就绪状态并停止本地进程。`init` 只执行一次；已有实例不再初始化。

```bash
npm --prefix web ci
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=2 web/node_modules/.bin/xcss-build-server --config xcss-web-build.json --mode development --no-install
read -r -s -p 'Initial administrator password: ' initial_password; printf '\n'
printf '%s\n' "$initial_password" | node scripts/local-service.mjs init
unset initial_password
node scripts/local-service.mjs start
node scripts/local-service.mjs status
node scripts/local-service.mjs stop
```

初始化只从 stdin 接收管理员密码，用户名为 admin。配置不保存密码或业务密钥；init 自动生成实例密钥并写入 `.runtime/local-service/data/secrets.json`（0600）。该文件与数据库共同保留，start/check 不补建或替换它。不要手工生成、导出或轮换此内部密钥来排除启动错误。
数据库位于 `.runtime/local-service/data/xsos.sqlite3`，有界轮转日志位于
`.runtime/local-service/data/logs/`。所有运行状态由 Git 忽略，重复启动复用原有配置、数据库和凭据。
重新构建二进制前先停止本服务；默认使用内嵌的管理页面。

Web 与 Server 使用 xcss 同一构建入口。默认二进制自带管理页面；本地启动默认使用内嵌资源；执行 `node scripts/local-service.mjs start --directory-web` 显式选择开发目录 `web/dist`，修改后重新构建 Web 即生效，无需重编译 Rust。也可运行 Vite 开发服务器获得源码热更新。正式 source-bound 二进制拒绝目录资源模式。

构建产物默认位于 `target/x86_64-unknown-linux-gnu/debug/`。若设置自定义 `CARGO_TARGET_DIR`，启动/状态/停止时使用 `XSOS_LOCAL_SERVER_BINARY` 指定同一绝对二进制路径。
