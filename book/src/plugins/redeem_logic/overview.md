# Redeem Logic Plugins

Redeem Logic plugins are used to calculate the payoff between the senior and junior sides. Given the old senior and junior deposits sides and the reserve_fair_value fetched from the rate_plugin the redeem_logic execute the logic inside and give back the new quantities owned by the senior and the junior side. Additionally in cases where the redeem_logic support it also a fee componened is added.

Differently from the rate_plugin for redeem_logics the plugin is called directly from vyper via CPI, with some input. Then the output returned from the plugin is used for further calculations on vyper. Because of that is important that the redeem logics stick to a precise interface declaration, a minimal redeem_logic plugin is at least as follow:

```rust

pub struct RedeemLogicExecuteInput {
    pub old_quantity: [u64; 2],
    pub old_reserve_fair_value: [[u8; 16]; 10],
    pub new_reserve_fair_value: [[u8; 16]; 10],
}

pub struct RedeemLogicExecuteResult {
    pub new_quantity: [u64; 2],
    pub fee_quantity: u64,
}

pub fn execute(
        ctx: Context<ExecuteContext>,
        input_data: RedeemLogicExecuteInput,
    ) -> Result<()> {
        input_data.is_valid()?;

        let result: RedeemLogicExecuteResult = execute_plugin(
            // ...
        )?;

        anchor_lang::solana_program::program::set_return_data(&result.try_to_vec()?);

        Ok(())
    }

```

Additionally plugins could have a state that can condition the `execute` result. Check the following plugins for deeper insights and examples.
