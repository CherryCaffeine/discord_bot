mod cfg_ext;
mod bot;
#[cfg(test)]
mod test_bot;

pub(crate) use cfg_ext::CfgExt;
pub(crate) use bot::Bot;
#[cfg(test)]
pub(crate) use test_bot::TestBot;