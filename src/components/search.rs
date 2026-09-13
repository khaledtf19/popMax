use std::sync::Arc;

use nucleo::{
    Config, Nucleo,
    pattern::{CaseMatching, Normalization},
};

use crate::types::Item;

pub struct SearchEngine {
    nucleo: Nucleo<Item>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            nucleo: Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1),
        }
    }

    pub fn add(&mut self, items: Vec<Item>) {
        let injector = self.nucleo.injector();
        for item in items {
            injector.push(item, |item, columns| {
                columns[0] = item.name.clone().into();
            });
        }
    }

    pub fn search(&mut self, query: &str) {
        self.nucleo
            .pattern
            .reparse(0, query, CaseMatching::Ignore, Normalization::Smart, false);
        loop {
            let status = self.nucleo.tick(10);

            if !status.running {
                break;
            }
        }
    }

    pub fn results(&self) -> Vec<Item> {
        self.nucleo
            .snapshot()
            .matched_items(0..)
            .map(|item| item.data.clone())
            .collect::<Vec<_>>()
    }
}
