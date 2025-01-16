import { PublicKey } from "@solana/web3.js";
import { default as bs58 } from "bs58";

import * as dotenv from "dotenv";
dotenv.config();

export const SOLANA_RPC_URL = process.env.RPC_URL;
