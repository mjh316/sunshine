# sunshine

To experience the lexer & parser, you'll need Rust and Cargo installed on your computer.
Then simply run `cargo install && cargo run -- [filename] --dbg`, and then you'll get ast.txt and tokens.txt with the output!

~~To run the interpreter, you'll need nodejs. After running the cargo command, run `node interpreter.js` - and the code will be run!~~
The interpreter is now written in Rust! There is a sample file with syntax in bob.txt - just `cargo run -- bob.txt` will run the file!
