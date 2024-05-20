cfg_if::cfg_if! {
    if #[cfg(all(test, feature = "std"))] {
        mod erc20;
        mod erc721;
        pub mod infrastructure;
    }
}
