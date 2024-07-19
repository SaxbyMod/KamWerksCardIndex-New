#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $value:expr,)+) => {
        {
            let mut m = std::collections::HashMap::new();

            $(m.insert($key, $value);)*

            m
        }
    };
}
#[macro_export]
macro_rules! set_map {
    ($($name:ident ($code:ident) => $link:literal),+) => {
        hashmap! {
            $(
                stringify!($code).to_owned() => {
                    let t = fetch_imf_set(
                        $link,
                        SetCode::new(stringify!($code)).unwrap()
                    )
                    .unwrap_or_die(&format!("Cannot process {} set", stringify!($name)))
                    .upgrade();

                    println!("Finish fetching {} set with code {}", stringify!($name), stringify!($code));

                    t
                },
            )*
        }
    };
}
