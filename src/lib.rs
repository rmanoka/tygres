#![feature(associated_type_defaults)]

#[macro_use]
pub mod macros;

pub mod utils;

pub mod query;
pub mod source;
pub mod getters;
pub mod order_by;
pub mod filter;
pub mod setters;
pub mod builders;

pub use self::{
    utils::{Seq},
    query::{Row, TypedRow, IntoSql, SqlInput},
    query::synchronous, // query::asynchronous,
    source::{Source, Column},
    getters::{Selection, ReturningClause, Makes, OptionalSelection, Getter},
    filter::{Clause, WhereClause, Equality},
    setters::{ColumnsSetter, Takes, HasSetter, HasOwnedSetter, RefSetter, ValSetter},
    order_by::{OrderByClause},
    builders::*,
};
