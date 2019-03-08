use super::page;
use maud::{html, Markup};

pub struct DashBoardPresenter<'a> {
    pub email: &'a str,
    pub entry_count: usize,
    pub event_count: usize,
    pub tag_count: usize,
    pub user_count: usize,
}

pub fn dashboard(email: Option<&str>, data: DashBoardPresenter) -> Markup {
    page(
        "Admin Dashboard",
        email,
        None,
        None,
        html! {
            main {
                h3 { "Database Statistics" }
                table {
                    tr {
                        td {"Number of Entries"}
                        td {(data.entry_count)}
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
            }
        },
    )
}
