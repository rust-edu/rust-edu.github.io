---
title: Resources
---

This page is currently a work-in-progress, but will eventually be home to a sizable, well-curated set of Rust education resources for educators and students alike.
If you would like to contribute to this list, please [open a pull request](https://github.com/rust-edu/rust-edu.github.io/edit/main/content/resources/_index.md)!

## Books

- [The Rust Programming Language](https://doc.rust-lang.org/book/) — Steve Klabnik and Carol Nichols. The official Rust programming book.
- [Programming Rust](https://www.oreilly.com/library/view/programming-rust-2nd/9781492052586/) — Jim Blandy, Jason Orendorff, and Leanora F. S. Tindall. Comprehensive Rust textbook with an emphasis on systems programming.
- [The Rust way of programming （Rust 编程之道）](https://item.jd.com/12479415.html) — Handong Zhang (张汉东). A Chinese Rust programming book. Helps Rust learners build a comprehensive Rust knowledge architecture.

## Exercises

- [Rustlings](https://github.com/rust-lang/rustlings/) — Collection of Rust exercises to be worked through on a user's local machine.

## Examples

- [Rust By Example](https://doc.rust-lang.org/rust-by-example/) — Collection of Rust code snippets which exemplify various Rust programming concepts.

## Teaching and Training

- [Comprehensive Rust 🦀](https://google.github.io/comprehensive-rust/) — A 3-day course on Rust, covering everything from basic syntax to more advanced topics such as generics, error handling, and unsafe Rust. Also includes 'deep-dive' content on Chromium, Bare-metal and Concurrency.
- [Rust 101 (Tweede Golf)](https://github.com/tweedegolf/101-rs/) — Modular course materials for teaching Rust. Organized as four tracks: Rust Language Introduction, Rust for the Web, Rust for Systems Programming, Scientific Rust.
- [Rust 101 (Ralf Jung)](https://github.com/RalfJung/rust-101) — This course covers core Rust in detail, including advanced concepts.

## University courses

Courses that universities teach, or have taught, in Rust, each with its public page: a place to see how others teach it, and whom to ask.
If you teach a course in Rust, please add it here, or tell us on [Zulip](/zulip/).

- [98-008: Intro to Rust Lang](https://stuco.rs/) — Carnegie Mellon University. A student-taught course (StuCo) for a general audience, from the borrow checker to parallelism and `unsafe`; its lectures and homework are public.
- [CS431: Concurrent Programming](https://github.com/kaist-cp/cs431) — KAIST. The theory of concurrency and its practice in Rust, from locks to lock-free data structures, with public homework and lecture videos.
- [Embedded Rust 101 (Wyliodrin & Politehnica Bucharest)](https://embedded-rust-101.wyliodrin.com) — The embedded systems course delivered in Rust using the Raspberry Pi Pico 2 at the Politehnica University of Bucharest.
- [CS 423/523: Rust Programming](https://github.com/pdx-cs-rust) — Portland State University. Bart Massey's course for undergraduate (423) and graduate (523) students; its example code is public.
- [CS 110L: Safety in Systems Programming](https://web.stanford.edu/class/cs110l/) — Stanford University. A companion to CS 110 on how program analysis finds common bugs in systems code, and how Rust helps to build robust systems software.
- [COMP6991: Solving Modern Programming Problems with Rust](https://www.handbook.unsw.edu.au/postgraduate/courses/2026/COMP6991) — UNSW Sydney. Uses Rust to study ideas found across programming languages, such as typing, polymorphism, testing, metaprogramming, concurrency and safety.
- [CMSC388Z: Programming in Rust](https://www.cs.umd.edu/class/fall2021/cmsc388Z/) — University of Maryland, Fall 2021. How to program effectively in Rust, with a project of their choice that students build in small groups.
- [CIS 1905: Rust Programming](https://www.cis.upenn.edu/~cis1905/) — University of Pennsylvania. The ideas beyond Rust's syntax, such as ownership, memory safety and data-race-free parallel programming, and the trade-offs behind them; formerly CIS 198.
- [ECE1724: Performant Software Systems with Rust](https://iqua.ece.toronto.edu/baochun/ece1724/) — University of Toronto. Baochun Li's course on building performant software systems in Rust, through three programming assignments.
- [cs4414: Operating Systems](https://www.cs.virginia.edu/~evans/cs4414/) — University of Virginia, 2013 and 2014. David Evans's operating systems course, an early one taught in Rust, from before Rust 1.0.
- [ECE 459: Programming for Performance](https://github.com/jzarnett/ece459) — University of Waterloo. Profiling, multicore programming, cache consistency and vectorization, taught in Rust rather than in C and C++; its lecture notes are public.

## Collected learning materials

- [rust-learning](https://github.com/ctjhoa/rust-learning) — This is a curated collection of Rust blog posts, videos, etc focused on learning Rust.
- [The Little Book of Rust Books](https://lborb.github.io/book/) — Glyn Normington. A catalogue of books about Rust: the official books of the Rust project, unofficial ones, and books on using Rust for specific kinds of applications.

## By topic

### Asynchronous

- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/) — Comprehensive, up-to-date guide to using Rust's async language features and libraries.
- [Comprehensive Rust 🦀: Concurrency](https://google.github.io/comprehensive-rust/concurrency.html) — A full-day course covering thread-based and async/await-based concurrency.

### Embedded

- [The Embedded Rust Book](https://docs.rust-embedded.org/book/) — Introductory book about using the Rust Programming Language on Bare Metal embedded systems, such as Microcontrollers.
- [Comprehensive Rust 🦀: Bare Metal](https://google.github.io/comprehensive-rust/bare-metal.html) — A full-day course on using Rust for embedded development in a `no_std` environment. Covers both microcontrollers and application processors.
- [Awesome Embedded Rust](https://github.com/rust-embedded/awesome-embedded-rust) — Curated list of resources related to embedded and low-level programming in the programming language Rust.
- [The Rust on ESP Book](https://esp-rs.github.io/book/) — Comprehensive guide on using Rust with Espressif SoCs and modules.
- [Embedded Rust on Espressif](https://esp-rs.github.io/std-training) — Training Material for learning to use Embedded Rust with the Espressif ESP32-C3.

### Unsafe

- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) — The Rustonomicon digs into all the awful details that you need to understand when writing Unsafe Rust programs.
