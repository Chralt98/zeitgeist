#![no_main]

use libfuzzer_sys::fuzz_target;

use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod data_structs;
use data_structs::GeneralPoolData;
use zeitgeist_primitives::{traits::Swaps as SwapsTrait, types::ScoringRule};
mod helper_functions;
use helper_functions::asset;

fuzz_target!(|data: GeneralPoolData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        if let Ok(pool_id) = Swaps::create_pool(
            data.pool_creation.origin.into(),
            data.pool_creation.assets.into_iter().map(asset).collect(),
            Some(data.pool_creation.base_asset).map(asset),
            data.pool_creation.market_id.into(),
            ScoringRule::CPMM,
            Some(data.pool_creation.swap_fee).into(),
            Some(data.pool_creation.weights).into(),
        ) {
            // join a pool with a valid pool id
            let _ = Swaps::pool_join(
                Origin::signed(data.origin.into()),
                pool_id,
                data.pool_amount,
                data.assets,
            );
        }
    });

    let _ = ext.commit_all();
});
