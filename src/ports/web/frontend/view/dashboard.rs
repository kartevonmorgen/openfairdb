use super::page;
use crate::core::entities::*;
use maud::{html, Markup};

pub struct DashBoardPresenter {
    pub user: User,
    pub place_count: usize,
    pub event_count: usize,
    pub tag_count: usize,
    pub user_count: usize,
}

pub fn dashboard(data: DashBoardPresenter) -> Markup {
    page(
        "Admin Dashboard",
        Some(&data.user.email),
        None,
        None,
        html! {
            main class="dashboard" {
                h3 { "Database Statistics" }
                table {
                    tr {
                        td {"Number of Places"}
                        td {(data.place_count)}
                    }
                    tr {
                        td {"Number of Events"}
                        td {(data.event_count)}
                    }
                    tr {
                        td {"Number of Users"}
                        td {(data.user_count)}
                    }
                    tr {
                        td {"Number of Tags"}
                        td {(data.tag_count)}
                    }
                }
                h3 { "User Management" }
                (super::search_users_form())
            }
        },
    )
}
