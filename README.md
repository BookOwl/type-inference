# type-inference

This project is an implementation of Hindley Milner Type inference in Rust. It is based off of chapter 16 in http://www.scala-lang.org/docu/files/ScalaByExample.pdf

## Running
This project contains a simple REPL that allows you to enter your own expressions to type check. To run the REPL clone this repo and run `cargo run`

## The language
The language that this project infers types on it very simple. It only contains

* Lambdas (`fun x -> x`)
* Function application (`a b`)
* Integers (`1`, `42`, etc.)
* Bools (`true`, `false`)
* Math Operators (`+`, `-`, `*`, `/`)
* Relational operators (`>`, `<`, `=`)
* Logical operators (`&&`, `||`)
* Variables (`x`, `foo_24`, etc.)
* Let (`let x = 5 in x + 1`)
* Letrec (`let x = fun y -> ... in x 5`). Like let but allows recursive functions.
* If (`if true then 1 else 0`, `let fact = fun x -> if x < 2 then 1 else (x * (fact (x - 1))) in fact 5`)

There are also a few predefined functions and values.

* `pair` takes an `a` and a `List<a>` and returns a `List<a>`. It is basically a typed cons.
* `nil` is a generic empty list.
* 'first` takes a `List<a>` and returns an `a`. It is like car in Scheme.
* `rest` takes a `List<a>` and returns a `List<a>`. It is like Scheme's cdr.
* `is_nil` takes a `List<a>` and returns a bool.

## Examples:
```
> 1 + 1
int
> 1 + true
TypeError("cannot unify int with bool")
> let square = fun x -> x * x in square 5
int
> letrec fact = fun x -> if x = 0 then 1 else (x * fact (x - 1)) in fact 5
int
> let id = fun x -> x in (id id) 0
int
> letrec map = fun f -> fun p -> if (is_nil p) then nil else pair (f (first p)) (map f (rest p)) in map
(('26 -> '27) -> (List<'26> -> List<'27>))
```


## License
This project is [Unlicensed](UNLICENSE).