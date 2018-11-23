use crate::{*, utils::*};

use tygres_macros::builder;
builder! {
    struct DeleteBuilder {
        source: F[Source],
        selection: S as returning(Sel: Selection<F>),
        where_clause: W as filter(Cl: Clause<F>),
    }
}

impl<F: Source, S: ReturningClause<F>, W: WhereClause<F>> IntoSql
for DeleteBuilder<F, S, W> {

    type Set = SqlInput<Unit, W::Set, Unit, Unit>;
    type Get = S;

    fn push_sql(&mut self, buf: &mut String, idx: usize) -> usize {
        buf.push_str("DELETE FROM ");
        self.source.push_source(buf);
        let idx = self.where_clause.push_where_clause(buf, idx);
        self.selection.push_returning(&self.source, buf);
        idx
    }

    fn into_types(self) -> (Self::Get, Self::Set) {
        (
            self.selection,
            SqlInput {
                values: Unit,
                where_clause: self.where_clause.into_types(),
                limit: Unit,
                offset: Unit,
            },
        )
    }
}
