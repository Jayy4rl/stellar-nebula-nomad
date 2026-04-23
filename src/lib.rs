#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

mod bug_bounty_payout;
mod event_framework;
mod fleet_manager;
mod nebula_explorer;
mod onboarding_tutorial;
mod resource_minter;
mod ship_registry;

pub use bug_bounty_payout::{BountyConfig, BountyError, BugReport};
pub use event_framework::{EventFrameworkError, StandardEvent};
pub use fleet_manager::{Fleet, FleetError, FleetStatus, FleetTemplate};
pub use nebula_explorer::{
    calculate_rarity_tier, compute_layout_hash, generate_nebula_layout, CellType, NebulaCell,
    NebulaLayout, Rarity, GRID_SIZE, TOTAL_CELLS,
};
pub use onboarding_tutorial::{OnboardingError, PlayerProfile, TutorialProgress};
pub use resource_minter::Resource;
pub use ship_registry::Ship;

#[contract]
pub struct NebulaNomadContract;

#[contractimpl]
impl NebulaNomadContract {
    /// Generate a 16×16 procedural nebula map using ledger-seeded PRNG.
    ///
    /// Combines the supplied `seed` with on-chain ledger sequence and
    /// timestamp. The player must authorize the call.
    pub fn generate_nebula_layout(env: Env, seed: BytesN<32>, player: Address) -> NebulaLayout {
        player.require_auth();
        nebula_explorer::generate_nebula_layout(&env, &seed, &player)
    }

    /// Calculate the rarity tier of a nebula layout using on-chain
    /// verifiable math (no off-chain RNG).
    pub fn calculate_rarity_tier(env: Env, layout: NebulaLayout) -> Rarity {
        nebula_explorer::calculate_rarity_tier(&env, &layout)
    }

    /// Full scan: generates layout, calculates rarity, and emits a
    /// `NebulaScanned` event containing the layout hash.
    pub fn scan_nebula(env: Env, seed: BytesN<32>, player: Address) -> (NebulaLayout, Rarity) {
        player.require_auth();

        let layout = nebula_explorer::generate_nebula_layout(&env, &seed, &player);
        let rarity = nebula_explorer::calculate_rarity_tier(&env, &layout);
        let layout_hash = nebula_explorer::compute_layout_hash(&env, &layout);

        nebula_explorer::emit_nebula_scanned(&env, &player, &layout_hash, &rarity);

        (layout, rarity)
    }

    // ─── Onboarding tutorial ─────────────────────────────────────────────

    pub fn init_onboarding(env: Env, admin: Address) -> Result<(), OnboardingError> {
        onboarding_tutorial::init_onboarding(&env, &admin)
    }

    pub fn create_profile(env: Env, player: Address) -> Result<(), OnboardingError> {
        onboarding_tutorial::create_profile(&env, &player)
    }

    pub fn start_tutorial(env: Env, player: Address) -> Result<(), OnboardingError> {
        onboarding_tutorial::start_tutorial(&env, &player)
    }

    pub fn complete_tutorial_step(
        env: Env,
        player: Address,
        step_id: u32,
    ) -> Result<i128, OnboardingError> {
        onboarding_tutorial::complete_tutorial_step(&env, &player, step_id)
    }

    pub fn get_tutorial_progress(env: Env, player: Address) -> Option<TutorialProgress> {
        onboarding_tutorial::get_tutorial_progress(&env, &player)
    }

    pub fn get_starter_resources(env: Env, player: Address) -> i128 {
        onboarding_tutorial::get_starter_resources(&env, &player)
    }

    pub fn set_tutorial_path(
        env: Env,
        admin: Address,
        path: soroban_sdk::Vec<u32>,
    ) -> Result<(), OnboardingError> {
        onboarding_tutorial::set_tutorial_path(&env, &admin, path)
    }

    // ─── Bug bounty payout engine ────────────────────────────────────────

    pub fn init_bounty_engine(
        env: Env,
        admin: Address,
        approvers: soroban_sdk::Vec<Address>,
        approval_threshold: u32,
        high_value_threshold: i128,
        timelock_seconds: u64,
    ) -> Result<(), BountyError> {
        bug_bounty_payout::init_bounty_engine(
            &env,
            &admin,
            approvers,
            approval_threshold,
            high_value_threshold,
            timelock_seconds,
        )
    }

    pub fn fund_bounty_pool(env: Env, admin: Address, amount: i128) -> Result<i128, BountyError> {
        bug_bounty_payout::fund_bounty_pool(&env, &admin, amount)
    }

