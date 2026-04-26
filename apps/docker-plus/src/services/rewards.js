// Simulated reward service for Docker++
// Payout Destination: Dynamic from environment variable

const REWARD_AMOUNT_SATS = 10000000; // 10 million satoshis

async function triggerReward(recipientAddress, reason) {
    // Priority: 1. Address passed to function, 2. Env variable, 3. Hardcoded fallback
    const finalRecipient = recipientAddress || process.env.BITCOIN_WALLET || "bc1qs4rr5prccv63fsws2yx6c8pctx50uzkvc8yyhj";
    
    console.log(`[REWARD] Triggered ${REWARD_AMOUNT_SATS} sats to ${finalRecipient} for: ${reason}`);
    
    // In a production environment, this would interface with a 
    // Lightning Network node or a custodial service API to execute the transfer.
    
    return {
        success: true,
        amount: REWARD_AMOUNT_SATS,
        recipient: finalRecipient,
        txid: `simulated_${Date.now()}`
    };
}

module.exports = { triggerReward };
