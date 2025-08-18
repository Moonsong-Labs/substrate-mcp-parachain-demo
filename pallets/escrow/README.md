# Escrow Pallet

A Substrate pallet for secure peer-to-peer transactions with escrow functionality.

## Overview

The Escrow Pallet enables trustless transactions between parties by holding funds in escrow until conditions are met. It supports optional third-party arbitration for dispute resolution and automatic timeout handling.

## Features

- **Secure Fund Locking**: Funds are locked in the pallet until released or refunded
- **Optional Arbitration**: Third-party arbitrators can resolve disputes
- **Automatic Expiry**: Buyers can reclaim funds after deadline
- **Configurable Fees**: Platform and arbitrator fees are configurable
- **Multi-Currency Support**: Works with any Substrate currency implementation

## Key Concepts

### Roles

- **Buyer**: Creates escrow and deposits funds
- **Seller**: Receives funds upon successful completion
- **Arbitrator** (optional): Resolves disputes between parties

### Escrow Lifecycle

1. **Creation**: Buyer creates escrow with seller details, amount, and deadline
2. **Active**: Funds are locked, waiting for completion
3. **Resolution**:
   - **Released**: Buyer confirms, funds go to seller
   - **Refunded**: Seller initiates refund
   - **Expired**: Deadline passed, buyer reclaims
   - **Disputed**: Conflict raised, arbitrator decides

## Extrinsics

- `create_escrow`: Create a new escrow transaction
- `release_funds`: Buyer releases funds to seller
- `refund`: Seller refunds buyer
- `claim_expired`: Buyer reclaims after deadline
- `raise_dispute`: Either party raises dispute
- `resolve_dispute`: Arbitrator resolves dispute
- `set_arbitrator_fee`: Arbitrator sets their fee

## Storage

- `Escrows`: Main escrow information storage
- `Disputes`: Active dispute details
- `UserEscrows`: Index of user's escrows
- `ArbitratorFees`: Custom arbitrator fee rates
- `EscrowCount`: Total escrows created
- `ActiveEscrowCount`: Active escrows per user

## Configuration

```rust
trait Config {
    // Currency for transactions
    type Currency: ReservableCurrency<Self::AccountId>;
    
    // Time provider
    type Time: Time;
    
    // Pallet ID for account derivation
    type PalletId: Get<PalletId>;
    
    // Amount limits
    type MinimumEscrowAmount: Get<BalanceOf<Self>>;
    type MaximumEscrowAmount: Get<BalanceOf<Self>>;
    
    // Time limits
    type MinimumDeadline: Get<MomentOf<Self>>;
    type MaximumDeadline: Get<MomentOf<Self>>;
    
    // Text limits
    type MaxDescriptionLength: Get<u32>;
    type MaxDisputeReasonLength: Get<u32>;
    
    // User limits
    type MaxActiveEscrowsPerUser: Get<u32>;
    
    // Fee configuration
    type PlatformFeePercent: Get<Percent>;
    type DefaultArbitratorFeePercent: Get<Percent>;
}
```

## Testing

Run tests with:

```bash
cargo test -p pallet-escrow
```

## Benchmarking

Generate weights with:

```bash
cargo build --release --features runtime-benchmarks
./target/release/node-template benchmark pallet \
    --chain dev \
    --pallet pallet_escrow \
    --extrinsic "*" \
    --steps 50 \
    --repeat 20 \
    --output pallets/escrow/src/weights.rs
```

## License

MIT-0