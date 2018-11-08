use crate::{*, utils::*};
use postgres::types::{FromSql};

pub trait Selection<F: Source> {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String);
}

impl<'a, F: Source, S: Selection<F>> Selection<F> for &'a S {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String) {
        <S as Selection<F>>::push_selection(self, src, buf);
    }
}

impl<F: Source, A: Selection<F>, B: Selection<F>> Selection<F> for Seq<A, B> {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String) {
        self.0.push_selection(src, buf);
        buf.push_str(", ");
        self.1.push_selection(src, buf);
    }
}

pub trait Makes<'a, S> {
    fn get<R: Row>(&'a self, row: &'a R, idx: usize) -> (S, usize);
}

impl<
    'a, S, T,
    A: Makes<'a, S>,
    B: Makes<'a, T>
> Makes<'a, Seq<S, T>> for Seq<A, B> {
    fn get<R: Row>(&'a self, row: &'a R, idx: usize) -> (Seq<S, T>, usize) {
        let (a, idx) = self.0.get(row, idx);
        let (b, idx) = self.1.get(row, idx);
        (Seq(a, b), idx)
    }
}

impl<'a, S, A: Makes<'a, S>> Makes<'a, S> for ColWrap<A> {
    fn get<R: Row>(&'a self, row: &'a R, idx: usize) -> (S, usize) {
        self.0.get(row, idx)
    }
}

impl<'a, S, A: Makes<'a, S>> Makes<'a, Wrap<S>> for Wrap<A> {
    fn get<R: Row>(&'a self, row: &'a R, idx: usize) -> (Wrap<S>, usize) {
        let (el, idx) = self.0.get(row, idx);
        (Wrap(el), idx)
    }
}


pub trait ReturningClause<F: Source> {
    #[inline]
    fn push_returning(&self, src: &F, buf: &mut String);
}

impl<F: Source, S: Selection<F>> ReturningClause<F> for Wrap<S> {
    #[inline]
    fn push_returning(&self, src: &F, buf: &mut String) {
        buf.push_str(" RETURNING ");
        self.0.push_selection(src, buf);
    }
}

impl<F: Source> ReturningClause<F> for Unit {
    #[inline]
    fn push_returning(&self, _src: &F, _buf: &mut String) {}
}
