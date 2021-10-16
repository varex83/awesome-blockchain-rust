#[derive(Default)]
pub struct Node<T> {
    data: T,
    prev: Option<Box<Node<T>>>,
}

#[derive(Default)]
pub struct Chain<T> {
    head: Option<Box<Node<T>>>,
    len: usize,
}

impl<'a, T> Iterator for ChainIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.prev.as_deref();
            &node.data
        })
    }
}

pub struct ChainIter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T: Default> Chain<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn append(&mut self, item: T) {
        let head = self.head.take();
        let node = Box::new(Node {
            data: item,
            prev: head,
        });
        self.head = Some(node);
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn head(&self) -> Option<&T> {
        match &self.head {
            None => None,
            Some(head) => Some(&head.data),
        }
    }

    pub fn iter(&self) -> ChainIter<T> {
        ChainIter {
            next: self.head.as_deref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append() {
        let mut chain = Chain::<u32>::new();
        chain.append(3);
        chain.append(1);

        assert_eq!(chain.head(), Some(&1));
    }

    #[test]
    fn test_append_loop() {
        let mut chain = Chain::<u32>::new();
        chain.append(1);
        chain.append(2);
        chain.append(3);
        chain.append(4);

        for i in chain.iter() {
            dbg!(i);
        }
    }
}
