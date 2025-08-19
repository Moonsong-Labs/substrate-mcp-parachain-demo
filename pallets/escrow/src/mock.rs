use crate as pallet_escrow;
use frame::{
	derive_impl, parameter_types,
	traits::{ConstU128, Hooks},
	deps::sp_runtime::BuildStorage,
};

type Block = frame::deps::frame_system::mocking::MockBlock<Test>;

frame::deps::frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		Balances: pallet_balances,
		EscrowPallet: pallet_escrow,
	}
);

#[derive_impl(frame::deps::frame_system::config_preludes::TestDefaultConfig)]
impl frame::deps::frame_system::Config for Test {
	type Block = Block;
	type AccountData = pallet_balances::AccountData<u128>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type Balance = u128;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
}

parameter_types! {
	pub const EscrowPalletId: frame::PalletId = frame::PalletId(*b"py/escro");
	pub const MinEscrow: u128 = 10;
	pub const MaxEscrow: u128 = 1_000_000;
	pub const MaxDescriptionLen: u32 = 500;
	pub const MinDeadline: u64 = 10;
	pub const MaxDeadline: u64 = 10_000;
	pub const MaxActiveEscrows: u32 = 100;
}

impl pallet_escrow::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type PalletId = EscrowPalletId;
	type MinEscrow = MinEscrow;
	type MaxEscrow = MaxEscrow;
	type MaxDescriptionLen = MaxDescriptionLen;
	type MinDeadline = MinDeadline;
	type MaxDeadline = MaxDeadline;
	type MaxActiveEscrows = MaxActiveEscrows;
	type GovernanceOrigin = frame::deps::frame_system::EnsureRoot<u64>;
	type WeightInfo = ();
}

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame::deps::frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 100_000),
			(BOB, 100_000),
			(CHARLIE, 100_000),
		],
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	let mut ext = frame::deps::sp_io::TestExternalities::new(storage);
	ext.execute_with(|| System::set_block_number(1));
	ext
}