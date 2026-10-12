# xsos

硬件监控服务。

## 功能

提供管理台、实例配对、硬件遥测接收与历史查询。

## 平台

Server 支持 Linux x86_64 GNU，发行包内嵌管理页面，使用 systemd 管理服务。

## 快速部署

解压发行包，按 config 中的示例准备 JSON，执行 `sudo ./install.sh setup /absolute/config.json [USERNAME]`。管理员名默认 admin，密码从标准输入读取；实例密钥由 init 自动生成。完成后按输出启动服务。升级执行 `sudo ./install.sh upgrade`。

## 源码编译

使用仓库固定的 Rust 1.99.0、Node.js 26.7.0：

```sh
npm ci --prefix web
web/node_modules/.bin/xcss-build-server --config xcss-web-build.json --mode development --no-install
```

[完整文档](docs/README.md)
