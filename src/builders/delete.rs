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

    type Set = SqlInput<Unit, W, Unit, Unit>;
    type Get = S;

    fn push_sql(&self, buf: &mut String) -> usize {
        buf.push_str("DELETE FROM ");
        self.source.push_source(buf);
        let idx = self.where_clause.push_where_clause(buf, 1);
        self.selection.push_returning(&self.source, buf);
        idx - 1
    }

    fn into_types(self) -> (Self::Get, Self::Set) {
        (
            self.selection,
            SqlInput {
                values: Unit,
                where_clause: self.where_clause,
                limit: Unit,
                offset: Unit,
            },
        )
    }
}
