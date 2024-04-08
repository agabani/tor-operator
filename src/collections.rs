pub fn vec_get_or_insert<T, P>(vec: &mut Vec<T>, predicate: P) -> &mut T
where
    P: Fn(&T) -> bool,
    T: Default,
{
    if let Some(position) = vec.iter().position(predicate) {
        vec.get_mut(position).expect("item to be present")
    } else {
        vec.push(T::default());
        vec.last_mut().expect("last item to be present")
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::vec_get_or_insert;

    #[test]
    fn vec_get_or_insert_get() {
        // arrange
        let mut vec = vec![1, 2, 3, 4, 5];

        // act
        let e = vec_get_or_insert(&mut vec, |f| *f == 3);

        // assert
        assert_eq!(3, *e);

        // act
        *e = 6;

        // assert
        assert_eq!(vec[2], 6);
    }

    #[test]
    fn vec_get_or_insert_insert() {
        // arrange
        let mut vec = vec![1, 2, 3, 4, 5];

        // act
        let e = vec_get_or_insert(&mut vec, |f| *f == 6);

        // assert
        assert_eq!(0, *e);

        // act
        *e = 6;

        // assert
        assert_eq!(vec[5], 6);
    }
}
