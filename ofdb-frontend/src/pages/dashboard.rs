use leptos::*;

use crate::api::PublicApi;

#[component]
pub fn Dashboard<A>(cx: Scope, api: A) -> impl IntoView
where
    A: PublicApi + Clone + 'static,
{
    let api_clone = api.clone();
    let fetch_entries_count_action = create_action(cx, move |_| {
        let api = api_clone.clone();
        async move { api.count_entries().await }
    });

    let fetch_tags_count_action = create_action(cx, move |_| {
        let api = api.clone();
        async move { api.count_tags().await }
    });

    fetch_entries_count_action.dispatch(());
    fetch_tags_count_action.dispatch(());

    view! { cx,
      <section>
        <div class="container p-6 mx-auto">
          <h3 class="text-xl font-semibold mt-1 mb-3">"Statistics"</h3>
          <div class="columns-3">
            <table class="border-collapse table-auto w-full">
              <tbody>
                <tr>
                  <td class="border-b border-slate-100">
                    "Number of Entries"
                  </td>
                  <td class="border-b border-slate-100">
                    <DisplayNumber
                      number = fetch_entries_count_action.value().into()
                  />
                  </td>
                </tr>
                <tr>
                  <td class="border-b border-slate-100">"Number of Tags"</td>
                  <td class="border-b border-slate-100">
                    <DisplayNumber
                      number = fetch_tags_count_action.value().into()
                    />
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>
    }
}

#[component]
fn DisplayNumber(
    cx: Scope,
    number: Signal<Option<Result<usize, crate::api::Error>>>,
) -> impl IntoView {
    let memorized_number = create_memo(cx, move |_| number.get());

    move || match memorized_number.get() {
        Some(Ok(nr)) => view! {cx, { nr.to_string() } }.into_view(cx),
        Some(Err(_)) => {
            // TODO: use an appropriate icon
            view! {cx, "- API error -" }.into_view(cx)
        }
        None => {
            // TODO: use spinner icon
            view! {cx, "-" }.into_view(cx)
        }
    }
}
