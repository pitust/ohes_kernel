use crate::prelude::*;
use alloc::rc::Rc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventListenerID {
    id: u32,
    name: String,
}
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Copy)]
pub struct EventListenerIDInterface {
    id: u32,
}
impl EventListenerIDInterface {
    pub fn get_name(&self) -> String {
        match EVENT_LISTENERS.get().get(&EventListenerID {
            id: self.id,
            name: "<unknown>".to_string(),
        }) {
            Some(listener) => listener.id.name.clone(),
            None => "<unknown>".to_string(),
        }
    }
    pub fn emit(&self, event_name: String) {
        let listeners = EVENT_LISTENERS.get();
        let listener = listeners
            .get(&EventListenerID {
                id: self.id,
                name: "<unknown>".to_string(),
            })
            .expect("Invalid EventListenerIDInterface");
        match listener.events.get(&event_name) {
            Some(listeners) => {
                for listener in listeners {
                    listener();
                }
            }
            None => {}
        }
    }
    pub fn listen(&self, event_name: String, fcn: Box<dyn SyncFn()>) {
        let listeners = EVENT_LISTENERS.get();
        let listener = listeners
            .get_mut(&EventListenerID {
                id: self.id,
                name: "<unknown>".to_string(),
            })
            .expect("Invalid EventListenerIDInterface");
        match listener.events.get_mut(&event_name) {
            Some(listeners) => {
                listeners.push_back(fcn);
            }
            None => {
                let mut list = LinkedList::new();
                list.push_back(fcn);
                listener.events.insert(event_name, list);
            }
        }
    }
}
impl core::fmt::Debug for EventListenerIDInterface {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ll = EVENT_LISTENERS.get();
        let v = ll.get(&EventListenerID {
            id: 0,
            name: "<unknown>".to_string(),
        });
        match v {
            Some(v) => write!(f, "[EventEmitter: {} (id: {})]", v.id.name, v.id.id),
            None => write!(f, "[EventEmitter: <invalid> (id: {})]", self.id),
        }
    }
}
impl core::cmp::PartialOrd for EventListenerID {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        return self.id.partial_cmp(&other.id);
    }
}
impl core::cmp::Ord for EventListenerID {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        return self.id.cmp(&other.id);
    }
}

pub trait SyncFn<T>: Sync + Send + Fn<T> {}
pub struct EventListener {
    events: BTreeMap<String, LinkedList<Box<dyn SyncFn()>>>,
    id: EventListenerID,
}

ezy_static! { EVENT_LISTENERS, BTreeMap<EventListenerID, EventListener>, BTreeMap::new() }
ezy_static! { EVENT_NAME_TO_ID, BTreeMap<String, u32>, BTreeMap::new() }
counter!(ListenerId);
pub fn create_listener(name: String, public: bool) -> EventListenerIDInterface {
    let id = ListenerId.inc();
    let id = id as u32;
    let idstruct = EventListenerID {
        id,
        name: name.clone(),
    };
    let l = EventListener {
        events: BTreeMap::new(),
        id: idstruct.clone(),
    };
    EVENT_LISTENERS.get().insert(idstruct, l);
    if public {
        EVENT_NAME_TO_ID.get().insert(name, id);
    }
    EventListenerIDInterface { id }
}
pub fn get_id_by_name(name: String) -> Option<EventListenerIDInterface> {
    match EVENT_NAME_TO_ID.get().get(&name) {
        Some(val) => Some(EventListenerIDInterface { id: *val }),
        None => None,
    }
}
