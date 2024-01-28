mod config;
mod parse;
mod tile;

use config::Config;
use parse::{parse_command, Command, Operation};
use river_layout_toolkit::{run, GeneratedLayout, Layout, Rectangle};
use std::{collections::HashMap, convert::Infallible};
use tile::{LeftPrimary, PaddedPrimary, Params, RightPrimary, Tile, TileType};

fn main() {
    let layout = FilTile {
        tag_log: TagLog::new(),
        configs: HashMap::new(),
        default_config: Config::new(),
    };
    run(layout).unwrap();
}

struct FilTile {
    tag_log: TagLog,
    configs: HashMap<u32, Config>,
    default_config: Config,
}

impl FilTile {
    fn get_config(&self) -> &Config {
        if let Some(c) = self.configs.get(&self.tag_log.last_tag) {
            c
        } else {
            &self.default_config
        }
    }

    fn set_config(&mut self, config: Config) {
        self.configs.insert(self.tag_log.last_tag, config);
    }
}

impl Layout for FilTile {
    type Error = Infallible;

    const NAMESPACE: &'static str = "filtile";

    fn user_cmd(
        &mut self,
        cmd: String,
        tags: Option<u32>,
        _output: &str,
    ) -> Result<(), Self::Error> {
        if let Some(t) = tags {
            self.tag_log.record_tags(t);
        }

        let mut config = self.get_config().clone();

        match parse_command(&cmd) {
            Command::Single("swap") => {
                if config.tile == TileType::LeftPrimary {
                    config.tile = TileType::RightPrimary;
                } else {
                    config.tile = TileType::LeftPrimary;
                }
            }
            Command::Single("pad") => {
                config.pad = !config.pad;
            }
            Command::Numeric {
                namespace: "view-padding",
                operation,
                value,
            } => match operation {
                Operation::Add => config.inc_inner(value),
                Operation::Subtract => config.dec_inner(value),
                Operation::Set => config.set_inner(value),
            },
            Command::Numeric {
                namespace: "outer-padding",
                operation,
                value,
            } => match operation {
                Operation::Add => config.inc_outer(value),
                Operation::Subtract => config.dec_outer(value),
                Operation::Set => config.set_outer(value),
            },
            Command::Numeric {
                namespace: "main-ratio",
                operation,
                value,
            } => match operation {
                Operation::Add => config.inc_ratio(value),
                Operation::Subtract => config.dec_ratio(value),
                Operation::Set => config.set_ratio(value),
            },
            _ => println!("invalid command {}", cmd),
        }

        self.set_config(config);

        Ok(())
    }

    fn generate_layout(
        &mut self,
        view_count: u32,
        usable_width: u32,
        usable_height: u32,
        tags: u32,
        _output: &str,
    ) -> Result<GeneratedLayout, Self::Error> {
        self.tag_log.record_tags(tags);

        let config = self.get_config();

        let params = Params {
            view_count,
            usable_width,
            usable_height,
        };

        let mut tile = match config.tile {
            TileType::LeftPrimary => {
                Box::new(LeftPrimary::new(config.inner, config.outer, config.ratio))
                    as Box<dyn Tile>
            }
            TileType::RightPrimary => {
                Box::new(RightPrimary::new(config.inner, config.outer, config.ratio))
                    as Box<dyn Tile>
            }
        };

        if config.pad {
            tile = Box::new(PaddedPrimary::new(tile));
        }

        let mut layout = GeneratedLayout {
            layout_name: "[]=".to_string(),
            views: Vec::with_capacity(view_count as usize),
        };

        layout.views.push(Rectangle {
            x: tile.get_primary_x(&params),
            y: tile.get_primary_y(&params),
            width: tile.get_primary_width(&params),
            height: tile.get_primary_height(&params),
        });

        for index in 1..view_count {
            layout.views.push(Rectangle {
                x: tile.get_stack_x(&params, index),
                y: tile.get_stack_y(&params, index),
                width: tile.get_stack_width(&params, index),
                height: tile.get_stack_height(&params, index),
            });
        }

        Ok(layout)
    }
}

// Keep track of the last "single" tag we see, so that we can store and
// recall configs not based on combinations.
struct TagLog {
    pub last_tag: u32,
    single_tags: Vec<u32>,
}

impl TagLog {
    pub fn new() -> TagLog {
        TagLog {
            last_tag: 0,
            single_tags: (0..31).map(|i| 1 << i).collect(),
        }
    }

    pub fn record_tags(&mut self, tag: u32) {
        if self.single_tags.contains(&tag) {
            self.last_tag = tag;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::TagLog;

    #[test]
    fn it_logs_single_tags() {
        let mut log = TagLog::new();

        log.record_tags(512);
        log.record_tags(14);
        log.record_tags(12);

        assert_eq!(512, log.last_tag);
    }
}