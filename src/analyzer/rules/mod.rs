pub mod solana;

use crate::analyzer::Result;
use crate::analyzer::engine::RuleEngine;


/// Register all built-in rules in the rule engine
pub fn register_builtin_rules(engine: &mut RuleEngine) -> Result<()> {
    // Register Solana rules
    register_solana_rules(engine)?;

    Ok(())
}

/// Register Solana specific rules
fn register_solana_rules(engine: &mut RuleEngine) -> Result<()> {
    // High severity rules
    engine.add_rule(solana::high::unsafe_code::create_rule());
    engine.add_rule(solana::high::missing_signer_check::create_rule());

    // Medium severity rules
    engine.add_rule(solana::medium::duplicate_mutable_accounts::create_rule());
    engine.add_rule(solana::medium::division_by_zero::create_rule());
    engine.add_rule(solana::medium::owner_check::create_rule());

    // Low severity rules
    engine.add_rule(solana::low::missing_error_handling::create_rule());
    engine.add_rule(solana::low::anchor_instructions::create_rule());

    Ok(())
}
