use serde::Deserialize;
use std::collections::HashMap;

/*
{
  "minecraft:acacia_button": {
    "properties": {
      "face": [
        "floor",
        "wall",
        "ceiling"
      ],
      "facing": [
        "north",
        "south",
        "west",
        "east"
      ],
      "powered": [
        "true",
        "false"
      ]
    },
    "states": [
      {
        "properties": {
          "face": "wall",
          "facing": "north",
          "powered": "false"
        },
        "id": 5408,
        "default": true
      },
      {
        "properties": {
          "face": "wall",
          "facing": "south",
          "powered": "true"
        },
        "id": 5409
      }
    ]
  }
}
*/

#[derive(Deserialize, Debug)]
struct BlockState {
    #[serde(default)]
    default: bool,
    id: u32,
    #[serde(default)]
    properties: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct Block {
    properties: HashMap<String, Vec<String>>,
    states: Vec<BlockState>,
}

impl Block {
    pub fn get_default_id(&self) -> Option<u32> {
        self.states
            .iter()
            .find_map(|state| if state.default { Some(state.id) } else { None })
            .or(self.states.first().map(|state| state.id))
    }

    pub fn get_property_count(&self) -> usize {
        self.properties.len()
    }
}

fn main() {}
