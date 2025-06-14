use std::{
    any::Any,
    cell::RefCell,
    collections::{HashMap, HashSet},
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct EffectId(u64);

static NEXT_EFFECT_ID: AtomicU64 = AtomicU64::new(0);
impl EffectId {
    fn new() -> Self {
        Self(NEXT_EFFECT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Subscriber {
    Node(NodeId),
    Effect(EffectId),
}

thread_local! {
    static SUBSCRIBER_STACK: RefCell<Vec<Subscriber>> = RefCell::new(Vec::new());
    static EFFECTS: RefCell<HashMap<EffectId, Rc<dyn Fn()>>> = RefCell::new(HashMap::new());
    static DIRTY_NODES_TX: RefCell<Option<mpsc::Sender<NodeId>>> = RefCell::new(None);
}

struct SignalInner<T> {
    value: T,
    subscribers: HashSet<Subscriber>,
}

#[derive(Clone)]
pub struct ReadSignal<T: 'static> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

#[derive(Clone)]
pub struct WriteSignal<T: 'static> {
    inner: Rc<RefCell<SignalInner<T>>>,
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
        SUBSCRIBER_STACK.with(|stack| {
            if let Some(subscriber) = stack.borrow().last() {
                self.inner.borrow_mut().subscribers.insert(*subscriber);
            }
        });
        self.inner.borrow().value.clone()
    }
}

impl<T: 'static> WriteSignal<T> {
    fn notify_subscribers(&self) {
        let subscribers = self.inner.borrow().subscribers.clone();
        for sub in subscribers {
            match sub {
                Subscriber::Node(node_id) => {
                    DIRTY_NODES_TX.with(|tx_cell| {
                        if let Some(tx) = tx_cell.borrow().as_ref() {
                            tx.send(node_id).unwrap();
                        }
                    });
                }
                Subscriber::Effect(effect_id) => {
                    EFFECTS.with(|effects| {
                        if let Some(effect_fn) = effects.borrow().get(&effect_id) {
                            effect_fn();
                        }
                    });
                }
            }
        }
    }

    pub fn set(&self, new_value: T) {
        self.inner.borrow_mut().value = new_value;
        self.notify_subscribers();
    }

    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        updater(&mut self.inner.borrow_mut().value);
        self.notify_subscribers();
    }
}

pub type Memo<T> = ReadSignal<T>;

pub fn create_memo<T, F>(derive_fn: F) -> Memo<T>
where
    T: Any + Clone + 'static,
    F: Fn() -> T + 'static,
{
    let (read_memo, write_memo) = create_signal(derive_fn());

    create_effect(move || {
        let new_value = derive_fn();
        write_memo.set(new_value);
    });

    read_memo
}

pub fn create_effect<F>(effect_fn: F)
where
    F: Fn() + 'static,
{
    let id = EffectId::new();
    let effect_fn_rc = Rc::new(effect_fn);

    let runner: Rc<dyn Fn()> = Rc::new({
        let effect_fn_rc = effect_fn_rc.clone();
        move || {
            SUBSCRIBER_STACK.with(|stack| {
                stack.borrow_mut().push(Subscriber::Effect(id));
            });

            (effect_fn_rc)();

            SUBSCRIBER_STACK.with(|stack| {
                stack.borrow_mut().pop();
            });
        }
    });

    EFFECTS.with(|effects| {
        effects.borrow_mut().insert(id, runner.clone());
    });

    runner();
}

pub struct ScopedNodeContext(Option<NodeId>);

impl ScopedNodeContext {
    pub fn new(id: NodeId) -> Self {
        SUBSCRIBER_STACK.with(|stack| {
            stack.borrow_mut().push(Subscriber::Node(id));
        });

        Self(Some(id))
    }
}

impl Drop for ScopedNodeContext {
    fn drop(&mut self) {
        if self.0.is_some() {
            SUBSCRIBER_STACK.with(|stack| {
                stack.borrow_mut().pop();
            });
        }
    }
}

pub fn init_reactivity(tx: mpsc::Sender<NodeId>) {
    DIRTY_NODES_TX.with(|tx_cell| *tx_cell.borrow_mut() = Some(tx));
}
