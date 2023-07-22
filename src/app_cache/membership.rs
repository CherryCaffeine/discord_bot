use core::convert::identity as id;
use serenity::model::prelude::Member;
use sqlx::PgPool;
use std::cmp::Ordering;
use drain_at_sorted_unchecked::drain_at_sorted_unchecked;

use crate::db::{self, dao};

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct Diff {
    db_info: Vec<dao::ServerMember>,
    fetched_info: Vec<Member>,
    pairings: Vec<[Option<usize>; 2]>,
}

impl Diff {
    // TODO: clean up this mess
    pub(super) fn new(mut db_info: Vec<dao::ServerMember>, fetched_info: Vec<Member>) -> Self {
        let mut pairings =
            Vec::<[Option<usize>; 2]>::with_capacity(db_info.len() + fetched_info.len());

        db_info.sort_unstable_by(|a, b| i64::cmp(&a.discord_id, &b.discord_id));

        // fetched_info is assumed to be already sorted

        let mut db_server_member_id_iter = db_info
            .iter()
            .map(|sm| sm.discord_id)
            .enumerate()
            .peekable();
        #[allow(clippy::cast_possible_wrap)]
        let mut fetched_server_member_id_iter = fetched_info
            .iter()
            .map(|sm| id::<u64>(sm.user.id.0) as i64)
            .enumerate()
            .peekable();

        let (mut db_info_idx, mut db_id) = if let Some(db_entry) = db_server_member_id_iter.peek() {
            *db_entry
        } else {
            pairings.extend(fetched_server_member_id_iter.map(|(i, _id)| [None, Some(i)]));
            return Diff {
                db_info,
                fetched_info,
                pairings,
            };
        };

        let (mut fetched_info_idx, mut fetched_id) = match fetched_server_member_id_iter.peek() {
            Some(fetched_entry) => *fetched_entry,
            None => {
                unreachable!("There's always at least a bot on the server");
            }
        };

        loop {
            match i64::cmp(&db_id, &fetched_id) {
                Ordering::Equal => {
                    pairings.push([Some(db_info_idx), Some(fetched_info_idx)]);
                    db_server_member_id_iter.next();
                    fetched_server_member_id_iter.next();
                    match [
                        db_server_member_id_iter.peek(),
                        fetched_server_member_id_iter.peek(),
                    ] {
                        [Some(next_db_entry), Some(next_fetched_entry)] => {
                            (db_info_idx, db_id) = *next_db_entry;
                            (fetched_info_idx, fetched_id) = *next_fetched_entry;
                        }
                        [Some(_), None] => {
                            pairings
                                .extend(db_server_member_id_iter.map(|(i, _id)| [Some(i), None]));
                            break;
                        }
                        [None, Some(_)] => {
                            pairings.extend(
                                fetched_server_member_id_iter.map(|(i, _id)| [None, Some(i)]),
                            );
                            break;
                        }
                        [None, None] => {
                            break;
                        }
                    };
                }
                Ordering::Less => {
                    pairings.push([Some(db_info_idx), None]);
                    db_server_member_id_iter.next();
                    if let Some(next_db_entry) = db_server_member_id_iter.peek() {
                        (db_info_idx, db_id) = *next_db_entry;
                    } else {
                        pairings
                            .extend(fetched_server_member_id_iter.map(|(i, _id)| [None, Some(i)]));
                        break;
                    }
                }
                Ordering::Greater => {
                    pairings.push([None, Some(fetched_info_idx)]);
                    fetched_server_member_id_iter.next();
                    if let Some(next_fetched_entry) = fetched_server_member_id_iter.peek() {
                        (fetched_info_idx, fetched_id) = *next_fetched_entry;
                    } else {
                        pairings.extend(db_server_member_id_iter.map(|(i, _id)| [Some(i), None]));
                        break;
                    }
                }
            }
        }
        Diff {
            db_info,
            fetched_info,
            pairings,
        }
    }

    pub(super) async fn sync_and_distill(mut self, pool: &PgPool) -> Vec<dao::ServerMember> {
        // quitters' data is not stored in Vec<(usize, i64)> because
        // sqlx favors slices over iterators.
        let mut quitters = Vec::<i64>::new();
        let mut quitters_indices = Vec::<usize>::new();
        let mut newcomers = Vec::<i64>::new();

        for [db_idx_opt, fetched_idx_opt] in &self.pairings {
            match [db_idx_opt, fetched_idx_opt] {
                [Some(db_idx), _fetched_idx @ None] => {
                    let quitter: &dao::ServerMember = &self.db_info[*db_idx];
                    quitters_indices.push(*db_idx);
                    quitters.push(quitter.discord_id);
                }
                [_db_idx @ None, Some(fetched_idx)] => {
                    let newcomer: &Member = &self.fetched_info[*fetched_idx];
                    #[allow(clippy::cast_possible_wrap)]
                    newcomers.push(id::<u64>(newcomer.user.id.0) as i64);
                }
                _ => {}
            }
        }
        db::mark_as_quitters(pool, &quitters)
            .await
            .expect("Failed to mark quitters as such in the database");
        db::add_newcomers(pool, &newcomers)
            .await
            .expect("Failed to add newcomers to the database");
        
        // Safety:
        //
        // * `quitters_indices` are in ascending order by construction
        // * the indices in `quitters_indices` are valid indices in `self.db_info`
        // * the indices in `quitters_indices` are unique
        // * the data in `self.db_info` is trivially movable
        unsafe { drain_at_sorted_unchecked(&mut self.db_info, quitters_indices) };
        self.db_info
    }
}
