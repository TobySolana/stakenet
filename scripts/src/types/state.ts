import { struct, blob, u16, u8, u32, seq } from "@solana/buffer-layout";
import { bool, u64 } from "@solana/buffer-layout-utils";

export interface StewardState {
  state_tag: bigint;
  validator_lamport_balances: bigint[];
  scores: number[];
  sorted_score_indices: number[];
  yield_scores: number[];
  sorted_yield_score_indices: number[];

  delegations: Delegation[];

  instant_unstake: Uint8Array;
  progress: Uint8Array;
  validators_for_immediate_removal: Uint8Array;
  validators_to_remove: Uint8Array;

  start_computing_scores_slot: bigint;
  current_epoch: bigint;
  next_cycle_epoch: bigint;
  num_pool_validators: bigint;
  scoring_unstake_total: bigint;
  instant_unstake_total: bigint;
  stake_deposit_unstake_total: bigint;
  status_flags: number;
  validators_added: number;

  _padding: Uint8Array;
}

export interface Delegation {
  numerator: number;
  denominator: number;
}

export interface StewardStateAccount {
  discriminator: bigint;
  state: StewardState;
  is_initialized: boolean;
  bump: number;
  _padding: Uint8Array;
}

export const StewardStateAccountLayout = struct<StewardStateAccount>([
  u64("discriminator"),
  struct<StewardState>(
    [
      u64("state_tag"),
      seq(u64("value"), 5_000, "validator_lamport_balances"),
      seq(u32("value"), 5_000, "scores"),
      seq(u16("value"), 5_000, "sorted_score_indices"),
      seq(u32("value"), 5_000, "yield_scores"),
      seq(u16("value"), 5_000, "sorted_yield_score_indices"),
      seq(
        struct<Delegation>([u32("numerator"), u32("denominator")]),
        5_000,
        "delegations"
      ),
      blob(632, "instant_unstake"),
      blob(632, "progress"),
      blob(632, "validators_for_immediate_removal"),
      blob(632, "validators_to_remove"),
      u64("start_computing_scores_slot"),
      u64("current_epoch"),
      u64("next_cycle_epoch"),
      u64("num_pool_validators"),
      u64("scoring_unstake_total"),
      u64("instant_unstake_total"),
      u64("stake_deposit_unstake_total"),
      u32("status_flags"),
      u16("validators_added"),
      blob(40002, "_padding0"),
    ],
    "state"
  ),
  bool("is_initialized"),
  u8("bump"),
  blob(6, "_padding"),
]);
