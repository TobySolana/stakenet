import { Connection, PublicKey } from "@solana/web3.js";
import { SOLANA_RPC_URL } from "./constants";
import { StewardConfigLayout, StewardStateAccountLayout } from "./types";

(async () => {
  const connection = new Connection(SOLANA_RPC_URL);
  const stewardConfigAccount = new PublicKey(
    "jitoVjT9jRUyeXHzvCwzPgHj7yWNRhLcUoXtes4wtjv"
  );
  const stewardStateAccount = new PublicKey(
    "9BAmGVLGxzqct6bkgjWmKSv3BFB6iKYXNBQp8GWG1LDY"
  );
  const validatorHistoryAccount = new PublicKey(
    "3R3nGZpQs2aZo5FDQvd2MUQ6R7KhAPainds6uT6uE2mn"
  );

  // Fetch steward config
  try {
    const account = await connection.getAccountInfo(
      stewardConfigAccount,
      "confirmed"
    );
    const config = StewardConfigLayout.decode(account.data);
    console.log(config);
  } catch (ex) {
    console.log("fetch steward config failed", ex);
  }

  // Fetch steward state
  try {
    const account = await connection.getAccountInfo(
      stewardStateAccount,
      "confirmed"
    );
    const stateData = StewardStateAccountLayout.decode(account.data);
    console.log(stateData);
  } catch (ex) {
    console.log("fetch steward state failed", ex);
  }
})();
