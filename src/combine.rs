use std::marker::PhantomData;

use crate::{CustomErrorTrait, ErrorKind};

/// Combine a new error into a stack of existing errors. This merges errors that can be merged
/// to be able to show a terser error if the same error happened multiple times in the same file.
pub fn combine_error<'a, E: CustomErrorTrait<'a, Kind>, Kind: ErrorKind>(
    errors: &mut Vec<E>,
    error: E,
) {
    for e in &mut *errors {
        if CustomErrorTrait::could_merge(e, &error) {
            e.add_contexts_ref(error.get_contexts().iter().cloned());
            return;
        }
    }
    errors.push(error);
}

/// An iterator adapter that keeps track separately of the errors to merge ones that can be merged.
/// The errors have to be retrieved separately using [`Self::errors`].
pub trait CombineErrorsExtender<Iter, T, E, Kind>
where
    Iter: Iterator<Item = Result<T, E>>,
{
    /// Adapt this iterator to keep track of the errors separately and combined them.
    fn combine_errors(self) -> CombineErrors<Iter, T, E, Kind>;
}

impl<'a, Iter, T, E, Kind> CombineErrorsExtender<Iter, T, E, Kind> for Iter
where
    Iter: Iterator<Item = Result<T, E>>,
    E: CustomErrorTrait<'a, Kind>,
    Kind: ErrorKind,
{
    fn combine_errors(self) -> CombineErrors<Iter, T, E, Kind> {
        CombineErrors {
            iter: self,
            errors: Vec::new(),
            _marker: PhantomData {},
        }
    }
}

/// An iterator adapter that keeps track separately of the errors to merge ones that can be merged.
/// The errors have to be retrieved separately using [`Self::errors`].
pub struct CombineErrors<Iter, T, E, Kind>
where
    Iter: Iterator<Item = Result<T, E>>,
{
    iter: Iter,
    errors: Vec<E>,
    _marker: PhantomData<Kind>,
}

impl<'a, Iter, T, E, Kind> Iterator for &mut CombineErrors<Iter, T, E, Kind>
where
    Iter: Iterator<Item = Result<T, E>>,
    E: CustomErrorTrait<'a, Kind>,
    Kind: ErrorKind,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        for result in self.iter.by_ref() {
            match result {
                Result::Ok(value) => {
                    return Some(value);
                }
                Result::Err(error) => combine_error(&mut self.errors, error),
            }
        }
        None
    }
}

impl<'a, Iter, T, E, Kind> CombineErrors<Iter, T, E, Kind>
where
    Iter: Iterator<Item = Result<T, E>>,
    E: CustomErrorTrait<'a, Kind>,
    Kind: ErrorKind,
{
    /// Retrieved the combined errors
    pub fn errors(&self) -> &[E] {
        &self.errors
    }
}
