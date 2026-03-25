#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

mod analytics;
mod nebula_explorer;
pub mod resource_minter;
mod ship_registry;

pub use analytics::{AnalyticsError, GlobalStats, LeaderboardEntry};
pub use nebula_explorer::{
    calculate_rarity_tier, compute_layout_hash, generate_nebula_layout, CellType, NebulaCell,
    NebulaLayout, Rarity, GRID_SIZE, TOTAL_CELLS,
};
pub use resource_minter::{ResourceError, ResourceType, StakeRecord, Config, LEDGERS_PER_DAY};
pub use ship_registry::Ship;

#[contract]
pub struct NebulaNomadContract;

#[contractimpl]
impl NebulaNomadContract {
    /// Generate a 16×16 procedural nebula map using ledger-seeded PRNG.
    ///
    /// Combines the supplied `seed` with on-chain ledger sequence and
    /// timestamp. The player must authorize the call.
    pub fn generate_nebula_layout(
        env: Env,
        seed: BytesN<32>,
        player: Address,
    ) -> NebulaLayout {
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
    ///
    /// Also updates the on-chain analytics counters (total_scans,
    /// total_essence_accrued) and registers the player for the leaderboard.
    pub fn scan_nebula(
        env: Env,
        seed: BytesN<32>,
        player: Address,
    ) -> (NebulaLayout, Rarity) {
        player.require_auth();

        let layout = nebula_explorer::generate_nebula_layout(&env, &seed, &player);
        let rarity = nebula_explorer::calculate_rarity_tier(&env, &layout);
        let layout_hash = nebula_explorer::compute_layout_hash(&env, &layout);

        nebula_explorer::emit_nebula_scanned(&env, &player, &layout_hash, &rarity);

        // Record analytics: use total_energy as the essence earned this scan.
        analytics::record_scan(&env, &player, layout.total_energy as u64);

        (layout, rarity)
    }

    /// Return aggregate global statistics (total scans, ships minted, etc.).
    ///
    /// Pure view — no ledger writes, zero gas cost beyond the read.
    pub fn get_global_stats(env: Env) -> GlobalStats {
        analytics::get_global_stats(&env)
    }

    /// Return the top-`top_n` explorers sorted by cumulative cosmic essence.
    ///
    /// Emits a `LeaderboardSnapshot` event so frontends can subscribe via
    /// Stellar event streams.  Returns `Err(InvalidTopN)` when `top_n` is 0
    /// or exceeds 50.
    pub fn snapshot_leaderboard(
        env: Env,
        top_n: u32,
    ) -> Result<Vec<LeaderboardEntry>, AnalyticsError> {
        analytics::snapshot_leaderboard(&env, top_n)
    }
}

