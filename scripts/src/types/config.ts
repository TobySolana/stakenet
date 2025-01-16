import { struct, blob, u16, f64, u8, u32 } from "@solana/buffer-layout";
import { u128, publicKey, bool, u64 } from "@solana/buffer-layout-utils";
import { PublicKey } from "@solana/web3.js";

export interface StewardConfig {
  discriminator: bigint;
  stakePool: PublicKey;
  validatorList: PublicKey;
  admin: PublicKey;
  parametersAuthority: PublicKey;
  blacklistAuthority: PublicKey;
  validatorHistoryBlacklist: Uint8Array;
  parameter: StewardParameter;
  paused: boolean;
  padding: Uint8Array;
}

export interface StewardParameter {
  mev_commission_range: number;
  epoch_credits_range: number;
  commission_range: number;
  mev_commission_bps_threshold: number;
  scoring_delinquency_threshold_ratio: number;
  instant_unstake_delinquency_threshold_ratio: number;
  commission_threshold: number;
  historical_commission_threshold: number;
  _padding_0: Uint8Array;
  num_delegation_validators: number;
  scoring_unstake_cap_bps: number;
  instant_unstake_cap_bps: number;
  stake_deposit_unstake_cap_bps: number;
  compute_score_slot_range: bigint;
  instant_unstake_epoch_progress: number;
  instant_unstake_inputs_epoch_progress: number;
  num_epochs_between_scoring: bigint;
  minimum_stake_lamports: bigint;
  minimum_voting_epochs: bigint;
  _padding_1: Uint8Array;
}

export const StewardConfigLayout = struct<StewardConfig>([
  u64("discriminator"),
  publicKey("stakePool"),
  publicKey("validatorList"),
  publicKey("admin"),
  publicKey("parametersAuthority"),
  publicKey("blacklistAuthority"),
  blob(2504, "validator_history_blacklist"),
  struct<StewardParameter>(
    [
      u16("mev_commission_range"),
      u16("epoch_credits_range"),
      u16("commission_range"),
      u16("mev_commission_bps_threshold"),
      f64("scoring_delinquency_threshold_ratio"),
      f64("instant_unstake_delinquency_threshold_ratio"),
      u8("commission_threshold"),
      u8("historical_commission_threshold"),
      blob(6, "_padding_0"),
      u32("num_delegation_validators"),
      u32("scoring_unstake_cap_bps"),
      u32("instant_unstake_cap_bps"),
      u32("stake_deposit_unstake_cap_bps"),
      u64("compute_score_slot_range"),
      f64("instant_unstake_epoch_progress"),
      f64("instant_unstake_inputs_epoch_progress"),
      u64("num_epochs_between_scoring"),
      u64("minimum_stake_lamports"),
      u64("minimum_voting_epochs"),
      blob(256, "_padding_1"),
    ],
    "parameter"
  ),
  bool("paused"),
  blob(1023, "padding"),
]);
