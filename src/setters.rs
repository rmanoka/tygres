use crate::{*, utils::*};
use postgres::types::ToSql;

pub trait ColumnsSetter<F: Source> {
    fn push_selection(&self, buf: &mut String);
    fn push_values(&self, buf: &mut String, idx: usize) -> usize;
}

impl<
    F: Source,
    A: ColumnsSetter<F>,
    B: ColumnsSetter<F>
> ColumnsSetter<F> for Seq<A, B> {
    #[inline]
    fn push_selection(&self, buf: &mut String) {
        self.0.push_selection(buf);
        buf.push_str(", ");
        self.1.push_selection(buf);
    }

    #[inline]
    fn push_values(&self, buf: &mut String, idx: usize) -> usize {
        let idx = self.0.push_values(buf, idx);
        buf.push_str(", ");
        self.1.push_values(buf, idx)
    }
}

pub trait Takes<'a, S: Sized> {
    fn push_values<'b>(&'a self, values: S, buf: &'b mut Vec<&'a ToSql>);
}

impl<'a, S, T: Takes<'a, S>> Takes<'a, Wrap<S>> for Wrap<T> {
    #[inline]
    fn push_values<'b>(&'a self, values: Wrap<S>, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
    }
}

impl<'a, S, T: Takes<'a, S>> Takes<'a, S> for ColWrap<T> {
    #[inline]
    fn push_values<'b>(&'a self, values: S, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values, buf);
    }
}

impl<'a, A, B, S: Takes<'a, A>, T: Takes<'a, B>> Takes<'a, Seq<A, B>> for Seq<S, T> {
    #[inline]
    fn push_values<'b>(&'a self, values: Seq<A, B>, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
        self.1.push_values(values.1, buf);
    }
}

impl<'a, A: 'a, S: Takes<'a, A>, I: IntoIterator<Item = A>> Takes<'a, I> for Reps<S> {
    #[inline]
    fn push_values<'b>(&'a self, iter: I, buf: &'b mut Vec<&'a ToSql>) {
        for value in iter {
            self.1.push_values(value, buf);
        }
    }
}

// impl<'a, A: 'a, S: Takes<'a, Wrap<A>>, I: IntoIterator<Item = A>> Takes<'a, Wrap<I>> for Reps<S> {
//     fn push_values<'b>(&'a self, iter: Wrap<I>, buf: &'b mut Vec<&'a ToSql>) {
//         let mut len = 0;
//         for value in iter.0 {
//             self.1.push_values(Wrap(value), buf);
//             len += 1;
//         }
//         if (self.0 != len) {
//             panic!("array length mismatch");
//         }

//     }
// }


// impl<S, T: Takes<'a, S>> Takes<'a, Wrap<S>> for Wrap<T> {
//     fn push_values<'b>(&'a self, values: &'a Wrap<S>, buf: &'b mut Vec<&'a ToSql>) {
//         self.0.push_values(&values.0, buf);
//     }
// }

impl<'a> Takes<'a, Unit> for Unit {
    #[inline]
    fn push_values<'b>(&'a self, _values: Unit, _buf: &'b mut Vec<&'a ToSql>) {}
}