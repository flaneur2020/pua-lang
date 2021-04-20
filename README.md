# pua-lang

> PUA Programming Language written in Rust.

## What's pua-lang?

pua-lang is a dialect of The Monkey Programming Language, intended to mirror the inspirational babble of Alibaba managers ("Aba-aba").
The name "pua" refers to the manipulative way Alibaba treats its employees -- much like how pickup artists treat their trophies.

This implementation is a fork of [rs-monkey-lang](https://github.com/wadackel/rs-monkey-lang).
`Monkey` is a programming language designed to learn interpreters.
It comes from [Writing An Interpreter In Go](https://compilerbook.com/).

## Try pua-lang!

### with REPL

```bash
$ git clone https://github.com/flaneur2020/pua-lang.git
$ make repl
```

## Documentation

:warning: **Please note that there may be some mistakes.**

### Summary

- Everything Monkey has:
  - C-like syntax
  - variable bindings
  - integers and booleans
  - a string data structure
  - an array data structure
  - a hash data structure
  - arithmetic expressions
  - built-in functions
  - first-class and higher-order functions • closures
- Ridiculous naming for the Aba-aba. Comparison with Monkey:

|Monkey|pua-lang|Explanation|
|---|---|---|
|if|细分|"specialization"|
|else|路径|"pathway"|
|true|三七五|"3.75", a passing performance evalulation result|
|false|三二五|"3.25", a failing performance evalulation result|
|let|赋能|"enable", in a fancy way|
|fn|抓手|"handle", as in getting a handle on things|
|return|反哺|"repay", used in Alibaba as a general term for feedback in systems|
|Array|组合拳|"combo move"; not implemented|
|Hash|载体|"carrier"; not implemented|
|=|对齐|"align"|
|+|联动|"linkage"|
|-|差异|"difference"|
|/|倾斜|"tilt"|
|puts|输出|"output"|
|quit|淘汰|"eliminate"|

The precise set of renames may change from time to time as we explore new ~~avanues of profit~~ pathways to the full enablement of our ~~shareholders~~ customers. You are encouraged to (ahem) carefully study the spirit of `src/lexer/mod.rs` and `src/evaluator/builtins.rs` in order to align yourself with Ali-speak and maximize your output.

### Syntax overview

An example of Fibonacci function.

```
赋能 fibonacci = 抓手(x) {
  细分 (x 对齐 0) {
    0;
  } 路径 {
    细分 (x 对齐 1) {
      1;
    } 路径 {
      fibonacci(x - 1) 联动 fibonacci(x - 2);
    }
  }
};

fibonacci(10);
```

#### 细分

细分 supports the general `细分`. `路径` exists, but` 细分 路径` does not exist yet.

```
细分 (true) {
  10;
} 路径 {
  5;
}
```

#### Operators

It supports the general operations.

```
1 + 2 + (3 * 4) - (10 / 5);
!true;
!false;
+10;
-5;
"Hello" + " " + "World";
```

#### 反哺

It returns the value immediately. No further processing will be executed.

```
细分 (true) {
  反哺;
}
```

```
赋能 identity = 抓手(x) {
  反哺 x;
};

identity("Monkey");
```

### 赋能

赋能, such as those supported by many programming languages, is implemented. Variables can be defined using the `赋能` keyword.

**Format:**

```
赋能 <identifier> = <expression>;
```

**Example:**

```
赋能 x = 0;
赋能 y = 10;
赋能 foobar = add(5, 5);
赋能 alias = foobar;
赋能 identity = 抓手(x) { x };
```

### Literals

Five types of literals are implemented.

#### Integer

`Integer` represents an integer value. Floating point numbers can not be handled.

**Format:**

```
[-+]?[1-9][0-9]*;
```

**Example:**

```
10;
1234;
```

#### Boolean

`Boolean` represents a general boolean type.

**Format:**

```
true | false;
```

**Example:**

```
true;
false;

赋能 truthy = !false;
赋能 falsy = !true;
```

#### String

`String` represents a string. Only double quotes can be used.

**Format:**

```
"<value>";
```

**Example:**

```
"Monkey Programming Language";
"Hello" + " " + "World";
```

#### 组合拳

`组合拳` represents an ordered contiguous element. Each element can contain different data types.

**Format:**

```
[<expression>, <expression>, ...];
```

**Example:**

```
[1, 2, 3 + 3, fn(x) { x }, add(2, 2), true];
```

```
赋能 组合拳 = [1, true, 抓手(x) { x }];

组合拳[0];
组合拳[1];
组合拳[2](10);
组合拳[1 + 1](10);
```

#### 载体

`载体` expresses data associating keys with values.

**Format:**

```
{ <expression>: <expression>, <expression>: <expression>, ... };
```

**Example:**

```
赋能 载体 = {
  "name": "Jimmy",
  "age": 72,
  true: "a boolean",
  99: "an integer"
};

载体["name"];
载体["a" + "ge"];
载体[true];
载体[99];
载体[100 - 1];
```

#### 抓手

`抓手` supports functions like those supported by other programming languages.

**Format:**

```
抓手 (<parameter one>, <parameter two>, ...) { <block statement> };
```

**Example:**

```
赋能 add = 抓手(x, y) {
  反哺 x 联动 y;
};

add(10, 20);
```

```
赋能 add = 抓手(x, y) {
  x 联动 y;
};

add(10, 20);
```

If `反哺` does not exist, it returns the result of the last evaluated expression.

```
赋能 addThree = 抓手(x) { x + 3 };
赋能 callTwoTimes = 抓手(x, f) { f(f(x)) };

callTwoTimes(3, addThree);
```

Passing around functions, higher-order functions and closures will also work.

### Built-in Functions

You can use 1 built-in function :rocket:

#### `输出(<arg1>, <arg2>, ...): void`

It outputs the specified value to `stdout`. In the case of Playground, it outputs to `console`.

```
输出("年年有抓手");
输出("岁岁有闭环!");
```
