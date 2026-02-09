🦀 Rust: [![Crates.io](https://img.shields.io/crates/v/context_error.svg)](https://crates.io/crates/context_error) [![context_error documentation](https://docs.rs/context_error/badge.svg)](https://docs.rs/context_error)

# Create nice error messages full of context

My take on creating nice error messages intended for both libraries and end user code. With a high amount of possible ways of adding more context for end users. This project started in pdbtx and then moved with all my new Rust projects but now finally has ended up in its very own crate.

## Features

* Supports rich context (file names, file numbers)
* Supports multiple highlights in a single context
```
   ╭─[file.txt:42]
42 │ Hello world
   ╎  ╶─╴╶╴⁃⁃
   ╵
```
* Supports multiline contexts
* Supports comments on highlights
```
 ╷
 │ Hello world
 ╎  ╶╴
 │ Make it a good one!
 ╎      ╶╴Cool    ╶─╴1
 ╵
```
* Supports adding suggestions to the error message
```
error: Invalid path
 ╷
 │ fileee.txt
 ╵
This file does not exist
Did you mean any of: file.txt, filet.txt?
```
* Supports merging multiple instances of the same error
* Supports version tags in the error
```
error: Invalid number
   ╷
3  │ null,80o0,YES,,67.77
   ╎      ╶──╴
13 │ null,7oo1,NO,-1,23.11
   ╎      ╶──╴
35 │ HOMOSAPIENS,12i1,YES,,1.23
   ╎             ╶──╴
   ╵
This column is not a number
Version: Software AB v2025.42
```
* Supports displaying the output with colours (controlled with a feature)
* Supports displaying the output using only ascii characters (controlled with a feature)
* Supports displaying the output as HTML
  
And most importantly it allows you to only define those properties that are known and make sense and slims down the output to always be neat.



## License

EUPL