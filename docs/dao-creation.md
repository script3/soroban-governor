# DAO Creation Guide

## Overview

Soroban Governer is a Smart Contract system that allows for the creation and management of Decentralized Autonomous Organizations (DAOs). This guide will walk you through the process of creating a DAO using the Soroban Governor CLI and interacting with it using the Soroban Governor interface.

Soroban Governor is based heavily on OpenZeppelin's Governor Alpha contract. As such, the process of creating and managing a DAO using Soroban Governor is very similar to that of OpenZeppelin's Governor Alpha.

## Deployment

TODO: should we reccomend the CLI for this?

## Initialization

Now that you've deployed your Governor and Votes contracts, you need to initialize them.

The Votes contract can be initialized with the `initialize()` function. This function takes a single argument, `token`, which is the address of the token contract the DAO uses for voting.

The Governor contract is also initialized with the `initialize()` function. This function takes two arguments:

`votes` - which is the address of the Votes contract you deployed earlier.

and

`Governor Settings` - which is a struct that contains the following fields:
| Field | Type | Explanation |
|--------------------|-------|-----------------------------------------------------------------------------------------------------------------------------------------|
| proposal_threshold | i128 | The votes required to create a proposal. |
| vote_delay | u64 | The delay (in seconds) from the proposal creation to when the voting period begins. |
| vote_period | u64 | The time (in seconds) the proposal will be open to vote against. |
| timelock | u64 | The time (in seconds) the proposal will have to wait between vote period closing and execution. |
| quorum | u32 | The percentage of votes (expressed in BPS) needed of the total available votes to consider a vote successful. |
| counting_type | u32 | Determine which votes to count against the quorum out of for, against, and abstain. |
| | | The value is encoded such that only the last 3 bits are considered, and follows the structure `MSB...{for}{against}{abstain}`, |
| | | such that any value != 0 means that type of vote is counted in the quorum. |
| | | For example, consider 5 == `0x0...0101`, this means that votes "for" and "abstain" are included in the quorum, but votes "against" are not. |
| vote_threshold | u32 | The percentage of votes "yes" (expressed in BPS) needed to consider a vote successful. |

### Settings Overview

_Proposal Threshold_ - The proposal threshold should be set based on how important it is to filter spam proposals. Setting this value too high makes the DAO less decentralized since less holders can create proposals - however if this is set too low, the DAO may be spammed with proposals. When setting this you should consider how widely the token is distributed, and how much token inflation you expect.

_Vote Delay_ - The vote delay is the time between when a proposal is created and when the voting period begins. This is to allow time for the community to review the proposal before voting begins. 1-3 days is a typical range for this value.

_Vote Period_ - The vote period is the time that the proposal will be open to vote against. This should be set based on how long it takes for the community to review and vote on proposals. If the vote period is too short not all voter's will have time to vote, however if it's too long it can delay decisions that should be implemented quickly (like risk parameter updates). 3-7 days is a typical range for this value.

_Timelock_ - The timelock is the time that the proposal will have to wait between the vote period closing and execution. This is to allow time for the community to review the results of the vote before the proposal is executed. If the timelock is too short user's may not have time to respond to the agreed upon changes (by withdrawing from a lending protocol for instance), however, if it's too long it can delay decisions that should be implemented quickly (like risk parameter updates). 1-3 days is a typical range for this value.

_Quorum_ - The quorum parameter represents the minimum number or percentage of participants required for a vote to be considered valid. It ensures that a sufficient number of stakeholders are actively participating in the decision-making process. The value of quorum should be set based on the size of your community and the percent of them that actively vote. It should strike a balance between ensuring broad participation and avoiding situations where quorum is frequently not met due to voter apathy. 5-10% is a typical range for this value.

_Counting Type_ - The counting type parameter determines which types of votes are counted against the quorum. It allows you to specify whether "for," "against," or "abstain" votes should be included in the quorum calculation. There is little real difference between counting _for_ and _against_ votes, but including or excluding _abstain_ votes is a good way to either require quorums to take a stance on a proposal, or simply require quorums to represent voter awareness of the proposal.

