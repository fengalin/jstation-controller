/// Generates a list of named values for a [`DiscreteParameter`].
macro_rules! generate_parameter_list {
    ($param:ty, $named_param:ident, $name:ident, $names:ident, $name_list:expr $(,)?) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $named_param {
            idx: usize,
            param: $param,
            name: &'static str,
        }

        impl $named_param {
            pub fn param(self) -> $param {
                self.param
            }
        }

        impl std::fmt::Display for $named_param {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(self.name, f)
            }
        }

        impl PartialEq for $named_param {
            fn eq(&self, other: &Self) -> bool {
                self.idx == other.idx
            }
        }

        impl Eq for $named_param {}

        impl $param {
            pub fn $names() -> &'static [$named_param] {
                static LIST: once_cell::sync::Lazy<Vec<$named_param>> =
                    once_cell::sync::Lazy::new(|| {
                        use crate::jstation::data::DiscreteParameter;

                        assert_eq!(
                            $name_list.len(),
                            (<$param>::MAX_RAW.as_u8() - <$param>::MIN_RAW.as_u8()) as usize + 1,
                            concat!(
                                stringify!($names),
                                " list len and ",
                                stringify!($param),
                                " range mismatch",
                            ),
                        );

                        Vec::<$named_param>::from_iter($name_list.into_iter().enumerate().map(
                            |(idx, name)| {
                                let param = <$param>::try_from_raw(RawValue::new(idx as u8))
                                    .expect("Param names and range mismatch");

                                $named_param { idx, param, name }
                            },
                        ))
                    });

                &*LIST
            }

            pub fn $name(self) -> $named_param {
                use crate::jstation::data::DiscreteParameter;
                Self::$names()[self.to_raw_value().as_u8() as usize]
            }
        }
    };
}
