# type-inference

This project is an implementation of Hindley Milner Type inference in Rust. It is based off of https://github.com/prakhar1989/type-inference

## Running
This project contains a simple REPL that allows you to enter your own expressions to type check. To run the REPL clone this repo and run `cargo run`

## The language
The language that this project infers types on it very simple. It only contains

* Lambdas (`fun x -> x`)
* Function application (`a b`)
* Integers (`1`, `42`, etc.)
* Bools (`true`, `false`)
* Math Operators (`+`, `-`, `*`, `/`)
* Relational operators (`>`, `<`)
* Logical operators (`&&`, `||`)
* Variables (`x`, `foo_24`, etc.)

## License
This project is [Unlicensed](UNLICENSE).