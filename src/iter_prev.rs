use std::fmt;

pub struct Iter<I>(usize, I, Vec<<I as Iterator>::Item>)
where
    I: Iterator;

impl<I> fmt::Debug for Iter<I>
where
    I: Iterator,
    <I as Iterator>::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Iter({}, {:?})", self.0, self.2)
    }
}

impl<I> Iter<I>
where
    I: Iterator,
    <I as Iterator>::Item: Clone + fmt::Debug,
{
    pub fn new(i: I) -> Self {
        Iter(0, i, vec![])
    }

    pub fn prev(&mut self) -> Option<<I as Iterator>::Item> {
        tracing::trace!("prev({self:?})");
        if self.0 > 0 {
            self.0 -= 1;
            Some(self.2[self.0].clone())
        } else if self.0 == 1 {
            self.0 -= 1;
            None
        } else {
            None
        }
    }

    pub fn first(&mut self) -> Option<<I as Iterator>::Item> {
        if self.0 > 0 {
            self.0 = 0;
            Some(self.2[self.0].clone())
        } else {
            self.1.next()
        }
    }
}

impl<I> Iterator for Iter<I>
where
    I: Iterator,
    <I as Iterator>::Item: Clone + fmt::Debug,
{
    type Item = <I as Iterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<<I as Iterator>::Item> {
        tracing::trace!("next({self:?})");
        if self.0 == self.2.len() {
            if let Some(e) = self.1.next() {
                self.0 += 1;
                self.2.push(e.clone());
                Some(e)
            } else {
                None
            }
        } else {
            self.0 += 1;
            Some(self.2[self.0 - 1].clone())
        }
    }
}

#[test]
fn test_iter_prev() {
    let _ = tracing_subscriber::fmt::try_init();

    let mut iter = Iter::new(0..5);
    assert_eq!(iter.prev(), None);
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.prev(), Some(0));
    assert_eq!(iter.prev(), None);
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.prev(), Some(1));
    assert_eq!(iter.prev(), Some(0));
    assert_eq!(iter.prev(), None);
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.prev(), Some(2));
    assert_eq!(iter.prev(), Some(1));
    assert_eq!(iter.prev(), Some(0));
    assert_eq!(iter.prev(), None);

    assert_eq!(Iter::new(0..5).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);

    let mut iter = Iter::new(0..5);
    assert_eq!(
        iter.by_ref().take(0).collect::<Vec<_>>(),
        Vec::<usize>::new(),
    );
    assert_eq!(iter.prev(), None);

    let mut iter = Iter::new(0..5);
    assert_eq!(iter.by_ref().take(1).collect::<Vec<_>>(), vec![0]);
    assert_eq!(iter.prev(), Some(0));

    let mut iter = Iter::new(0..5);
    assert_eq!(iter.by_ref().take(3).collect::<Vec<_>>(), vec![0, 1, 2]);
    assert_eq!(iter.prev(), Some(2));
    assert_eq!(iter.take(3).collect::<Vec<_>>(), vec![2, 3, 4]);
}

#[test]
fn test_iter_first() {
    let _ = tracing_subscriber::fmt::try_init();

    assert_eq!(Iter::new(0..5).first(), Some(0));

    let mut iter = Iter::new(0..5);
    let _: Vec<_> = iter.by_ref().take(3).collect();
    assert_eq!(iter.first(), Some(0));
}
