macro_rules! cfg_async {
    ($($item:item)*) => {
        $(
            #[cfg(all(feature = "tokio", feature = "futures"))]
            #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
            $item
        )*
    }
}
