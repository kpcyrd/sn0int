use std::fmt;
pub use std::fmt::Write;
pub use std::fmt::{Result, Formatter, Display, Debug};


pub mod colors {
    use std::fmt::{self, Write};

    pub trait Color {
        fn color<W: Write>(w: &mut W) -> fmt::Result;
        fn display<W: Write, D: fmt::Display>(w: &mut W, v: D) -> fmt::Result;
        fn debug<W: Write, D: fmt::Debug>(w: &mut W, v: D) -> fmt::Result;
    }

    pub struct Red;

    impl Color for Red {
        #[inline]
        fn color<W: Write>(w: &mut W) -> fmt::Result {
            write!(w, "\x1b[31m")
        }

        #[inline]
        fn display<W: Write, D: fmt::Display>(w: &mut W, v: D) -> fmt::Result {
            write!(w, "\x1b[31m{}\x1b[0m", v)
        }

        #[inline]
        fn debug<W: Write, D: fmt::Debug>(w: &mut W, v: D) -> fmt::Result {
            write!(w, "\x1b[31m{:?}\x1b[0m", v)
        }
    }

    pub struct Green;

    impl Color for Green {
        #[inline]
        fn color<W: Write>(w: &mut W) -> fmt::Result {
            write!(w, "\x1b[32m")
        }

        #[inline]
        fn display<W: Write, D: fmt::Display>(w: &mut W, v: D) -> fmt::Result {
            write!(w, "\x1b[32m{}\x1b[0m", v)
        }

        #[inline]
        fn debug<W: Write, D: fmt::Debug>(w: &mut W, v: D) -> fmt::Result {
            write!(w, "\x1b[32m{:?}\x1b[0m", v)
        }
    }

    pub struct Yellow;

    impl Color for Yellow {
        #[inline]
        fn color<W: Write>(w: &mut W) -> fmt::Result {
            write!(w, "\x1b[33m")
        }

        #[inline]
        fn display<W: Write, D: fmt::Display>(w: &mut W, v: D) -> fmt::Result {
            write!(w, "\x1b[33m{}\x1b[0m", v)
        }

        #[inline]
        fn debug<W: Write, D: fmt::Debug>(w: &mut W, v: D) -> fmt::Result {
            write!(w, "\x1b[33m{:?}\x1b[0m", v)
        }
    }
}
use self::colors::*;


pub struct DetailFormatter<'a, 'b> {
    w: &'a mut fmt::Formatter<'b>,
    scoped: bool,
    in_group: bool,
    fresh_group: bool,
}

impl<'a, 'b> DetailFormatter<'a, 'b> {
    pub fn new(w: &'a mut fmt::Formatter<'b>, scoped: bool) -> DetailFormatter<'a, 'b> {
        DetailFormatter {
            w,
            scoped,
            in_group: false,
            fresh_group: true,
        }
    }

    #[inline]
    pub fn start_group(&mut self) {
        self.in_group = true;
        self.fresh_group = true;
    }

    #[inline]
    pub fn end_group(&mut self) -> fmt::Result {
        self.in_group = false;
        if !self.fresh_group {
            write!(self, "]")?
        }
        Ok(())
    }

    #[inline]
    pub fn start(&mut self) -> fmt::Result {
        if !self.scoped {
            write!(self, "\x1b[90m")
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn end(&mut self) -> fmt::Result {
        if !self.scoped {
            write!(self, "\x1b[0m")
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn id<D: fmt::Display>(&mut self, v: D) -> fmt::Result {
        if self.scoped {
            write!(self, "\x1b[32m#{}\x1b[0m, ", v)
        } else {
            write!(self, "#{}, ", v)
        }
    }

    #[inline]
    pub fn color<C: Color>(&mut self) -> fmt::Result {
        if self.scoped {
            C::color(self)
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn direct_display<C: Color, D: fmt::Display>(&mut self, v: D) -> fmt::Result {
        if self.scoped {
            C::display(self, v)
        } else {
            write!(self, "{}", v)
        }
    }

    #[inline]
    pub fn display<C: Color, D: fmt::Display>(&mut self, v: D) -> fmt::Result {
        self.push_into_group()?;
        self.direct_display::<C, _>(v)
    }

    pub fn display_label<C: Color, D: fmt::Display>(&mut self, label: &str, v: D) -> fmt::Result {
        self.push_into_group()?;
        write!(self, "{}=", label)?;
        self.direct_display::<C, _>(v)
    }

    #[inline]
    pub fn direct_debug<C: Color, D: fmt::Debug>(&mut self, v: D) -> fmt::Result {
        if self.scoped {
            C::debug(self, v)
        } else {
            write!(self, "{:?}", v)
        }
    }

    #[inline]
    pub fn debug<C: Color, D: fmt::Debug>(&mut self, v: D) -> fmt::Result {
        self.push_into_group()?;
        self.direct_debug::<C, _>(v)
    }

    pub fn opt_debug<C: Color, D: fmt::Debug>(&mut self, v: &Option<D>) -> fmt::Result {
        if let Some(v) = &v {
            self.debug::<C, _>(v)?;
        }
        Ok(())
    }

    pub fn opt_debug_label<C: Color, D: fmt::Debug>(&mut self, label: &str, v: &Option<D>) -> fmt::Result {
        if let Some(v) = &v {
            self.push_into_group()?;
            write!(self, "{}=", label)?;
            self.direct_debug::<C, _>(v)?;
        }
        Ok(())
    }

    #[inline]
    pub fn clear(&mut self) -> fmt::Result {
        if self.scoped {
            write!(self, "\x1b[0m")
        } else {
            Ok(())
        }
    }

    fn push_into_group(&mut self) -> fmt::Result {
        if self.in_group {
            if !self.fresh_group {
                write!(self, " / ")?;
            } else {
                write!(self, " [")?;
                self.fresh_group = false;
            }
        }
        Ok(())
    }

    #[inline]
    pub fn child<D: fmt::Display>(&mut self, c: D) -> fmt::Result {
        if self.scoped {
            // if child is unscoped, draw as grey as well
            write!(self, "\n\t\x1b[33m{}\x1b[0m", c)
        } else {
            write!(self, "\n\t\x1b[90m{}\x1b[0m", c)
        }
    }
}

impl<'a, 'b> fmt::Write for DetailFormatter<'a, 'b> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.w.write_str(s)
    }
}
