<h1 align="center">todo cli</h1>
<div align="center">
  <img width="500" src="img/demo.gif" alt="demo"/>
  <p>A command line todo manager to manage your tasks across multiple projects</p>
</div>
<p align="center">
  <a href="https://github.com/jofmar00/todo/blob/master/LICENSE">
    <img src="https://img.shields.io/badge/Licence-MIT-yellow.svg?longCache=true&style=flat-square"/>
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/Made With-Rust-red.svg?longCache=true&style=flat-square"/>
  </a>
  <img src="https://img.shields.io/badge/Version-0.1.0-blue.svg?longCache=true&style=flat-square"/>
  <img src="https://img.shields.io/badge/Platform-CLI-lightgrey.svg?longCache=true&style=flat-square"/>
</p>

## Installation

This project is completely made in [Rust](https://www.rust-lang.org/). Cargo is necesary in order to install the tool.

```bash
cargo install --git https://github.com/jofmar00/todo.git
```

## Getting started
The todo cli is a very simple todo task manager written in Rust, with various subcommands.

Here are some to get you started:

- Run `todo add "My epic task" (tag)` to add a new todo wit.
- Run `todo done <id>` to complete your task. 
- Run `todo` to list all of your tasks, or `todo ls <tag1> <tag2>` to list by tag.
- Run `todo use <tag>` to use a new tag for all your new tasks and to list by that tag.

Checkout all the subcommands using `todo help`.
## Configuring
The location of your todos and your configuration will depend on these environment variables (in this order):

1. **TODO_PATH**: determines where your `.todo` file will live
2. **HOME**: a fallback if `TODO_PATH` is not set. 
