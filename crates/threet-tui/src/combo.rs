use std::collections::BTreeMap;
use std::pin::Pin;

use crate::app::AppContext;
use crate::event::Key;

pub type ComboCallback = fn(&AppContext) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

// TODO
macro_rules! combos {
    ($($($key:expr)* => callback:tt),*) => {};
}

/// a node that represent a key in a path, or an action
#[derive(Default)]
struct Node {
    entries: BTreeMap<Key, Node>,
    callback: Option<ComboCallback>,
}

impl Node {
    fn add<I>(&mut self, mut combo: I, callback: ComboCallback)
    where
        I: Iterator<Item = Key>,
    {
        // if there is a next key in the combo iterator
        // it means the current node should hold a child
        // and pass the combo to the child, if we are at the end of the combo (`None`)
        // it means the current node should hold the callback
        match combo.next() {
            Some(key) => self.entries.entry(key).or_default().add(combo, callback),
            None => self.callback = Some(callback),
        };
    }

    fn get<I>(&self, mut keys: I) -> Option<&ComboCallback>
    where
        I: Iterator<Item = Key>,
    {
        match keys.next() {
            Some(key) => self.entries.get(&key).and_then(|node| node.get(keys)),
            None => self.callback.as_ref(),
        }
    }
}

#[repr(transparent)]
pub struct Combo {
    root: Node,
}

impl Combo {
    pub fn new() -> Self {
        Self {
            root: Node::default(),
        }
    }

    #[inline(always)]
    pub fn add<I>(&mut self, keys: I, callback: ComboCallback)
    where
        I: IntoIterator<Item = Key>,
    {
        self.root.add(keys.into_iter(), callback)
    }

    #[inline(always)]
    pub fn get<I>(&self, keys: I) -> Option<&ComboCallback>
    where
        I: IntoIterator,
        I::Item: AsRef<Key>,
    {
        self.root
            .get(keys.into_iter().map(|key| key.as_ref().clone()))
    }
}
