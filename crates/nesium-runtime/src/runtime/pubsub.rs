use std::collections::HashMap;

use crate::runtime::types::{Event, EventTopic, RuntimeEventSender};

pub struct RuntimePubSub {
    subscribers: HashMap<EventTopic, Box<dyn RuntimeEventSender>>,
}

impl RuntimePubSub {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, topic: EventTopic, sender: Box<dyn RuntimeEventSender>) {
        self.subscribers.insert(topic, sender);
    }

    pub fn unsubscribe(&mut self, topic: EventTopic) {
        self.subscribers.remove(&topic);
    }

    pub fn has_subscriber(&self, topic: EventTopic) -> bool {
        self.subscribers.contains_key(&topic)
    }

    pub fn broadcast(&mut self, topic: EventTopic, event: Box<dyn Event>) {
        if let Some(subscriber) = self.subscribers.get(&topic) {
            let active = subscriber.send(event);
            if !active {
                // Remove the disconnected subscriber
                self.subscribers.remove(&topic);
            }
        }
    }
}
