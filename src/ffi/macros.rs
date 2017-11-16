// Copyright © ByteHeed.  All rights reserved.

#[macro_export]
macro_rules! STRUCT {
    ($(#[$attrs:meta])* struct $name:ident {
        $($field:ident: $ftype:ty,)+
    }) => (
        #[repr(C)] $(#[$attrs])*
        pub struct $name {
            $(pub $field: $ftype,)+
        }
        impl Copy for $name {}
        impl Clone for $name {
            #[inline]
            fn clone(&self) -> $name { *self }
        }
    );
}