    pub fn submit_bug_report(
        env: Env,
        reporter: Address,
        description: soroban_sdk::String,
        severity: soroban_sdk::Symbol,
    ) -> Result<u64, BountyError> {
        bug_bounty_payout::submit_bug_report(&env, &reporter, description, severity)
    }

    pub fn approve_and_pay_bounty(
        env: Env,
        approver: Address,
        report_id: u64,
        amount: i128,
    ) -> Result<bool, BountyError> {
        bug_bounty_payout::approve_and_pay_bounty(&env, &approver, report_id, amount)
    }

    pub fn approve_and_pay_bounty_burst(
        env: Env,
        approver: Address,
        report_ids: soroban_sdk::Vec<u64>,
        amounts: soroban_sdk::Vec<i128>,
    ) -> Result<u32, BountyError> {
        bug_bounty_payout::approve_and_pay_bounty_burst(&env, &approver, report_ids, amounts)
    }

    pub fn set_emergency_pause(env: Env, admin: Address, paused: bool) -> Result<(), BountyError> {
        bug_bounty_payout::set_emergency_pause(&env, &admin, paused)
    }

    pub fn set_community_voted_mode(
        env: Env,
        admin: Address,
        enabled: bool,
    ) -> Result<(), BountyError> {
        bug_bounty_payout::set_community_voted_mode(&env, &admin, enabled)
    }

    pub fn get_report(env: Env, report_id: u64) -> Option<BugReport> {
        bug_bounty_payout::get_report(&env, report_id)
    }

    pub fn get_bounty_balance(env: Env, reporter: Address) -> i128 {
        bug_bounty_payout::get_bounty_balance(&env, &reporter)
    }

    pub fn get_bounty_pool(env: Env) -> i128 {
        bug_bounty_payout::get_bounty_pool(&env)
    }

    // ─── Standardized event framework ────────────────────────────────────

    pub fn init_event_framework(env: Env, admin: Address) {
        event_framework::init_event_framework(&env, &admin)
    }

    pub fn register_event_schema(
        env: Env,
        admin: Address,
        event_type: soroban_sdk::Symbol,
        version: u32,
    ) -> Result<(), EventFrameworkError> {
        event_framework::register_event_schema(&env, &admin, event_type, version)
    }

    pub fn emit_standard_event(
        env: Env,
        caller: Address,
        event_type: soroban_sdk::Symbol,
        payload: BytesN<256>,
    ) -> Result<u64, EventFrameworkError> {
        event_framework::emit_standard_event(&env, &caller, event_type, payload)
    }

    pub fn emit_standard_event_burst(
        env: Env,
        caller: Address,
        event_type: soroban_sdk::Symbol,
        payloads: soroban_sdk::Vec<BytesN<256>>,
    ) -> Result<u32, EventFrameworkError> {
        event_framework::emit_standard_event_burst(&env, &caller, event_type, payloads)
    }

    pub fn query_recent_events(
        env: Env,
        filter: soroban_sdk::Symbol,
        limit: u32,
    ) -> soroban_sdk::Vec<StandardEvent> {
        event_framework::query_recent_events(&env, filter, limit)
    }

    // ─── Fleet manager ────────────────────────────────────────────────────

    pub fn init_fleet_templates(env: Env) -> Result<(), FleetError> {
        fleet_manager::init_fleet_templates(&env)
    }

    pub fn register_ship_for_owner(env: Env, owner: Address, ship: Ship) -> Result<(), FleetError> {
        fleet_manager::register_ship_for_owner(&env, &owner, ship)
    }

    pub fn register_fleet(
        env: Env,
        owner: Address,
        ship_ids: soroban_sdk::Vec<u64>,
        template_id: u32,
    ) -> Result<Fleet, FleetError> {
        fleet_manager::register_fleet(&env, &owner, ship_ids, template_id)
    }

    pub fn sync_fleet_status(env: Env, fleet_id: u64) -> Result<FleetStatus, FleetError> {
        fleet_manager::sync_fleet_status(&env, fleet_id)
    }

    pub fn get_fleet(env: Env, fleet_id: u64) -> Option<Fleet> {
        fleet_manager::get_fleet(&env, fleet_id)
    }

    pub fn get_fleet_status(env: Env, fleet_id: u64) -> Option<FleetStatus> {
        fleet_manager::get_fleet_status(&env, fleet_id)
    }
}
