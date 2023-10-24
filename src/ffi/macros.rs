// Copyright © ByteHeed.  All rights reserved.

#[macro_export]
macro_rules! STRUCT {
    ($(#[$attrs:meta])* struct $name:ident {
        $($field:ident: $ftype:ty,)+
    }) => (
        #[repr(C)] $(#[$attrs])*
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $field: $ftype,)+
        }
    );
}
