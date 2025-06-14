use std::{
    any::Any,
    cell::RefCell,
    collections::HashSet,
    rc::Rc,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

static NEXT_NODE_ID: AtomicU64 = AtomicU64::new(0);
impl NodeId {
    pub fn new() -> Self {
        Self(NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

struct SignalInner<T> {
    value: T,
    subscribers: HashSet<NodeId>,
}

#[derive(Clone)]
pub struct ReadSignal<T: 'static> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

#[derive(Clone)]
pub struct WriteSignal<T: 'static> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

thread_local! {
    static CURRENT_NODE_ID: RefCell<Option<NodeId>> = RefCell::new(None);
    static DIRTY_NODES_TX: RefCell<Option<mpsc::Sender<NodeId>>> = RefCell::new(None);
}

pub fn create_signal<T: Any + Clone>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let inner = Rc::new(RefCell::new(SignalInner {
        value,
        subscribers: HashSet::new(),
    }));

    (
        ReadSignal {
            inner: inner.clone(),
        },
        WriteSignal { inner },
    )
}

impl<T: Clone> ReadSignal<T> {
    pub fn get(&self) -> T {
        CURRENT_NODE_ID.with(|id_cell| {
            if let Some(id) = *id_cell.borrow() {
                self.inner.borrow_mut().subscribers.insert(id);
            }
        });
        self.inner.borrow().value.clone()
    }
}

impl<T: 'static> WriteSignal<T> {
    pub fn set(&self, new_value: T) {
        let mut inner = self.inner.borrow_mut();
        inner.value = new_value;

        DIRTY_NODES_TX.with(|tx_cell| {
            if let Some(tx) = tx_cell.borrow().as_ref() {
                for node_id in &inner.subscribers {
                    tx.send(*node_id).unwrap();
                }
            }
        });
    }

    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        let mut inner = self.inner.borrow_mut();
        updater(&mut inner.value);

        DIRTY_NODES_TX.with(|tx_cell| {
            if let Some(tx) = tx_cell.borrow().as_ref() {
                for node_id in &inner.subscribers {
                    tx.send(*node_id).unwrap();
                }
            }
        });
    }
}

pub struct ScopedNodeContext(Option<NodeId>);

impl ScopedNodeContext {
    pub fn new(id: NodeId) -> Self {
        let previous = CURRENT_NODE_ID.with(|id_cell| id_cell.borrow_mut().replace(id));
        Self(previous)
    }
}

impl Drop for ScopedNodeContext {
    fn drop(&mut self) {
        CURRENT_NODE_ID.with(|id_cell| *id_cell.borrow_mut() = self.0);
    }
}

pub fn init_reactivity(tx: mpsc::Sender<NodeId>) {
    DIRTY_NODES_TX.with(|tx_cell| *tx_cell.borrow_mut() = Some(tx));
}
