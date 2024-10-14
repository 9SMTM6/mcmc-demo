#[macro_export]
macro_rules! cfg_if_expr {
    (
        =>[$condition: meta]
        $true_block: expr
        =>[not]
        $false_block: expr
    ) => {{
        #[cfg($condition)]
        let _return = $true_block;
        #[cfg(not($condition))]
        let _return = $false_block;
        _return
    }};
}

#[allow(unused)]
fn test() {
    let var = cfg_if_expr!(
    => [target_arch = "wasm32"]
    {
        let inner = 4;
        4
    }
    => [not]
    {5});
}
