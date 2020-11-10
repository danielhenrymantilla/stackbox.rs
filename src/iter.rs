use crate::prelude::*;

pub
struct Iter<'frame, Item : 'frame> (
    StackBox<'frame, [Item]>,
);

impl<'frame, Item : 'frame> Iterator for Iter<'frame, Item> {
    type Item = Item;

    fn next (self: &'_ mut Iter<'frame, Item>)
      -> Option<Item>
    {
        self.0.stackbox_pop()
    }
}

impl<'frame, Item : 'frame> IntoIterator
    for StackBox<'frame, [Item]>
{
    type IntoIter = Iter<'frame, Item>;
    type Item = Item;

    fn into_iter (self: StackBox<'frame, [Item]>)
      -> Iter<'frame, Item>
    {
        Iter(self)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn doctest_for_miri ()
    {
        use ::stackbox::prelude::*;

        let boxed_slice: StackBox<'_, [_]>; stackbox!([
            String::from("Hello, "),
            String::from("World!"),
        ] => boxed_slice);
        for s in boxed_slice {
            println!("{}", s);
            drop::<String>(s);
        }
    }
}
