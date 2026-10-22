use crate::*;
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::json_types::U64;
use near_sdk::serde::Serialize;
use near_sdk::{assert_one_yocto, near_bindgen, AccountId, Promise, PromiseResult};

/// One accepted USDT/mpDAO observation. `mpdao_per_token_e9` is what sells use for that day.
/// `cap_base_e9` is the previous accepted price, and it does not move when today's price falls,
/// so a crash cannot be followed by an uncapped pump on the same day.
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct SellPriceDay {
    pub day_index: u64,
    pub mpdao_per_token_e9: u64,
    pub cap_base_e9: u64,
}

/// Seven daily observations plus how much USDT today's sells have already paid.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct SellPriceGuard {
    pub days: Vec<SellPriceDay>,
    pub payout_day_index: u64,
    pub usdt_paid_today: u128,
}

impl Default for SellPriceGuard {
    fn default() -> Self {
        Self {
            days: Vec::new(),
            payout_day_index: 0,
            usdt_paid_today: 0,
        }
    }
}

/// Quote for selling one locked mpDAO position back to the vote contract.
///
/// The USDT comes from the buy-and-lock reserve (`TokenInfo.amount_received` of the
/// USDT token), which can also be funded with `ft_transfer_call` msg `"fund-reserve"`.
/// The payout price is the lower USD price of today's accepted observation and the
/// median of the last 7 accepted days. That price can fall immediately and can rise
/// at most 10% from the previous accepted day.
///
/// Discount versus that price is `max(40%, 90% * min(days, 300) / 300)`.
/// A locked position uses its locking period. An unlocking position uses the days
/// still left, rounded up. A 30-day lock keeps 60%. A 300-day lock keeps 10%.
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SellPositionQuote {
    pub payout_token: AccountId,
    pub index: PositionIndex,
    pub mpdao_amount: U128String,
    pub locking_days: Days,
    /// Basis points taken off the protected free-mpDAO USDT price. 9000 = 90%.
    pub discount_bps: u16,
    /// Basis points of that price that are paid. 1000 = 10%.
    pub retained_bps: u16,
    pub anchor_days: u16,
    pub anchor_discount_bps: u16,
    pub floor_discount_bps: u16,
    /// USDT the same mpDAO would fetch at the protected price, with no lock.
    pub spot_usdt: U128String,
    /// USDT the same mpDAO would fetch at the operator's latest raw price, with no lock.
    pub operator_spot_usdt: U128String,
    /// USDT offered for this lock.
    pub usdt_amount: U128String,
    /// USDT still in the reserve.
    pub usdt_reserve: U128String,
    pub reserve_covers_offer: bool,
    pub price_fresh: bool,
    pub price_paused: bool,
    pub token_enabled: bool,
    pub voting_power_free: bool,
    pub is_locked: bool,
    /// True while the unbond clock is still running. The offer then uses days left.
    pub is_unlocking: bool,
    /// False until the operator has posted at least one USDT observation.
    pub price_guard_ready: bool,
    pub operator_mpdao_per_token_e9: U64,
    /// mpDAO per USDT used to price the offer. Higher means a lower USDT payout.
    pub sell_mpdao_per_token_e9: U64,
    pub price_observation_days: u8,
    pub daily_usdt_cap: U128String,
    pub usdt_paid_today: U128String,
    pub daily_cap_covers_offer: bool,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SellPriceDayView {
    pub day_index: U64,
    pub mpdao_per_token_e9: U64,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SellPriceGuardView {
    pub days: Vec<SellPriceDayView>,
    pub payout_day_index: U64,
    pub usdt_paid_today: U128String,
    pub daily_usdt_cap: U128String,
    pub sell_mpdao_per_token_e9: U64,
}

/// `max(40%, 90% * min(days, 300) / 300)`.
pub(crate) fn sell_discount_bps(locking_days: Days) -> u16 {
    let days = (locking_days as u32).min(SELL_DISCOUNT_ANCHOR_DAYS as u32);
    let linear =
        (SELL_DISCOUNT_AT_ANCHOR_BPS as u32 * days / SELL_DISCOUNT_ANCHOR_DAYS as u32) as u16;
    linear.max(SELL_DISCOUNT_FLOOR_BPS)
}

pub(crate) fn sell_retained_bps(locking_days: Days) -> u16 {
    10_000 - sell_discount_bps(locking_days)
}

/// Days still left, rounded up. One millisecond into a 240-day unbond still counts as 240 days.
pub(crate) fn ceil_days_remaining(now_ms: u64, started_ms: u64, locking_days: Days) -> Days {
    let ends = started_ms.saturating_add(days_to_millis(locking_days));
    if now_ms >= ends {
        return 0;
    }
    let left = ends - now_ms;
    let day_ms = days_to_millis(1) as u128;
    ((left as u128 + day_ms - 1) / day_ms) as Days
}

/// Locked positions use the original lock. Unlocking positions use the days still left.
/// A finished unbond cannot be sold.
pub(crate) fn sell_pricing_days(position: &LockingPosition) -> Option<Days> {
    if position.is_locked() {
        return Some(position.locking_period);
    }
    let started = position.unlocking_started_at?;
    if !position.is_unlocking() {
        return None;
    }
    let days = ceil_days_remaining(
        get_current_epoch_millis(),
        started,
        position.locking_period,
    );
    if days == 0 {
        None
    } else {
        Some(days)
    }
}

/// Inverse of `TokenInfo::compute_mpdao_amount` at a given price, without the freshness check.
/// `mpdao_per_token_e9` is mpDAO per 1 whole token, scaled by 1e9.
pub(crate) fn token_out_for_mpdao(
    mpdao_amount: u128,
    token_decimals: u8,
    mpdao_per_token_e9: u64,
) -> u128 {
    assert!(mpdao_per_token_e9 > 0, "price is zero");
    let scale = 10u128.pow((token_decimals as u32) + 9 - 6);
    proportional(mpdao_amount, scale, mpdao_per_token_e9 as u128)
}

pub(crate) fn discounted_token_out(
    mpdao_amount: u128,
    token_decimals: u8,
    mpdao_per_token_e9: u64,
    locking_days: Days,
) -> u128 {
    let spot = token_out_for_mpdao(mpdao_amount, token_decimals, mpdao_per_token_e9);
    proportional(spot, sell_retained_bps(locking_days) as u128, 10_000)
}

/// Accept a lower USD price immediately. A higher USD price cannot exceed
/// `rise_bps` above the previous accepted price. USD up means mpDAO-per-token down.
pub(crate) fn capped_mpdao_per_token(previous: u64, observed: u64, rise_bps: u16) -> u64 {
    if observed >= previous {
        return observed;
    }
    let floor = (previous as u128) * 10_000 / (10_000 + rise_bps as u128);
    observed.max(floor.max(1) as u64)
}

pub(crate) fn median_u64(values: &[u64]) -> u64 {
    assert!(!values.is_empty(), "median of empty prices");
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        ((sorted[mid - 1] as u128 + sorted[mid] as u128) / 2) as u64
    }
}

/// Lower USD price of today's accepted observation and the median of the window.
/// Lower USD price is the larger mpDAO-per-token number.
pub(crate) fn protective_mpdao_per_token(days: &[SellPriceDay]) -> u64 {
    assert!(!days.is_empty(), "no sell price observations");
    let values: Vec<u64> = days.iter().map(|day| day.mpdao_per_token_e9).collect();
    let today = *values.last().expect("no sell price observations");
    today.max(median_u64(&values))
}

pub(crate) fn accept_usdt_observation(days: &mut Vec<SellPriceDay>, day_index: u64, observed: u64) {
    require!(observed > 0, "USDT price is zero");
    if days.is_empty() {
        days.push(SellPriceDay {
            day_index,
            mpdao_per_token_e9: observed,
            cap_base_e9: observed,
        });
        return;
    }
    let last = days.len() - 1;
    if days[last].day_index == day_index {
        days[last].mpdao_per_token_e9 = capped_mpdao_per_token(
            days[last].cap_base_e9,
            observed,
            SELL_PRICE_MAX_DAILY_RISE_BPS,
        );
        return;
    }
    require!(
        day_index > days[last].day_index,
        "USDT price day is older than the stored day"
    );
    let previous = days[last].mpdao_per_token_e9;
    let accepted = capped_mpdao_per_token(previous, observed, SELL_PRICE_MAX_DAILY_RISE_BPS);
    days.push(SellPriceDay {
        day_index,
        mpdao_per_token_e9: accepted,
        cap_base_e9: previous,
    });
    if days.len() > SELL_PRICE_HISTORY_DAYS {
        let overflow = days.len() - SELL_PRICE_HISTORY_DAYS;
        days.drain(0..overflow);
    }
}

pub(crate) fn is_usdt_token(token: &AccountId) -> bool {
    token.as_str().split('.').next() == Some("usdt")
}

#[near_bindgen]
impl MetaVoteContract {
    fn usdt_payout_token(&self) -> AccountId {
        let usdt_tokens: Vec<AccountId> = self
            .token_info
            .keys()
            .filter(|token| is_usdt_token(token))
            .collect();
        require!(!usdt_tokens.is_empty(), "USDT token is not configured");
        require!(
            usdt_tokens.len() == 1,
            "more than one USDT token is configured"
        );
        usdt_tokens.into_iter().next().unwrap()
    }

    pub(crate) fn record_usdt_sell_observation(&mut self, observed: u64) {
        let day = env::block_timestamp_ms() / MS_PER_DAY;
        accept_usdt_observation(&mut self.sell_price_guard.days, day, observed);
    }

    fn usdt_paid_today(&self) -> u128 {
        let day = env::block_timestamp_ms() / MS_PER_DAY;
        if self.sell_price_guard.payout_day_index == day {
            self.sell_price_guard.usdt_paid_today
        } else {
            0
        }
    }

    /// Credit USDT into the sell reserve. The tokens must arrive through `ft_transfer_call`.
    pub(crate) fn credit_usdt_reserve(
        &mut self,
        sender_id: &AccountId,
        token: &AccountId,
        amount: u128,
    ) {
        require!(amount > 0, "Amount of tokens sent must be greater than 0");
        let payout = self.usdt_payout_token();
        require!(token == &payout, "only USDT can fund the sell reserve");
        let mut token_info = self.token_info.get(&payout).expect("USDT token not found");
        require!(
            token_info.token_decimals == 6,
            "USDT payout token must have 6 decimals"
        );
        token_info.amount_received = token_info
            .amount_received
            .checked_add(amount)
            .expect("USDT reserve overflow");
        let reserve = token_info.amount_received;
        self.token_info.insert(&payout, &token_info);
        log!(
            "FUND RESERVE: {} added {} USDT, reserve now {}",
            sender_id,
            amount,
            reserve
        );
    }

    fn build_sell_quote(
        &self,
        voter_id: &String,
        voter: &Voter,
        index: PositionIndex,
        position: &LockingPosition,
    ) -> SellPositionQuote {
        let payout_token = self.usdt_payout_token();
        let token_info = self
            .token_info
            .get(&payout_token)
            .expect("USDT token not found");
        require!(
            token_info.token_decimals == 6,
            "USDT payout token must have 6 decimals"
        );
        let price = self
            .mpdao_prices
            .get(&payout_token)
            .expect("USDT price not found");
        require!(price.mpdao_per_token_e9 > 0, "USDT price is zero");

        let paid_today = self.usdt_paid_today();
        let guard_ready = !self.sell_price_guard.days.is_empty();
        let sell_price_e9 = if guard_ready {
            protective_mpdao_per_token(&self.sell_price_guard.days)
        } else {
            0
        };
        let operator_spot_usdt = token_out_for_mpdao(
            position.amount,
            token_info.token_decimals,
            price.mpdao_per_token_e9,
        );
        let spot_usdt = if guard_ready {
            token_out_for_mpdao(position.amount, token_info.token_decimals, sell_price_e9)
        } else {
            0
        };
        let pricing_days = sell_pricing_days(position);
        let days = pricing_days.unwrap_or(0);
        let can_price = pricing_days.is_some();
        let retained_bps = sell_retained_bps(days);
        let usdt_amount = if guard_ready && can_price {
            proportional(spot_usdt, retained_bps as u128, 10_000)
        } else {
            0
        };
        let daily_room = SELL_DAILY_USDT_CAP.saturating_sub(paid_today);

        let full_vp = self.internal_get_delegated_vp(voter_id) + voter.sum_locked_vp();
        let available_vp = full_vp.saturating_sub(voter.sum_used_votes());
        // Unlocking positions already dropped their voting power when the unbond started.
        let voting_power_free = if position.is_locked() {
            available_vp >= position.voting_power
        } else {
            can_price
        };

        SellPositionQuote {
            payout_token,
            index,
            mpdao_amount: position.amount.into(),
            locking_days: days,
            discount_bps: sell_discount_bps(days),
            retained_bps,
            anchor_days: SELL_DISCOUNT_ANCHOR_DAYS,
            anchor_discount_bps: SELL_DISCOUNT_AT_ANCHOR_BPS,
            floor_discount_bps: SELL_DISCOUNT_FLOOR_BPS,
            spot_usdt: spot_usdt.into(),
            operator_spot_usdt: operator_spot_usdt.into(),
            usdt_amount: usdt_amount.into(),
            usdt_reserve: token_info.amount_received.into(),
            reserve_covers_offer: usdt_amount > 0 && token_info.amount_received >= usdt_amount,
            price_fresh: price.is_fresh(),
            price_paused: price.paused,
            token_enabled: token_info.enabled,
            voting_power_free,
            is_locked: position.is_locked(),
            is_unlocking: position.is_unlocking() && can_price,
            price_guard_ready: guard_ready,
            operator_mpdao_per_token_e9: price.mpdao_per_token_e9.into(),
            sell_mpdao_per_token_e9: sell_price_e9.into(),
            price_observation_days: self.sell_price_guard.days.len() as u8,
            daily_usdt_cap: SELL_DAILY_USDT_CAP.into(),
            usdt_paid_today: paid_today.into(),
            daily_cap_covers_offer: usdt_amount > 0 && usdt_amount <= daily_room,
        }
    }

    /// USDT the contract would pay for `voter_id`'s locking position `index`.
    pub fn get_sell_position_quote(
        &self,
        voter_id: VoterId,
        index: PositionIndex,
    ) -> SellPositionQuote {
        let voter = self.internal_get_voter_or_panic(&voter_id);
        let position = voter.get_position(index);
        self.build_sell_quote(&voter_id, &voter, index, &position)
    }

    /// Accepted daily sell prices and how much of today's USDT cap is already used.
    pub fn get_sell_price_guard(&self) -> SellPriceGuardView {
        let sell_price = if self.sell_price_guard.days.is_empty() {
            0
        } else {
            protective_mpdao_per_token(&self.sell_price_guard.days)
        };
        SellPriceGuardView {
            days: self
                .sell_price_guard
                .days
                .iter()
                .map(|day| SellPriceDayView {
                    day_index: day.day_index.into(),
                    mpdao_per_token_e9: day.mpdao_per_token_e9.into(),
                })
                .collect(),
            payout_day_index: self.sell_price_guard.payout_day_index.into(),
            usdt_paid_today: self.usdt_paid_today().into(),
            daily_usdt_cap: SELL_DAILY_USDT_CAP.into(),
            sell_mpdao_per_token_e9: sell_price.into(),
        }
    }

    /// Sell a whole locked or unlocking position for USDT from the reserve.
    /// A locked position also drops its voting power. An unlocking position already did.
    /// mpDAO is returned to the sell inventory only after the USDT transfer succeeds.
    #[payable]
    pub fn sell_locking_position(&mut self, index: PositionIndex) -> Promise {
        assert_one_yocto();
        let seller = env::predecessor_account_id();
        let voter_id = seller.to_string();
        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        let position = voter.get_position(index);
        require!(
            sell_pricing_days(&position).is_some(),
            "only a locked or unlocking position can be sold"
        );

        let quote = self.build_sell_quote(&voter_id, &voter, index, &position);
        require!(
            quote.price_guard_ready,
            "USDT sell price is not initialized"
        );
        require!(quote.token_enabled, "USDT is not enabled");
        require!(quote.price_fresh, "price is stale");
        require!(!quote.price_paused, "USDT price is paused");
        require!(quote.usdt_amount.0 > 0, "USDT offer is zero");
        require!(
            quote.reserve_covers_offer,
            "not enough USDT in the buy-and-lock reserve"
        );
        require!(
            quote.daily_cap_covers_offer,
            "daily USDT payout cap reached"
        );

        let mpdao_amount = position.amount;
        let locking_days = position.locking_period;
        let voting_power = position.voting_power;
        let was_locked = position.is_locked();
        let unlocking_started_at = position.unlocking_started_at.unwrap_or(0);
        let usdt_amount = quote.usdt_amount.0;
        let payout_token = quote.payout_token.clone();
        let payout_day = env::block_timestamp_ms() / MS_PER_DAY;
        if self.sell_price_guard.payout_day_index != payout_day {
            self.sell_price_guard.payout_day_index = payout_day;
            self.sell_price_guard.usdt_paid_today = 0;
        }
        self.sell_price_guard.usdt_paid_today = self
            .sell_price_guard
            .usdt_paid_today
            .checked_add(usdt_amount)
            .expect("daily USDT payout overflow");

        voter.remove_position(index);
        if was_locked {
            self.total_voting_power = self.total_voting_power.saturating_sub(voting_power);
            self.require_vp_available(&voter_id, &mut voter);
        }
        self.voters.insert(&voter_id, &voter);

        let mut token_info = self
            .token_info
            .get(&payout_token)
            .expect("USDT token not found");
        token_info.amount_received = token_info
            .amount_received
            .checked_sub(usdt_amount)
            .expect("USDT reserve underflow");
        self.token_info.insert(&payout_token, &token_info);

        log!(
            "SELL: {} selling position {} ({} mpDAO, {} days) for {} USDT, discount {} bps",
            seller,
            index,
            mpdao_amount,
            locking_days,
            usdt_amount,
            quote.discount_bps
        );

        ext_ft_core::ext(payout_token.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(
                seller.clone(),
                usdt_amount.into(),
                Some("Sell locked mpDAO position".to_string()),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_SELL_CALLBACK)
                    .after_sell_locking_position(
                        &seller,
                        mpdao_amount.into(),
                        locking_days,
                        voting_power.into(),
                        unlocking_started_at,
                        usdt_amount.into(),
                        &payout_token,
                        payout_day,
                    ),
            )
    }

    #[private]
    pub fn after_sell_locking_position(
        &mut self,
        seller: &AccountId,
        mpdao_amount: U128String,
        locking_days: Days,
        voting_power: U128String,
        unlocking_started_at: u64,
        usdt_amount: U128String,
        payout_token: &AccountId,
        payout_day: u64,
    ) {
        let mpdao_amount = mpdao_amount.0;
        let usdt_amount = usdt_amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                self.mpdao_avail_to_sell = self
                    .mpdao_avail_to_sell
                    .checked_add(mpdao_amount)
                    .expect("mpdao inventory overflow");
                log!(
                    "SELL: {} sold {} mpDAO locked {} days for {} {}",
                    seller,
                    mpdao_amount,
                    locking_days,
                    usdt_amount,
                    payout_token
                );
            }
            PromiseResult::Failed => {
                log!(
                    "FAILED: USDT transfer to {} failed. Restoring position.",
                    seller
                );
                if self.sell_price_guard.payout_day_index == payout_day {
                    self.sell_price_guard.usdt_paid_today = self
                        .sell_price_guard
                        .usdt_paid_today
                        .saturating_sub(usdt_amount);
                }
                let mut token_info = self
                    .token_info
                    .get(payout_token)
                    .expect("USDT token not found");
                token_info.amount_received = token_info
                    .amount_received
                    .checked_add(usdt_amount)
                    .expect("USDT reserve overflow");
                self.token_info.insert(payout_token, &token_info);

                let voter_id = seller.to_string();
                let mut voter = self.internal_get_voter(&voter_id);
                if unlocking_started_at == 0 {
                    self.deposit_locking_position(
                        mpdao_amount,
                        locking_days,
                        &voter_id,
                        &mut voter,
                    );
                } else {
                    self.restore_unlocking_position(
                        &voter_id,
                        &mut voter,
                        mpdao_amount,
                        locking_days,
                        voting_power.0,
                        unlocking_started_at,
                    );
                }
            }
        }
    }

    /// Put an unlocking position back after a failed USDT transfer.
    /// Voting power stays removed, because the unbond already dropped it.
    fn restore_unlocking_position(
        &mut self,
        voter_id: &String,
        voter: &mut Voter,
        mpdao_amount: MpDAOAmount,
        locking_days: Days,
        voting_power: u128,
        unlocking_started_at: u64,
    ) {
        assert!(
            (voter.locking_positions.len() as u8) < self.max_locking_positions,
            "The max number of locking positions is {}",
            self.max_locking_positions
        );
        voter.locking_positions.push(&LockingPosition::new(
            mpdao_amount,
            locking_days,
            voting_power,
            Some(unlocking_started_at),
        ));
        self.voters.insert(voter_id, voter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_days_round_up_and_a_finished_unbond_is_zero() {
        let day = days_to_millis(1);
        let started = 1_000_000_000_000u64;
        assert_eq!(ceil_days_remaining(started, started, 240), 240);
        assert_eq!(ceil_days_remaining(started + 1, started, 240), 240);
        assert_eq!(ceil_days_remaining(started + day, started, 240), 239);
        assert_eq!(ceil_days_remaining(started + day + 1, started, 240), 239);
        let ends = started + days_to_millis(240);
        assert_eq!(ceil_days_remaining(ends - 1, started, 240), 1);
        assert_eq!(ceil_days_remaining(ends, started, 240), 0);
    }

    #[test]
    fn short_locks_keep_sixty_percent_and_300_days_keeps_ten() {
        assert_eq!(sell_discount_bps(0), 4_000);
        assert_eq!(sell_retained_bps(0), 6_000);

        assert_eq!(sell_discount_bps(30), 4_000);
        assert_eq!(sell_retained_bps(30), 6_000);

        // 90% * 120/300 = 36%, so the 40% floor still binds.
        assert_eq!(sell_discount_bps(120), 4_000);
        assert_eq!(sell_retained_bps(120), 6_000);

        // 90% * 150/300 = 45%, above the floor.
        assert_eq!(sell_discount_bps(150), 4_500);
        assert_eq!(sell_retained_bps(150), 5_500);

        assert_eq!(sell_discount_bps(300), 9_000);
        assert_eq!(sell_retained_bps(300), 1_000);
        assert_eq!(sell_discount_bps(360), 9_000);
        assert_eq!(sell_retained_bps(360), 1_000);
    }

    #[test]
    fn offer_uses_the_floor_then_the_anchor() {
        let price_e9 = 1_000_000_000u64;
        let one_mpdao = 1_000_000u128;
        let spot = token_out_for_mpdao(one_mpdao, 6, price_e9);
        assert_eq!(spot, 1_000_000);

        let offer_300 = discounted_token_out(one_mpdao, 6, price_e9, 300);
        assert_eq!(offer_300, 100_000);

        let offer_30 = discounted_token_out(one_mpdao, 6, price_e9, 30);
        assert_eq!(offer_30, 600_000);

        let offer_150 = discounted_token_out(one_mpdao, 6, price_e9, 150);
        assert_eq!(offer_150, 550_000);
    }

    #[test]
    fn spot_is_the_inverse_of_the_buy_price() {
        let price_e9 = 20_000_000_000u64;
        let mpdao = 2_942_857u128 * 1_000_000;
        let spot = token_out_for_mpdao(mpdao, 6, price_e9);
        assert_eq!(spot, 147_142_850_000);
        let offer = discounted_token_out(mpdao, 6, price_e9, 300);
        assert_eq!(offer, spot / 10);
    }

    #[test]
    fn usd_price_can_fall_immediately_and_rise_at_most_ten_percent() {
        let previous = 11_000u64;
        // mpDAO-per-token down is a higher USD price. 10% up stops at 10,000.
        assert_eq!(capped_mpdao_per_token(previous, 9_000, 1_000), 10_000);
        assert_eq!(capped_mpdao_per_token(previous, 10_000, 1_000), 10_000);
        // mpDAO-per-token up is a lower USD price and is accepted whole.
        assert_eq!(capped_mpdao_per_token(previous, 20_000, 1_000), 20_000);
    }

    #[test]
    fn a_same_day_crash_does_not_reset_the_rise_cap() {
        let mut days = Vec::new();
        accept_usdt_observation(&mut days, 0, 11_000);
        // Crash, then a pump, still on day 0. The pump is capped against the open, not the crash.
        accept_usdt_observation(&mut days, 0, 22_000);
        accept_usdt_observation(&mut days, 0, 1_000);
        assert_eq!(days.len(), 1);
        assert_eq!(days[0].mpdao_per_token_e9, 10_000);
        assert_eq!(days[0].cap_base_e9, 11_000);
    }

    #[test]
    fn rising_price_pays_the_median_and_a_drop_pays_today() {
        let mut days = Vec::new();
        accept_usdt_observation(&mut days, 0, 11_000);
        // Next day the market tries to double the USD price (halve mpDAO-per-token).
        accept_usdt_observation(&mut days, 1, 5_000);
        assert_eq!(days[1].mpdao_per_token_e9, 10_000);
        // Today is 10,000 and the median of [11,000, 10,000] is 10,500.
        // The larger mpDAO-per-token is the lower USD payout.
        assert_eq!(protective_mpdao_per_token(&days), 10_500);

        accept_usdt_observation(&mut days, 2, 40_000);
        assert_eq!(days[2].mpdao_per_token_e9, 40_000);
        assert_eq!(protective_mpdao_per_token(&days), 40_000);
    }

    #[test]
    fn history_keeps_seven_days() {
        let mut days = Vec::new();
        for day in 0..9 {
            accept_usdt_observation(&mut days, day, 10_000 + day * 100);
        }
        assert_eq!(days.len(), 7);
        assert_eq!(days[0].day_index, 2);
        assert_eq!(days[6].day_index, 8);
    }

    #[test]
    #[should_panic(expected = "USDT price day is older than the stored day")]
    fn an_older_day_is_rejected() {
        let mut days = Vec::new();
        accept_usdt_observation(&mut days, 5, 10_000);
        accept_usdt_observation(&mut days, 4, 10_000);
    }
}
