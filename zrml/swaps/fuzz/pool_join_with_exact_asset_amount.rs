// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};

use utils::ExactAssetAmountData;
mod utils;
use orml_traits::currency::MultiCurrency;
use utils::construct_asset;
use zrml_swaps::mock::AssetManager;

fuzz_target!(|data: ExactAssetAmountData| {
    let mut ext = ExtBuilder::default().build();
    ext.execute_with(|| {
        // ensure that the account origin has a sufficient balance
        // use orml_traits::MultiCurrency; required for this
        for a in &data.pool_creation.assets {
            let _ = AssetManager::deposit(
                construct_asset(*a),
                &data.pool_creation.origin,
                // In order to successfully join the pool, data.asset_amount more tokens needed
                data.pool_creation.amount.saturating_add(data.asset_amount),
            );
        }
        let pool_id = data.pool_creation.create_pool();
        let _ = Swaps::pool_join_with_exact_asset_amount(
            Origin::signed(data.origin),
            pool_id,
            construct_asset(data.asset),
            data.asset_amount,
            data.pool_amount,
        );
    });
    let _ = ext.commit_all();
});
