[![crates.io](https://img.shields.io/crates/v/diesel-pg-explain.svg)](https://crates.io/crates/diesel-pg-explain)
[![docs.rs](https://docs.rs/diesel-pg-explain/badge.svg)](https://docs.rs/diesel-pg-explain)
[![CI](https://github.com/johnlepikhin/diesel-pg-explain/actions/workflows/ci.yml/badge.svg)](https://github.com/johnlepikhin/diesel-pg-explain/actions)
[![License: MIT](https://img.shields.io/crates/l/diesel-pg-explain.svg)](https://github.com/johnlepikhin/diesel-pg-explain/blob/main/LICENSE)


# diesel-explain-plan

A lightweight helper crate for wrapping [Diesel](https://diesel.rs/) PostgreSQL queries
in `EXPLAIN (FORMAT JSON)` and parsing the result into structured Rust types.

This crate is intended for diagnostics, performance analysis, query logging, and automated tooling that works with PostgreSQL query plans.

---

## ✨ Features

- 🧱 Wrap any Diesel query in `EXPLAIN (FORMAT JSON)`
- 📊 Parse the output into a structured `ExplainPlan` tree
- 🛠 Integrates with Diesel’s query builder and connection types
- ⚠️ Does not execute the actual query — just retrieves the plan

---

## 🚀 Example

```rust
use diesel::prelude::*;
use diesel_pg_explain::{ExplainWrapped, ExplainPlan};

let connection = &mut establish_connection();

let query = users::table.filter(users::age.gt(30));

// Wrap the query with EXPLAIN
let plan: ExplainPlan = query.wrap_explain().explain(connection)?;

// Print the query plan tree
println!("{:#?}", plan);
```
