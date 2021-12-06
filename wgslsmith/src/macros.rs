#[macro_export]
macro_rules! define_type {
    (scalar: $type:ident) => {
        define_type!($type => DataType::Scalar(ScalarType::$type));
    };

    (vec: $(($n:literal, $t:ident)),* $(,)?) => {
        paste::paste! {
            $(define_type!([<Vec $n $t>] => (DataType::Vector($n, ScalarType::$t)));)*
        }
    };

    ($name:ident => ($($type:ident),* $(,)?)) => {
        paste::paste!{
            static [<$name:snake:upper>]: OnceCell<TypeConstraints> = OnceCell::new();
            impl TypeConstraints {
                #[allow(non_snake_case)]
                pub fn [<$name>]() -> &'static TypeConstraints {
                    [<$name:snake:upper>].get_or_init(||{
                        let mut set = TypeConstraints::empty();
                        $(set.insert_all(TypeConstraints::$type());)*
                        set
                    })
                }
            }
        }
    };

    ($name:ident => $type:expr) => {
        paste::paste!{
            static [<$name:snake:upper>]: OnceCell<TypeConstraints> = OnceCell::new();
            impl TypeConstraints {
                #[allow(non_snake_case)]
                pub fn [<$name>]() -> &'static TypeConstraints {
                    [<$name:snake:upper>].get_or_init(|| {
                        TypeConstraints({
                            let mut set = HashTrieSetSync::new_sync();
                            set.insert_mut($type);
                            set
                        })
                    })
                }
            }
        }
    };
}
