use std::collections::HashMap;

use crate::RuntimeEvent;
use crate::runtime::types::{EventTopic, RuntimeEventSender};

pub struct RuntimePubSub<S: RuntimeEventSender> {
    subscribers: HashMap<EventTopic, S>,
}

impl<S: RuntimeEventSender> RuntimePubSub<S> {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, topic: EventTopic, sender: S) {
        self.subscribers.insert(topic, sender);
    }

    pub fn broadcast(&mut self, event: RuntimeEvent) {
        let topic = event.topic();
        if let Some(subscriber) = self.subscribers.get(&topic) {
            let active = subscriber.send(event);
            if !active {
                // Remove the disconnected subscriber
                self.subscribers.remove(&topic);
            }
        }
    }
}
