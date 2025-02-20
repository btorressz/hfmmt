# hfmmt
# 📊 **High-Frequency Market-Making Token (HFMMT) - Solana Smart Contract(program)**

## 📝 **Overview**
The **High-Frequency Market-Making Token (HFMMT)** is a **staking and liquidity provisioning token** designed to **reward market makers** based on their contributions to liquidity, execution volume, and spread efficiency.

The contract(program) is implemented using **Anchor for Solana** using **Solana Playground** and follows **efficient market-making principles**.

## **Key Features**
- 📌 **Incentivized Staking & Liquidity Provisioning:**  
  - Market makers stake HFMMT tokens to gain access to **low-latency execution privileges**.
  - Liquidity providers earn HFMMT rewards based on **tight spreads and order book contributions**.

- 📌 **Dynamic Reward Adjustments:**  
  - Rewards change **based on market volatility** to encourage deeper liquidity provisioning.

- 📌 **Market Efficiency Mechanisms:**  
  - Fee rebates for market makers.
  - Slashing mechanisms for inactivity.
  - Anti-front-running and Sybil resistance.

---

## **Program Instructions**
### **1️⃣ Initialize Configuration**
**Function:** `initialize(ctx, config_params)`  
- Sets up global parameters:
  - `reward_rate` (how rewards are distributed).
  - `volatility_threshold` (used to adjust incentives dynamically).
- This function **must** be called once by an admin to initialize the contract.

  ### **2️⃣ Staking HFMMT Tokens**
**Function:** `stake_tokens(ctx, amount)`  
- Allows a **market maker** to stake tokens.
- Staking enables **priority execution and rewards eligibility**.
- Tokens are transferred from the **staker's account** to the **vault**.

  ### **3️⃣ Updating Market Maker Performance**
**Function:** `update_performance(ctx, execution_volume, spread_efficiency, order_flow)`  
- Market makers' rewards are based on:
  - **Execution volume**
  - **Spread efficiency**
  - **Order flow contributions**
- Higher activity results in **higher reward payouts**.

  ### **4️⃣ Distributing Rewards**
**Function:** `distribute_rewards(ctx)`  
- Calculates and distributes **staking and market-making rewards**.
- If **auto-compound** is enabled, rewards are **automatically staked**.

### **5️⃣ Liquidity Provisioning & Withdrawals**
- **Provide Liquidity:** `provide_liquidity(ctx, amount)`  
- **Withdraw Liquidity:** `withdraw_liquidity(ctx, amount)`  

- Liquidity providers **earn HFMMT rewards** based on the **duration of liquidity commitment**.

  ### **6️⃣ Fee Rebates for Market Makers**
**Function:** `claim_fee_rebate(ctx)`  
- Market makers receive **rebates on trading fees** if they contribute significantly to liquidity.
- **Rebate amount** is proportional to **execution volume**.

### **7️⃣ Slashing for Inactivity**
**Function:** `enforce_activity_slashing(ctx)`  
- If a market maker **remains inactive for 30+ days**, **10% of their staked tokens** are slashed.
- Prevents passive staking without contributing to market efficiency.

### **8️⃣ Flash Loan Resistance for Liquidity Providers**
**Function:** `claim_liquidity_rewards(ctx)`  
- Liquidity rewards can **only be claimed after 7 days**, preventing **flash loan abuse**.

### **9️⃣ Time-Based Liquidity Rewards**
**Function:** `lock_liquidity(ctx, lock_duration)`  
- Liquidity providers **earn bonus multipliers** if they **lock their liquidity**:
  - **1-month lock → 1x rewards**
  - **3-month lock → 1.5x rewards**
  - **6-month lock → 2x rewards**





