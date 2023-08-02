use core::convert::identity as id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Exp(pub(crate) u64);

impl Exp {
    pub(crate) fn to_i64(self) -> i64 {
        let Exp(exp) = self;
        #[allow(clippy::cast_possible_wrap)]
        let exp = id::<u64>(exp) as i64;
        exp
    }

    pub(crate) fn from_i64(exp: i64) -> Self {
        debug_assert!(exp >= 0);
        #[allow(clippy::cast_sign_loss)]
        let exp: u64 = id::<i64>(exp) as u64;
        Exp(exp)
    }
}
