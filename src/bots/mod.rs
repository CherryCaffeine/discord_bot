mod bot;
mod main_bot;
#[cfg(test)]
mod test_bot;

pub(crate) use bot::Bot;
pub(crate) use main_bot::MainBot;
#[cfg(test)]
pub(crate) use test_bot::TestBot;
