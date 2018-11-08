use crate::{*, utils::*};

pub trait OrderSeq<F: Source> {
    #[inline]
    fn push_seq(&self, buf: &mut String);
}

impl<F: Source, A: OrderSeq<F>, B: OrderSeq<F>> OrderSeq<F> for Seq<A, B> {
    #[inline]
    fn push_seq(&self, buf: &mut String) {
        self.0.push_seq(buf);
        buf.push_str(", ");
        self.1.push_seq(buf);
    }
}

pub trait OrderByClause<F: Source> {
    #[inline]
    fn push_order_by(&self, buf: &mut String);
}

impl<F: Source> OrderByClause<F> for Unit {
    #[inline]
    fn push_order_by(&self, _: &mut String) {}
}

impl<F: Source, O: OrderSeq<F>> OrderByClause<F> for Wrap<O> {
    #[inline]
    fn push_order_by(&self, buf: &mut String) {
        buf.push_str(" ORDER BY ");
        self.0.push_seq(buf);
    }
}

pub struct Asc<C>(pub C);
pub struct Desc<C>(pub C);

impl<C> Wrap<C> {
    #[inline]
    pub fn asc(self) -> Asc<C> {
        Asc(self.0)
    }

    #[inline]
    pub fn desc(self) -> Desc<C> {
        Desc(self.0)
    }
}

impl<F: Source, C: Column<F>> OrderSeq<F> for Asc<C> {
    #[inline]
    fn push_seq(&self, buf: &mut String) {
        self.0.push_name(buf);
        buf.push_str(" ASC");
    }
}

impl<F: Source, C: Column<F>> OrderSeq<F> for Desc<C> {
    #[inline]
    fn push_seq(&self, buf: &mut String) {
        self.0.push_name(buf);
        buf.push_str(" DESC");
    }
}

