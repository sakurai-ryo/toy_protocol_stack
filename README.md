## 概要
以下の本を元に色々実験するリポジトリ <br />
https://cha-shu00.hatenablog.com/entry/2020/12/30/124948

```
$ ./setup.sh

# terminal 1
$ sudo ip netns exec host2 nc -l 10.0.1.1 40000

# terminal 2
$ cargo build --examples
$ sudo ip netns exec host1 target/debug/examples/echo_client 10.0.1.1 40000
```
