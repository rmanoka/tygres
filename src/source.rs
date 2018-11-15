use crate::{*, utils::*};

pub trait Source: Sized {
    #[inline]
    fn push_source(&self, buf: &mut String);

    #[inline]
    fn select(self) -> SelectBuilder<Self, Unit, Unit, Unit, Unit, Unit> {
        SelectBuilder {
            source: self,
            selection: Unit,
            where_clause: Unit,
            limit: Unit,
            offset: Unit,
            order: Unit,
        }
    }


    #[inline]
    fn insert(self) -> InsertBuilder<Self, Unit, Unit, Unit> {
        InsertBuilder{
            source: self,
            values: Unit,
            selection: Unit,
            reps: Unit,
        }
    }

    #[inline]
    fn update(self) -> UpdateBuilder<Self, Unit, Unit, Unit> {
        UpdateBuilder{
            source: self,
            values: Unit,
            selection: Unit,
            where_clause: Unit,
        }
    }

    #[inline]
    fn delete(self) -> DeleteBuilder<Self, Unit, Unit> {
        DeleteBuilder {
            source: self,
            selection: Unit,
            where_clause: Unit,
        }
    }
}

pub trait Column<F: Source> {
    #[inline]
    fn push_name(&self, buf: &mut String);
}

impl<F: Source, C: Column<F>> Selection<F> for ColWrap<C> {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String) {
        src.push_source(buf);
        buf.push_str(".");
        self.0.push_name(buf);
    }
}

impl<F: Source, C: Column<F>> ColumnsSetter<F> for ColWrap<C> {
    #[inline]
    fn push_selection(&self, buf: &mut String) -> bool {
        self.0.push_name(buf);
        true
    }

    #[inline]
    fn push_values(&self, buf: &mut String, idx: usize) -> usize {
        if (idx != 1) {
            buf.push_str(", ");
        }
        buf.push_str(&format!("${}", idx));
        idx + 1
    }
}

