use std::collections::BTreeMap;
use std::pin::Pin;

use crate::app::Context;
use crate::event::Key;
use crate::event::KeyCode;

// lifetime: the returned future is bounded to the `Context` lifetime, it is possible because we don't
// hand this callback over to `tokio::spawn` which would have required us to have `'static` lifetime
pub type BindCallback = fn(Context<'_>) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

// set the max combo depth for keys, this is also to prevent
// constant reallocation if user give a long combo
pub const MAX_BIND_DEPTH: usize = 8;

/// a node that represent a key in a path, or an action
#[derive(Default)]
struct Bind {
    entries: BTreeMap<Key, Bind>,
    callback: Option<BindCallback>,
}

impl Bind {
    fn add<I>(&mut self, mut combo: I, callback: BindCallback)
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

    /// get the callback based on the given iterator next value
    fn get<I>(&self, mut keys: I) -> Option<&BindCallback>
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
pub struct Binder {
    root: Bind,
}

impl Binder {
    pub fn new() -> Self {
        Self {
            root: Bind::default(),
        }
    }

    /// add a combo to the combo register
    #[inline(always)]
    pub fn add<I>(&mut self, keys: I, callback: BindCallback)
    where
        I: IntoIterator,
        I::Item: Into<Key>,
    {
        self.root.add(keys.into_iter().map(|i| i.into()), callback)
    }

    #[inline(always)]
    pub fn get<I>(&self, keys: I) -> Option<&BindCallback>
    where
        I: IntoIterator,
        I::Item: AsRef<Key>,
    {
        self.root
            .get(keys.into_iter().map(|key| key.as_ref().clone()))
    }
}

#[repr(transparent)]
pub struct BindBuffer(Vec<Key>);

impl BindBuffer {
    pub fn new() -> BindBuffer {
        BindBuffer(Vec::with_capacity(MAX_BIND_DEPTH))
    }

    #[inline]
    pub fn is_mepty(&self) -> bool {
        self.0.is_empty()
    }

    /// pushes the given key to the combo record, return a boolean value inidicating
    /// if the key indeed have an effect on the record
    pub fn push(&mut self, key: Key) -> bool {
        if key.keycode == KeyCode::Esc {
            self.clear();
            return true;
        } else if self.0.len() < MAX_BIND_DEPTH {
            // prevent pushing keys to the vector, this will also not allow
            // for more allocations from the vector
            self.0.push(key);
            return true;
        }
        return false;
    }

    /// clear all the pressed keys in the recorder
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl AsRef<[Key]> for BindBuffer {
    fn as_ref(&self) -> &[Key] {
        self.0.as_slice()
    }
}
