use crate::types::SubCalldata;
use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    vec, Env, Vec,
};

pub fn create_sub_auth(e: &Env, sub_auth: &Vec<SubCalldata>) -> Vec<InvokerContractAuthEntry> {
    let mut sub_auth_vec = vec![&e];
    for call_data in sub_auth.iter() {
        let pre_auth_entry = InvokerContractAuthEntry::Contract(SubContractInvocation {
            context: ContractContext {
                contract: call_data.contract_id,
                fn_name: call_data.function,
                args: call_data.args,
            },
            sub_invocations: create_sub_auth(&e, &call_data.sub_auth.clone()),
        });
        sub_auth_vec.push_back(pre_auth_entry);
    }
    sub_auth_vec
}

#[cfg(test)]
mod test {
    use super::create_sub_auth;
    use crate::types::SubCalldata;
    use soroban_sdk::{
        auth::InvokerContractAuthEntry, testutils::Address as _, vec, Address, Env, IntoVal,
        Symbol, Vec,
    };

    #[test]
    fn test_create_sub_auth() {
        let e = Env::default();
        let inner_subcall_address = Address::generate(&e);
        let token_address = Address::generate(&e);
        let governor_address = Address::generate(&e);
        let call_amount: i128 = 100 * 10i128.pow(7);

        let sub_calldata: Vec<SubCalldata> = vec![
            &e,
            SubCalldata {
                contract_id: inner_subcall_address.clone(),
                function: Symbol::new(&e, "subcall"),
                args: (call_amount.clone(),).into_val(&e),
                sub_auth: vec![
                    &e,
                    SubCalldata {
                        contract_id: token_address.clone(),
                        function: Symbol::new(&e, "transfer"),
                        args: (
                            governor_address.clone(),
                            inner_subcall_address.clone(),
                            call_amount.clone(),
                        )
                            .into_val(&e),
                        sub_auth: vec![&e],
                    },
                ],
            },
        ];
        let sub_auth = create_sub_auth(&e, &sub_calldata);
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
                    _ => panic!("Expected sub_invocation"),
                }
            }
            _ => panic!("Expected sub_invocation"),
        }
    }
}
