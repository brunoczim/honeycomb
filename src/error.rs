pub struct Chain<E0, E1>
where
    E0: IntoIterator,
    E1: IntoIterator<Item = E0::Item>,
{
    left: Option<E0::IntoIter>,
    right: Option<E1::IntoIter>,
}

impl<E0, E1> Chain<E0, E1>
where
    E0: IntoIterator,
    E1: IntoIterator<Item = E0::Item>,
{
    pub fn new(left: E0, right: E1) -> Self {
        Self { left: Some(left.into_iter()), right: Some(right.into_iter()) }
    }

    pub fn from_left(left: E0) -> Self {
        Self { left: Some(left.into_iter()), right: None }
    }

    pub fn from_right(right: E1) -> Self {
        Self { left: None, right: Some(right.into_iter()) }
    }
}

impl<E0, E1> Iterator for Chain<E0, E1>
where
    E0: IntoIterator,
    E1: IntoIterator<Item = E0::Item>,
{
    type Item = E0::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let left_item = self
            .left
            .take()
            .and_then(|mut iter| iter.next().map(|item| (item, iter)));

        match left_item {
            Some((item, iter)) => {
                self.left = Some(iter);
                Some(item)
            },
            None => self.right.take().and_then(|mut iter| {
                iter.next().map(|item| {
                    self.right = Some(iter);
                    item
                })
            }),
        }
    }
}
