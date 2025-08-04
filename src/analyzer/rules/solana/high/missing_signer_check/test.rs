use crate::analyzer::rules::solana::high::missing_signer_check::filters::has_missing_signer_checks;
use syn::{ItemStruct, parse_quote};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerable_account_info() {
        let struct_def: ItemStruct = parse_quote! {
            #[derive(Accounts)]
            pub struct VulnerableStruct<'info> {
                pub authority: AccountInfo<'info>,
            }
        };
        
        assert!(has_missing_signer_checks(&struct_def), 
                "Should detect AccountInfo without signer constraint");
    }

    #[test]
    fn test_safe_signer_field() {
        let struct_def: ItemStruct = parse_quote! {
            #[derive(Accounts)]
            pub struct SafeStruct<'info> {
                #[account(signer)]
                pub authority: AccountInfo<'info>,
            }
        };
        
        assert!(!has_missing_signer_checks(&struct_def), 
                "Should not detect AccountInfo with signer constraint");
    }

    #[test]
    fn test_vulnerable_unchecked_account() {
        let struct_def: ItemStruct = parse_quote! {
            #[derive(Accounts)]
            pub struct VulnerableStruct<'info> {
                pub admin: UncheckedAccount<'info>,
            }
        };
        
        assert!(has_missing_signer_checks(&struct_def), 
                "Should detect UncheckedAccount without signer constraint");
    }

    #[test]
    fn test_proper_signer_type() {
        let struct_def: ItemStruct = parse_quote! {
            #[derive(Accounts)]
            pub struct SafeStruct<'info> {
                pub proper_signer: Signer<'info>,
            }
        };
        
        assert!(!has_missing_signer_checks(&struct_def), 
                "Should not detect Signer<'info> type as vulnerable");
    }

    #[test]
    fn test_mixed_fields() {
        let struct_def: ItemStruct = parse_quote! {
            #[derive(Accounts)]
            pub struct MixedStruct<'info> {
                pub proper_signer: Signer<'info>,
                pub vulnerable_account: AccountInfo<'info>,
                #[account(signer)]
                pub safe_account: AccountInfo<'info>,
            }
        };
        
        assert!(has_missing_signer_checks(&struct_def), 
                "Should detect vulnerable field even with safe fields present");
    }

    #[test]
    fn test_account_loader_safe() {
        let struct_def: ItemStruct = parse_quote! {
            #[derive(Accounts)]
            pub struct SafeStruct<'info> {
                pub data: AccountLoader<'info, MyData>,
            }
        };
        
        assert!(!has_missing_signer_checks(&struct_def), 
                "Should not detect AccountLoader as vulnerable");
    }

    #[test]
    fn test_empty_struct() {
        let struct_def: ItemStruct = parse_quote! {
            #[derive(Accounts)]
            pub struct EmptyStruct<'info> {}
        };
        
        assert!(!has_missing_signer_checks(&struct_def), 
                "Should not detect empty struct as vulnerable");
    }
}
