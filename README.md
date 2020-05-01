A simplified file system of ext2 standard.

### How to run
```
git clone https://github.com/susliko/ext2.git
cd ext2/
cargo run --bin ext2server 4242 # run daemon listening on the port 4242
cargo run --bin ext2client 127.0.0.1 4242 # connect to the daemon running at 127.0.0.1:4242
```

### Example of usage
```
Welcome to a modest ext2-like file system!. Type `help` to list its capabilities.
/ > mkdir home
/ > cd home
/home/ > touch bashrc
export PATH=/home/bin
/home/ > ls      
bashrc
..
/home/ > cat bashrc
export PATH=/home/bin
/home/ > cd ..
/ > rm home/
/ > cd home
Unknown directory name: home/
```



