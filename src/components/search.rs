use std::sync::Arc;

use nucleo::{
    Config, Nucleo,
    pattern::{CaseMatching, Normalization},
};

use crate::types::Item;

pub struct SearchEngine {
    nucleo: Nucleo<(usize, String)>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            nucleo: Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1),
        }
    }

    pub fn add(&mut self, items: &[Item]) {
        let injector = self.nucleo.injector();
        for (i, item) in items.iter().enumerate() {
            injector.push((i, item.name.clone()), |(_, name), columns| {
                columns[0] = name.clone().into();
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

    pub fn results(&self) -> Vec<usize> {
        self.nucleo
            .snapshot()
            .matched_items(0..)
            .map(|item| item.data.0)
            .collect::<Vec<_>>()
    }
}
