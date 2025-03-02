# 某个扫描工具

usage: read the cli

```text
Hello, world!
Usage: scans.exe [OPTIONS] --url <URL> --ip_path <IP_PATH>

Options:
  -t, --threads <THREADS>                  线程数量 ( 建议开到 10+ 别太大 都是真线程 ) [default: 4]
  -u, --url <URL>                          源站, 包含 https:// 或 http://
  -i, --ip_path <IP_PATH>                  可能 ip 文件夹/文件
  -d, --db_path <DB_PATH>                  数据库文件路径 [default: main.sqlite]
  -v, --verbose                            是否开启 debug
  -m, --max_ip <MAX_IP_COUNT>              单次搜索的最大 ip 数量 [default: 100]
  -c, --compare <COMPARE>                  对比文件路径 [default: check.src]
  -o, --timeout <TIMEOUT>                  超时时间 (默认 1s) [default: 1]
  -w, --worker_interval <WORKER_INTERVAL>  每次 搜索 worker 启动间隔 (ms) [default: 200]
  -h, --help                               Print helpPrint help
```

## 用途(至少作者的)

扫描某个套了 Cloudflare 的服务器 ip

> 其实我应该写一下自动从 xxx 获取 ip 的
> 不过懒了, 就没写

所以这玩意是支持 "分布式部署"的

只要自己处理一下ip列表, 然后给每个节点分发就行
