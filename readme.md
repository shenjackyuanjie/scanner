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
