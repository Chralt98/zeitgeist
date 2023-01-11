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

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod mock;
mod simple_disputes_pallet_api;
mod tests;

pub use pallet::*;
pub use simple_disputes_pallet_api::SimpleDisputesPalletApi;

#[frame_support::pallet]
mod pallet {
    use crate::SimpleDisputesPalletApi;
    use core::marker::PhantomData;
    use frame_system::pallet_prelude::*;
    use frame_support::{
        dispatch::DispatchResult,
        ensure,
        traits::{Currency, Get, Hooks, IsType},
        PalletId,
        pallet_prelude::*,
        transactional,
    };
    use sp_runtime::{traits::Saturating, DispatchError};
    use zeitgeist_primitives::{
        traits::{DisputeApi, DisputeResolutionApi},
        types::{Report, Asset, Market, MarketDispute, MarketDisputeMechanism, MarketStatus, OutcomeReport},
    };
    use sp_runtime::traits::Saturating;
    use sp_runtime::SaturatedConversion;
    use zeitgeist_primitives::traits::ZeitgeistAssetManager;
    use zrml_market_commons::MarketCommonsPalletApi;
    use orml_traits::currency::NamedMultiReservableCurrency;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Shares of outcome assets and native currency
        type AssetManager: ZeitgeistAssetManager<
            Self::AccountId,
            Balance = <CurrencyOf<Self> as Currency<Self::AccountId>>::Balance,
            CurrencyId = Asset<MarketIdOf<Self>>,
            ReserveIdentifier = [u8; 8],
        >;

        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The base amount of currency that must be bonded in order to create a dispute.
        #[pallet::constant]
        type DisputeBond: Get<BalanceOf<Self>>;

        type DisputeResolution: DisputeResolutionApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
        >;

        /// The additional amount of currency that must be bonded when creating a subsequent
        /// dispute.
        #[pallet::constant]
        type DisputeFactor: Get<BalanceOf<Self>>;

        /// The identifier of individual markets.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The maximum number of disputes allowed on any single market.
        #[pallet::constant]
        type MaxDisputes: Get<u32>;

