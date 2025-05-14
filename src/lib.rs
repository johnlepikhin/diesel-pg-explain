//! A lightweight utility for wrapping Diesel queries with `EXPLAIN (FORMAT JSON)`
//! and parsing the resulting execution plan into a structured Rust type.
//!
//! This crate is intended for use with PostgreSQL and the Diesel ORM.
//! It provides a simple API to introspect query plans programmatically,
//! enabling query diagnostics, optimization tools, or logging systems.
//!
//! # Features
//!
//! - Wraps any Diesel query using `EXPLAIN (FORMAT JSON)`
//! - Parses the JSON output into a typed `ExplainPlan` structure
//! - Compatible with Diesel's `QueryDsl` and `RunQueryDsl`
//! - Deserialization errors are reported as standard Diesel errors
//!
//! # Example
//!
//! ```rust
//! use diesel::prelude::*;
//! use diesl_pg_explain::{ExplainWrapped, ExplainPlan};
//!
//! let connection = &mut establish_connection();
//! let query = users::table.filter(users::name.like("%example%"));
//!
//! let plan: ExplainPlan = query.wrap_explain().explain(connection)?;
//! println!("{:#?}", plan);
//! ```
//!
//! # Integration
//!
//! This crate is best used in development tooling, diagnostics dashboards,
//! or CLI utilities where understanding PostgreSQL query plans is helpful.
//!
//! Note: this does not run the actual query — it only asks PostgreSQL to
//! generate and return the execution plan.
//!
//! # See also
//!
//! - [PostgreSQL EXPLAIN documentation](https://www.postgresql.org/docs/current/using-explain.html)
//!
//! # Crate Features
//!
//! Currently no optional features. May add feature gates for serde or Diesel version in the future.

use diesel::pg::{Pg, PgConnection};
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::query_dsl::methods::LoadQuery;
use serde::{Deserialize, Serialize};

/// Recursive struct which describes the plan of a query
#[derive(Debug, Serialize, Deserialize)]
pub struct ExplainPlan {
    /// The type of the plan node (e.g., "Seq Scan", "Nested Loop", "Hash Join").
    /// Indicates the operation performed at this step in the query execution plan.
    #[serde(rename = "Node Type")]
    pub node_type: String,

    /// The relationship of this node to its parent in the plan tree.
    /// Common values include:
    /// - "Outer": This node is the outer input to a join (e.g., Nested Loop).
    /// - "Inner": This node is the inner input to a join.
    /// - "Subquery": This node is part of a subquery.
    /// - "InitPlan", "SubPlan", "Member": Special plan node roles.
    ///
    /// May be `None` for root nodes or when not applicable.
    #[serde(rename = "Parent Relationship", default)]
    pub parent_relationship: Option<String>,

    /// Indicates whether the plan node is aware of parallel query execution.
    /// If true, the node may participate in or benefit from parallelism.
    #[serde(rename = "Parallel Aware")]
    pub parallel_aware: bool,

    /// Indicates whether the node supports asynchronous execution.
    /// Async-capable nodes can execute operations concurrently with others,
    /// improving performance in some plans (especially with I/O or remote sources).
    #[serde(rename = "Async Capable")]
    pub async_capable: bool,

    /// The estimated cost of starting this plan node.
    /// This typically includes one-time setup costs, like initializing data structures.
    #[serde(rename = "Startup Cost")]
    pub startup_cost: f64,

    /// The estimated total cost of fully executing this plan node,
    /// including startup and all tuple processing.
    #[serde(rename = "Total Cost")]
    pub total_cost: f64,

    /// The estimated number of rows this plan node will output.
    /// This is a planner estimate, not an actual runtime value.
    #[serde(rename = "Plan Rows")]
    pub plan_rows: u64,

    /// The estimated average width (in bytes) of each row produced by this node.
    /// Useful for understanding memory and I/O implications.
    #[serde(rename = "Plan Width")]
    pub plan_width: u64,

    /// Child plan nodes that this node depends on or drives.
    /// For example, a join node will typically have two child plans (inner and outer).
    #[serde(rename = "Plans", default)]
    pub plans: Vec<ExplainPlan>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExplainItem {
    #[serde(rename = "Plan")]
    pub plan: ExplainPlan,
}

/// A wrapper around a Diesel query that transforms it into an
/// `EXPLAIN (FORMAT JSON)` query.
///
/// Use this type to inspect the query execution plan without running the query.
///
/// Example:
/// ```rust
/// let plan = my_query.wrap_explain().explain(&mut conn)?;
/// println!("{:#?}", plan);
/// ```
#[derive(Clone, Copy, QueryId)]
pub struct Explain<Q>(pub Q);

impl<Q> QueryFragment<Pg> for Explain<Q>
where
    Q: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> diesel::result::QueryResult<()> {
        out.push_sql("EXPLAIN (FORMAT JSON) ");
        self.0.walk_ast(out.reborrow())?;
        Ok(())
    }
}

impl<Q: Query> Query for Explain<Q> {
    type SqlType = diesel::sql_types::Text;
}

impl<Q> RunQueryDsl<PgConnection> for Explain<Q> {}

impl<Q> Explain<Q> {
    /// Executes the wrapped query using `EXPLAIN (FORMAT JSON)`, parses the result,
    /// and returns a structured `ExplainPlan` that represents the root of the query plan tree.
    ///
    /// # Errors
    /// Returns a `diesel::result::Error::DeserializationError` if the JSON returned
    /// by PostgreSQL cannot be parsed into an `ExplainPlan`.
    pub fn explain<'a>(self, conn: &mut PgConnection) -> QueryResult<ExplainPlan>
    where
        Self: LoadQuery<'a, PgConnection, String>,
    {
        let r = self.load::<String>(conn)?.into_iter().next().unwrap();

        let r: Vec<ExplainItem> = serde_json::from_str(&r).map_err(|e: serde_json::Error| {
            diesel::result::Error::DeserializationError(Box::new(e))
        })?;
        let r = r.into_iter().next().unwrap().plan;
        Ok(r)
    }
}

/// A trait that allows any Diesel query to be wrapped
/// in an `EXPLAIN (FORMAT JSON)` call using the [`Explain`] wrapper.
///
/// This is implemented for all query types.
pub trait ExplainWrapped: Sized {
    /// Wraps the query into an `EXPLAIN` wrapper, allowing it to be analyzed
    /// using [`Explain::explain()`].
    ///
    /// Example:
    /// ```rust
    /// use diesel_pg_explain::ExplainWrapped;
    /// let explained = query.wrap_explain();
    /// ```
    fn wrap_explain(&self) -> Explain<&Self>;
}

impl<Q> ExplainWrapped for Q {
    fn wrap_explain(&self) -> Explain<&Self> {
        Explain(self)
    }
}