_Vote Threshold_: The vote threshold parameter represents the minimum percentage of "yes" votes needed for a proposal to pass. This is normally set at 50%, but can be adjusted higher or lower to require stronger or weaker consensus on decisions. When setting a threshold above 50% it's important to consider how this will give minority decision voters more power, and how this will affect the DAO's ability to make decisions. Thresholds of >50% could result in a small group stonewalling DAO decisions.

Now that you've initialized your Governor and Votes contracts, you're ready to start using your DAO!

## Making Your First Proposal

To make a proposal, you'll need to call the `propose()` function on the Governor contract. This function takes 5 parameters:

| Parameter    | Type             | Description                                                     |
| ------------ | ---------------- | --------------------------------------------------------------- |
| creator      | Address          | The address of the account creating the proposal                |
| calldata     | Calldata         | The calldata to execute when the proposal is executed           |
| sub_calldata | Vec<SubCalldata> | The sub calldata to pre-authorize when the proposal is executed |
| title        | String           | The title of the proposal                                       |
| description  | String           | The description of the proposal                                 |

### Parameters Overview

_Creator_ - The address of the account creating the proposal. This is your address.

_Calldata_ - The calldata to execute when the proposal is executed. CallData is a struct with the following arguments:

| Field       | Type     | Description                                      |
| ----------- | -------- | ------------------------------------------------ |
| contract_id | Address  | The address of the contract to call              |
| function    | Symbol   | The name of the function to call on the contract |
| args        | Vec<Val> | The arguments to pass to the function            |

The call data parameter governs what the proposal will do when it is executed. This can be a call to a contract to change a parameter, a transfer of funds, or any other action that can be executed on the blockchain.

_Sub Calldata_ - The sub calldata to pre-authorize when the proposal is executed. This is a list of SubCallData structs, each with the following arguments:

| Field       | Type             | Explanation                                          |
| ----------- | ---------------- | ---------------------------------------------------- |
| contract_id | Address          | The address of the contract being called             |
| function    | Symbol           | The name of the function being call on that contract |
| args        | Vec<Val>         | The arguments to pass to the function                |
| sub_auth    | Vec<SubCalldata> | The sub calldata to pre-authorize                    |

Sub calldata is a way to authorize actions the function being called by the proposal needs to carry out. For example, if the DAO is calling a `deposit()` function to deposit a portion of the DAO's treasury in a lending pool the sub calldata must authorize the `transfer()` call the lending pool contract needs to make.

Once this function is called successfully (the caller must have sufficient voting power) a proposal_id u32 value will be returned and broadcasted. This id can be used to vote on the proposal.

## Voting

Once a proposal has been created, it will be open for voting for the duration of the vote period. To vote on a proposal, you'll need to call the `vote()` function on the Votes contract. This function takes 3 parameters:

| Parameter   | Type    | Description                       |
| ----------- | ------- | --------------------------------- |
| voter       | Address | The address of the account voting |
| proposal_id | u32     | The id of the proposal to vote on |
| support     | u32     | The vote to cast:                 |
|             |         | - 0 to vote abstain               |
|             |         | - 1 to vote against               |
|             |         | - 2 to vote for                   |

## Executing a Proposal

Once a Proposal as been voted on to finalize voting you must call the `close()` function on the Governor contract. This function takes a single parameter, `proposal_id`, which is the id of the proposal to close.

This function will stop any further votes from being cast on the proposal, and will check if the proposal has passed, and if quorum was met. If the proposal has passed, it will be queued for execution.

After the queue period (defined by the Governor settings) has passed you can call the `execute()` function on the Governor contract. This function takes a single parameter, `proposal_id`, which is the id of the proposal to execute. This function will carry out the actions specified in the proposal's calldata.

Congratulations! You've now created and executed a proposal using Soroban Governor.
