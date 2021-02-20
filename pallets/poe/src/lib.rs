#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure, StorageMap
};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as PoeModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Proofs: map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Event emitted when a proof has been claimed. [who, claim]
        ClaimCreated(AccountId, Vec<u8>),
        /// Event emitted when a claim is revoked by the owner. [who, claim]
        ClaimRevoked(AccountId, Vec<u8>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// 该证明已经被声明
        ProofAlreadyClaimed,
        /// 该证明不存在，因此它不能被撤销
        NoSuchProof,
        /// 该证明已经被另一个账号声明，因此它不能被撤销
        NotProofOwner,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		 /// 允许用户队未声明的证明拥有所有权
        #[weight = 10_000]
        fn create_claim(origin, proof: Vec<u8>) {
            // 检查 extrinsic 是否签名并获得签名者
            // 如果 extrinsic 未签名，此函数将返回一个错误。
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // 校验指定的证明是否被声明
            ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);

            // 从 FRAME 系统模块中获取区块号.
            let current_block = <frame_system::Module<T>>::block_number();

            // 存储证明：发送人与区块号
            Proofs::<T>::insert(&proof, (&sender, current_block));

            // 声明创建后，发送事件
            Self::deposit_event(RawEvent::ClaimCreated(sender, proof));
        }

        /// 允许证明所有者撤回声明
        #[weight = 10_000]
        fn revoke_claim(origin, proof: Vec<u8>) {
            //  检查 extrinsic 是否签名并获得签名者
            // 如果 extrinsic 未签名，此函数将返回一个错误。
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // 校验指定的证明是否被声明
            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            // 获取声明的所有者
            let (owner, _) = Proofs::<T>::get(&proof);

            // 验证当前的调用者是证声明的所有者
            ensure!(sender == owner, Error::<T>::NotProofOwner);

            // 从存储中移除声明
            Proofs::<T>::remove(&proof);

            // 声明抹掉后，发送事件
            Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));
        }
	}
}
