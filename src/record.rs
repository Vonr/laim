#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Record {
    pub position: u32,
    pub score: u32,
    pub millis: u128,
    pub rows: u32,
    pub columns: u32,
    pub active: u32,
}

impl Record {
    #[inline]
    pub const fn new(
        position: u32,
        score: u32,
        millis: u128,
        rows: u32,
        columns: u32,
        active: u32,
    ) -> Self {
        Self {
            position,
            score,
            millis,
            rows,
            columns,
            active,
        }
    }

    pub fn from_str(str: &str) -> Option<Self> {
        let mut iter = str.split(',');
        let version = iter.next()?.parse().ok()?;

        let ret = match version {
            1 => Self {
                position: iter.next()?.parse().ok()?,
                score: iter.next()?.parse().ok()?,
                millis: iter.next()?.parse().ok()?,
                rows: iter.next()?.parse().ok()?,
                columns: iter.next()?.parse().ok()?,
                active: iter.next()?.parse().ok()?,
            },
            _ => return None,
        };

        if iter.next().is_some() {
            return None;
        }

        Some(ret)
    }

    pub fn to_string(&self) -> String {
        format!(
            "1,{},{},{},{},{},{}",
            self.position, self.score, self.millis, self.rows, self.columns, self.active
        )
    }
}
