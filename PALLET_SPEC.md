# Escrow Pallet Specification

## Overview

The Escrow Pallet is a blockchain-based system that facilitates transactions between parties by acting as an automated intermediary. It ensures both buyers and sellers fulfill their obligations without requiring mutual trust.

### Core Functionality

The pallet operates as a decentralized escrow service where:

1. **Transaction Initiation**: A buyer creates an escrow by depositing funds into the pallet's secure account, specifying the seller, transaction amount and deadline. The funds become locked and inaccessible to all parties until specific conditions are met.

2. **Fulfillment Process**: The seller delivers goods or services knowing payment is guaranteed. Once the buyer confirms satisfaction, they can release the funds to the seller. This eliminates the seller's risk of non-payment and the buyer's risk of paying without receiving delivery.

3. **Automatic Safeguards**: The system includes timeout mechanisms where buyers can reclaim funds if deadlines pass without resolution, preventing indefinite fund locks. Sellers can also initiate refunds if they cannot fulfill orders.


## User Stories

### Creating an Escrow

**As a buyer**, I want to create an escrow transaction so that my funds are protected until I receive the goods/services.

**Acceptance Criteria:**
- I can specify the seller's address
- I can set the payment amount and currency
- I can set a deadline for the transaction
- I can add a description of what I'm purchasing
- The system locks my funds upon creation
- I receive confirmation with a unique escrow ID

### Releasing Funds

**As a buyer**, I want to release funds when satisfied so that the seller receives payment.

**Acceptance Criteria:**
- I can view my active escrows
- I can confirm receipt of goods/services
- Funds transfer to seller immediately
- Platform fee is deducted automatically
- Both parties receive notification

### Requesting Refund

**As a seller**, I want to initiate a refund if I cannot fulfill the order so that the buyer gets their money back.

**Acceptance Criteria:**
- I can view escrows where I'm the seller
- I can initiate refund with reason
- Funds return to buyer immediately
- No platform fee charged on refunds
- Both parties receive notification

### Claiming Expired Escrow

**As a buyer**, I want to reclaim my funds if the deadline passes so that my money isn't locked forever.

**Acceptance Criteria:**
- Can query expired escrows
- I can claim refund after deadline
- Transaction marked as expired
- Seller notified of expiration

## Functional Requirements

### Escrow Creation
- Minimum and maximum amount limits
- Maximum description length (500 characters)
- Deadline must be future date (1 hour minimum, 90 days maximum)
- Support for native and custom tokens

### Fund Management
- Funds locked in pallet account, not accessible to anyone
- Automatic fee calculation and display
- Clear breakdown of amounts (principal, fees, total)
- Support for partial releases in future versions

### Notifications
- On-chain events for all state changes

### Security Requirements
- Only authorized parties can perform actions
- No single party can unilaterally access funds
- Rate limiting to prevent spam
- Minimum escrow amount to discourage abuse

## Platform Configuration

### Fee Structure
- Platform fee: 0.5% - 2% (configurable by governance)
- No fees on refunds or expired claims
- Reduced fees for high-volume users

### Limits and Constraints
- Minimum escrow: 10 units
- Maximum escrow: 1,000,000 units
- Description: 500 characters
- Dispute reason: 1000 characters
- Maximum active escrows per user: 100

