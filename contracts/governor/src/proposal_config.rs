use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    panic_with_error, vec, Env, Val, Vec,
};

use crate::{
    errors::GovernorError,
    storage,
    types::{Calldata, ProposalAction, ProposalConfig},
};

impl ProposalConfig {
    /// Execute the proposal based on the configuration
    pub fn execute(&self, e: &Env) {
        match self.action {
            ProposalAction::Calldata(ref calldata) => {
                let auth_vec = build_auth_vec(e, &calldata.auths);
                e.authorize_as_current_contract(auth_vec);
                e.invoke_contract::<Val>(
                    &calldata.contract_id,
                    &calldata.function,
                    calldata.args.clone(),
                );
            }
            ProposalAction::Settings(ref settings) => {
                storage::set_settings(e, settings);
            }
            ProposalAction::Upgrade(ref wasm_hash) => {
                e.deployer().update_current_contract_wasm(wasm_hash.clone());
            }
            ProposalAction::Snapshot => {
                panic_with_error!(e, GovernorError::InvalidProposalType)
            }
        }
    }

    /// Check if the proposal is executable
    pub fn is_executable(&self) -> bool {
        match self.action {
            ProposalAction::Calldata(_) => true,
            ProposalAction::Settings(_) => true,
            ProposalAction::Upgrade(_) => true,
            ProposalAction::Snapshot => false,
        }
    }
}

/// Create an vec of auth entries the contract needs to sign to execute a calldata proposal
fn build_auth_vec(e: &Env, auths: &Vec<Calldata>) -> Vec<InvokerContractAuthEntry> {
    let mut auth_vec: Vec<InvokerContractAuthEntry> = vec![&e];
    for auth in auths.iter() {
        let pre_auth_entry = InvokerContractAuthEntry::Contract(SubContractInvocation {
            context: ContractContext {
                contract: auth.contract_id,
                fn_name: auth.function,
                args: auth.args,
            },
            sub_invocations: build_auth_vec(&e, &auth.auths),
        });
        auth_vec.push_back(pre_auth_entry);
    }
    auth_vec
}

#[cfg(test)]
mod test {
    use super::build_auth_vec;
    use crate::types::Calldata;
    use soroban_sdk::{
        auth::InvokerContractAuthEntry, testutils::Address as _, vec, Address, Env, IntoVal,
        Symbol, Vec,
    };

    #[test]
    fn test_build_auth_vec() {
        let e = Env::default();
        let inner_subcall_address = Address::generate(&e);
        let token_address = Address::generate(&e);
        let governor_address = Address::generate(&e);
        let call_amount: i128 = 100 * 10i128.pow(7);

        let sub_calldata: Vec<Calldata> = vec![
            &e,
            Calldata {
                contract_id: inner_subcall_address.clone(),
                function: Symbol::new(&e, "subcall"),
                args: (call_amount.clone(),).into_val(&e),
                auths: vec![
                    &e,
                    Calldata {
                        contract_id: token_address.clone(),
                        function: Symbol::new(&e, "transfer"),
                        args: (
                            governor_address.clone(),
                            inner_subcall_address.clone(),
                            call_amount.clone(),
                        )
                            .into_val(&e),
                        auths: vec![&e],
                    },
                ],
            },
        ];
        let sub_auth = build_auth_vec(&e, &sub_calldata);
        assert_eq!(sub_auth.len(), 1);
        match sub_auth.get_unchecked(0) {
            InvokerContractAuthEntry::Contract(sub_invocation) => {
                assert_eq!(sub_invocation.context.contract, inner_subcall_address);
                assert_eq!(sub_invocation.context.fn_name, Symbol::new(&e, "subcall"));
                assert_eq!(
                    sub_invocation.context.args,
                    (call_amount.clone(),).into_val(&e)
                );
                assert_eq!(sub_invocation.sub_invocations.len(), 1);
                match sub_invocation.sub_invocations.get(0) {
                    Some(InvokerContractAuthEntry::Contract(sub_invocation)) => {
                        assert_eq!(sub_invocation.context.contract, token_address);
                        assert_eq!(sub_invocation.context.fn_name, Symbol::new(&e, "transfer"));
                        assert_eq!(
                            sub_invocation.context.args,
                            (
                                governor_address.clone(),
                                inner_subcall_address.clone(),
                                call_amount.clone()
                            )
                                .into_val(&e)
                        );
                        assert_eq!(sub_invocation.sub_invocations.len(), 0);
                    }
                    _ => assert!(false, "Expected sub_invocation"),
                }
            }
            _ => assert!(false, "Expected sub_invocation"),
        }
    }
}
