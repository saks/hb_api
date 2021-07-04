[![Build Status](https://travis-ci.org/saks/hb_api.svg?branch=master)](https://travis-ci.org/saks/hb_api)

### Run tests
./run.sh diesel database setup
./run.sh cargo test

### Setup
You need to install OpenSSL and set the environment variable to make it visible to the compiler; this changes depending on the operation system and package manager, for example, in macOS you may need to do something like this:

```
$ brew install openssl
$ export LIBRARY_PATH="$(brew --prefix openssl)/lib"
$ export CFLAGS="-I$(brew --prefix openssl)/include"
$ cargo ...
```
