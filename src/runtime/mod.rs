macro_rules! import_fns {
    ($x:ident) => {
        mod $x;
        pub use self::$x::*;
    };
}

import_fns!(db);
import_fns!(dns);
import_fns!(error);
import_fns!(http);
import_fns!(html);
import_fns!(json);
import_fns!(logger);
import_fns!(print);
import_fns!(sleep);
import_fns!(url);