        /// The pallet identifier.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        #[pallet::constant]
        type PredictionMarketsPalletId: Get<PalletId>;
    }

    type BalanceOf<T> =
    <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketOf<T> = Market<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as frame_system::Config>::BlockNumber,
        MomentOf<T>,
    >;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// For each market, this holds the dispute information for each dispute that's
    /// been issued.
    #[pallet::storage]
    pub type Disputes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        BoundedVec<MarketDispute<T::AccountId, T::BlockNumber>, T::MaxDisputes>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config, {
            OutcomeReserved { market_id: MarketIdOf<T>, dispute: MarketDispute<T::AccountId, T::BlockNumber> },
        }

    #[pallet::error]
    pub enum Error<T> {
        /// 1. Any resolution must either have a `Disputed` or `Reported` market status
        /// 2. If status is `Disputed`, then at least one dispute must exist
        InvalidMarketStatus,
        /// On dispute or resolution, someone tried to pass a non-simple-disputes market type
        MarketDoesNotHaveSimpleDisputesMechanism,
        StorageOverflow,
        OutcomeMismatch,
        CannotDisputeSameOutcome,
        MarketIsNotReported,
        MaxDisputesReached,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(5000)]
        #[transactional]
        pub fn reserve_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            ensure!(
                market.status == MarketStatus::Disputed,
                Error::<T>::InvalidMarketStatus
            );
            ensure!(market.matches_outcome_report(&outcome), Error::<T>::OutcomeMismatch);
            let report = market.report.as_ref().ok_or(Error::<T>::MarketIsNotReported)?;

            let now = <frame_system::Pallet<T>>::block_number();
            let disputes = Disputes::<T>::get(&market_id);
            let num_disputes: u32 = disputes.len().saturated_into();

            Self::ensure_can_not_dispute_the_same_outcome(&disputes, report, &outcome)?;
            Self::ensure_disputes_does_not_exceed_max_disputes(num_disputes)?;

            T::AssetManager::reserve_named(
                &Self::reserve_id(),
                Asset::Ztg,
                &who,
                default_dispute_bond::<T>(disputes.len()),
            )?;

            let market_dispute = MarketDispute { at: now, by: who, outcome };
            <Disputes<T>>::try_mutate(market_id, |disputes| {
                disputes.try_push(market_dispute.clone()).map_err(|_| <Error<T>>::StorageOverflow)
            })?;

            Self::deposit_event(Event::OutcomeReserved {
                market_id,
                dispute: market_dispute,
            });

            Ok((Some(5000)).into())
        }
    }

    impl<T: Config> Pallet<T> {
        #[inline]
        pub fn reserve_id() -> [u8; 8] {
            T::PredictionMarketsPalletId::get().0
        }

        fn ensure_can_not_dispute_the_same_outcome(
            disputes: &[MarketDispute<T::AccountId, T::BlockNumber>],
            report: &Report<T::AccountId, T::BlockNumber>,
            outcome: &OutcomeReport,
        ) -> DispatchResult {
            if let Some(last_dispute) = disputes.last() {
                ensure!(&last_dispute.outcome != outcome, Error::<T>::CannotDisputeSameOutcome);
            } else {
                ensure!(&report.outcome != outcome, Error::<T>::CannotDisputeSameOutcome);
            }

            Ok(())
        }

        #[inline]
        fn ensure_disputes_does_not_exceed_max_disputes(num_disputes: u32) -> DispatchResult {
            ensure!(num_disputes < T::MaxDisputes::get(), Error::<T>::MaxDisputesReached);
            Ok(())
        }

        fn get_auto_resolve(
            disputes: &[MarketDispute<T::AccountId, T::BlockNumber>],
            market: &MarketOf<T>,
        ) -> Option<T::BlockNumber> {
            disputes.last().map(|last_dispute| {
                last_dispute.at.saturating_add(market.deadlines.dispute_duration)
            })
        }

        fn remove_auto_resolve(
            disputes: &[MarketDispute<T::AccountId, T::BlockNumber>],
            market_id: &MarketIdOf<T>,
            market: &MarketOf<T>,
        ) {
            if let Some(dispute_duration_ends_at_block) = Self::get_auto_resolve(disputes, market) {
                T::DisputeResolution::remove_auto_resolve(
                    market_id,
                    dispute_duration_ends_at_block,
                );
            }
        }
    }

    impl<T> DisputeApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type BlockNumber = T::BlockNumber;
        type MarketId = MarketIdOf<T>;
        type Moment = MomentOf<T>;
        type Origin = T::Origin;

        fn on_dispute(
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            Self::remove_auto_resolve(disputes, market_id, market);
            let curr_block_num = <frame_system::Pallet<T>>::block_number();
            // each dispute resets dispute_duration
            let dispute_duration_ends_at_block =
                curr_block_num.saturating_add(market.deadlines.dispute_duration);
            T::DisputeResolution::add_auto_resolve(market_id, dispute_duration_ends_at_block)?;
            Ok(())
        }

        fn on_resolution(
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            _: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::InvalidMarketStatus);

            if let Some(last_dispute) = disputes.last() {
                Ok(Some(last_dispute.outcome.clone()))
            } else {
                Err(Error::<T>::InvalidMarketStatus.into())
            }
        }

        fn get_auto_resolve(
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            _: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<Self::BlockNumber>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            Ok(Self::get_auto_resolve(disputes, market))
        }

        fn has_failed(
            _: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            _: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<bool, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::SimpleDisputes,
                Error::<T>::MarketDoesNotHaveSimpleDisputesMechanism
            );
            // TODO when does simple disputes fail?
            Ok(false)
        }
    }

    impl<T> SimpleDisputesPalletApi for Pallet<T> where T: Config {}

    // No-one can bound more than BalanceOf<T>, therefore, this functions saturates
    pub(crate) fn default_dispute_bond<T>(n: usize) -> BalanceOf<T>
    where
        T: Config,
    {
        T::DisputeBond::get().saturating_add(
            T::DisputeFactor::get().saturating_mul(n.saturated_into::<u32>().into()),
        )
    }
}
