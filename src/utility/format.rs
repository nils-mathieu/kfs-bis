use core::fmt::{Display, Formatter, Result};

/// Displays a size in a way that's readable.
#[derive(Debug, Clone, Copy)]
pub struct HumanBytes(pub u64);

impl Display for HumanBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        fn write_with_point(x: u64, f: &mut Formatter, end: &str) -> Result {
            let frac = ((x % 1024) * 100) / 1024;
            let int = x / 1024;

            if frac != 0 {
                write!(f, "{int}.{frac} {end}")
            } else {
                write!(f, "{int} {end}")
            }
        }

        let mut val = self.0;

        if val < 1024 {
            return write!(f, "{} B", val);
        }

        for ext in ["KiB", "MiB", "GiB", "TiB"] {
            if val < 1024 * 1024 {
                return write_with_point(val, f, ext);
            }

            val /= 1024;
        }

        // Wtf this is so large??
        write_with_point(val, f, "PiB")
    }
}
