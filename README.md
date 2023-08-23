## ContriTrack
一个非常方便的工具，统计个人开源贡献情况

### 相关操作
``` shell
[xxxxxx]:/home/rust/ContriTrack/target/release# contritrack -h
Usage: contritrack [OPTIONS] --duration <DURATION>

Options:
  -a, --author <AUTHOR>      提交者
  -s, --state <STATE>        Pull Request的状态
                               merged 已合入
                               open   提交未合入
  -d, --duration <DURATION>  近 x 天提交数据
  -e, --exc                  是否保存为excel文件
  -i, --inventory <FILE>     批量处理提交者pr
  -h, --help                 Print help
  -V, --version              Print version

```


#### 获取gitee账号名为 zyljy， pr 为 open 状态 且 距今40天内的统计
```shell
[xxxxxx]:/home/rust/ContriTrack/target/release# contritrack -a zyljy -s open -d 40 
  zyljy, doc, openeuler/docs, https://gitee.com/openeuler/docs/pulls/5853, 2023-07-20 17:31:48
  zyljy, doc, openeuler/docs, https://gitee.com/openeuler/docs/pulls/5522, 2023-07-17 14:58:12
```

#### --exc 将统计数据保存到 excel 中
```shell
[xxxxxx]:/home/rust/ContriTrack/target/release# contritrack -a zyljy -s open -d 40 --exc
请等待....
```

#### --file=<path> 通过清单文件，批量处理数据
```shell
[xxxxxx]:contritrack -s open -d 40 -i users.txt
...
```
