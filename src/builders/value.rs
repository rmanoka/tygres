use crate::{*, utils::*};

use dsl_macros::builder;
builder! {
    struct ValueBuilder {
        values: V as set(Val),
        where_clause: W as filter(Cl),
        limit: L as limit(Lim),
        offset: O as offset(Off),
    }
}

impl ValueBuilder<Unit, Unit, Unit, Unit> {
    pub fn new() -> Self {
        ValueBuilder {
            values: Unit,
            where_clause: Unit,
            limit: Unit,
            offset: Unit,
        }
    }
}

use postgres::types::ToSql;
impl<
    'a, V, W, L, O,
    A: Takes<'a, V>, B: Takes<'a, W>, C: Takes<'a, L>, D: Takes<'a, O>
> Takes<'a, ValueBuilder<V, W, L, O>> for SqlInput<A, B, C, D> {
    #[inline]
    fn push_values<'b>(&'a self, values: ValueBuilder<V, W, L, O>, buf: &'b mut Vec<&'a ToSql>) {
        self.values.push_values(values.values, buf);
        self.where_clause.push_values(values.where_clause, buf);
        self.limit.push_values(values.limit, buf);
        self.offset.push_values(values.offset, buf);
    }
}
