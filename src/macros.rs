macro_rules! i64_from_as_ref_user_id {
    ($discord_id:expr) => {{
        let UserId(ref discord_id) = $discord_id.as_ref();
        let discord_id: u64 = discord_id.clone();
        let discord_id: i64 = ::core::convert::identity::<u64>(discord_id) as i64;
        discord_id
    }};
}

macro_rules! u63_from_as_ref_user_id {
    ($discord_id:expr) => {{
        let UserId(ref discord_id) = $discord_id.as_ref();
        let discord_id: u64 = discord_id.clone();
        let discord_id: u63 = ::ux::u63::new(discord_id);
        discord_id
    }};
}

// Exporting the macro
// https://stackoverflow.com/questions/26731243/how-do-i-use-a-macro-across-module-files/67140319#67140319
pub(crate) use i64_from_as_ref_user_id;
pub(crate) use u63_from_as_ref_user_id;
