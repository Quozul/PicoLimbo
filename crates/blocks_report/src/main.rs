mod block_state_builder;

use crate::block_state_builder::BlockStateBuilder;
use blocks_report::load_internal_mapping;

fn main() -> anyhow::Result<()> {
    let internal_mapping = load_internal_mapping()?;

    let search = "minecraft:oak_wall_sign[facing=east]";
    let internal_id = BlockStateBuilder::new(&internal_mapping)
        .with_state_string(search)?
        .build();
    println!("{:#?}", internal_id);

    Ok(())
}
