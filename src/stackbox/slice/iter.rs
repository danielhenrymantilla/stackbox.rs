use crate::prelude::*;

pub struct Iter<'frame, Item: 'frame>(StackBox<'frame, [Item]>);

impl<'frame, Item: 'frame> Iterator for Iter<'frame, Item> {
    type Item = Item;

    #[inline]
    fn next(self: &'_ mut Iter<'frame, Item>) -> Option<Item> {
        self.0.pop_front()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}

impl<'frame, Item: 'frame> ExactSizeIterator for Iter<'frame, Item> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'frame, Item: 'frame> DoubleEndedIterator for Iter<'frame, Item> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

impl<'frame, Item: 'frame> IntoIterator for StackBox<'frame, [Item]> {
    type Item = Item;
    type IntoIter = Iter<'frame, Item>;

    #[inline]
    fn into_iter(self: StackBox<'frame, [Item]>) -> Iter<'frame, Item> {
        Iter(self)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn doctest_for_miri() {
        use ::stackbox::prelude::*;

        stackbox!(let boxed_slice: StackBox<'_, [_]> = [
            String::from("Hello, "),
            String::from("World!"),
        ]);
        for s in boxed_slice {
            println!("{}", s);
            drop::<String>(s);
        }
    }
}
