# pmux

Tcp port multiplexer. Nothing special.

## Usage

1. On the target machine,

```
$ pmux backend 127.0.0.1:10000
```

2. On the local machine,

```
$ pmux frontend 127.0.0.1:10000 -r 8080:1001 -r 8081:1002
```

Then, connecting to `127.0.0.1:8080` on the local machine actually connects to `1001` of the target machine. `127.0.0.1:8081` connects to `1002`.
