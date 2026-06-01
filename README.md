# COCOX

A Conventional Commitlint binary tool (x for executable).

[Work in Progress] Drop-in replacement of [opensource-nepal/commitlint](https://github.com/opensource-nepal/commitlint)

## Development


## Usage: 
```shell
A Conventional Commitlint binary tool

Usage: cocox [OPTIONS] <MESSAGE|--file <FILE>|--hash <HASH>|--from-hash <FROM_HASH>>

Arguments:
  [MESSAGE]  

Options:
      --file <FILE>            
      --hash <HASH>            
      --from-hash <FROM_HASH>  
      --to-hash <TO_HASH>      [default: HEAD]
  -h, --help                   Print help
  -V, --version                Print version
```


Run cocox

```shell
cargo run -- "my commit message"

cargo run -- --file "/foo/bar"


# using commit hash
cargo run -- --hash xxxx-xxxx

# using  range of hashes: 
cargo run -- --from-hash xxxx-xxxx --to-hash xxxx-xxxx

# --to-hash points to HEAD by default
cargo run -- --from-hash xxxx-xxxx 

```
