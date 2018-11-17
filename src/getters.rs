use crate::{*, utils::*};
use postgres::types::{FromSql};

pub trait Selection<F: Source> {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String) -> bool;
}

impl<'a, F: Source, S: Selection<F>> Selection<F> for &'a S {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String) -> bool {
        <S as Selection<F>>::push_selection(self, src, buf)
    }
}

impl<F: Source, A: Selection<F>, B: Selection<F>> Selection<F> for Seq<A, B> {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String) -> bool {
        if self.0.push_selection(src, buf) {
            buf.push_str(", ");
        }
        self.1.push_selection(src, buf)
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

pub trait Value: Sized {
    type Get;
    fn get<'a, R: Row>(g: &'a Self::Get, row: &'a R, idx: usize) -> (Self, usize);
}

// impl<'a, G, V> Makes<'a, V> for G
// where V: Value<Get = G> {
//     fn get<R: Row>(&'a self, row: &'a R, idx: usize) -> (V, usize) {
//         V::get(self, row, idx)
//     }
// }


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

pub struct OptionalSelection<S>(pub S, bool);

impl<C> ColWrap<C> {
    #[inline]
    pub fn select_if<S: Source>(self, assignment: bool) -> OptionalSelection<Self>
    where Self: Selection<S> {
        OptionalSelection(self, assignment)
    }
}

impl<F: Source, S: Selection<F>> Selection<F> for OptionalSelection<S> {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String) -> bool {
        self.1 &&
            self.0.push_selection(src, buf)
    }
}

impl<'a, S, A: Makes<'a, S>> Makes<'a, Option<S>> for OptionalSelection<A> {
    fn get<R: Row>(&'a self, row: &'a R, idx: usize) -> (Option<S>, usize) {
        if (self.1) {
            let (obj, idx) = self.0.get(row, idx);
            (Some(obj), idx)
        } else {
            (None, idx)
        }
    }
}
