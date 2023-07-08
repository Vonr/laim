use std::collections::HashSet;

use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use rand::Rng;
use web_sys::{Attr, Event};
use web_time::Instant;

type SignalPair<T> = (ReadSignal<T>, WriteSignal<T>);
type Position = (usize, usize);
type Positions = HashSet<Position, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

mod record {
    use serde::*;
    #[derive(Clone, Copy, Serialize, Deserialize)]
    pub struct Record(u64, u64, u128, usize, usize);

    #[allow(dead_code)]
    impl Record {
        #[inline]
        pub const fn new(
            position: u64,
            score: u64,
            millis: u128,
            rows: usize,
            columns: usize,
        ) -> Self {
            Self(position, score, millis, rows, columns)
        }

        #[inline]
        pub const fn position(&self) -> u64 {
            self.0
        }

        #[inline]
        pub fn set_position(&mut self, value: u64) {
            self.0 = value;
        }

        #[inline]
        pub const fn score(&self) -> u64 {
            self.1
        }

        #[inline]
        pub fn set_score(&mut self, value: u64) {
            self.1 = value;
        }

        #[inline]
        pub const fn millis(&self) -> u128 {
            self.2
        }

        #[inline]
        pub fn set_millis(&mut self, value: u128) {
            self.2 = value;
        }

        #[inline]
        pub const fn rows(&self) -> usize {
            self.3
        }

        #[inline]
        pub fn set_rows(&mut self, value: usize) {
            self.3 = value;
        }

        #[inline]
        pub const fn columns(&self) -> usize {
            self.4
        }

        #[inline]
        pub fn set_columns(&mut self, value: usize) {
            self.4 = value;
        }
    }
}

type Record = record::Record;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let columns = create_signal(cx, LocalStorage::get("columns").unwrap_or(3));
    let rows = create_signal(cx, LocalStorage::get("rows").unwrap_or(3));
    let active = create_signal(cx, LocalStorage::get("active").unwrap_or(3));

    let current: SignalPair<Positions> = create_signal(
        cx,
        HashSet::with_capacity_and_hasher(active.0() + 1, Default::default()),
    );
    let history = create_signal(
        cx,
        LocalStorage::get("history").unwrap_or_else(|_| Vec::new()),
    );

    let current_record = create_signal(cx, Record::new(0, 0, 0, rows.0(), columns.0()));
    let score = move || current_record.0().score();

    let history_best = move || {
        let mut history_obj = history.0();
        history_obj.retain(|e: &Record| e.rows() == rows.0() && e.columns() == columns.0());
        history_obj.sort_by(|a: &Record, b: &Record| {
            use std::cmp::Ordering::*;
            match a.score().cmp(&b.score()) {
                Equal => b.millis().cmp(&a.millis()),
                otherwise => otherwise,
            }
        });
        history_obj
            .last()
            .copied()
            .unwrap_or_else(|| Record::new(0, 0, 0, rows.0(), columns.0()))
    };

    let best_record = create_signal(cx, history_best());

    let update_current = move || {
        let rows = rows.0();
        let columns = columns.0();
        let active = active.0().min(columns * rows - 1);
        let mut rng = rand::thread_rng();

        current.1.update(|current| {
            current.clear();
            while current.len() < active {
                let new = (rng.gen_range(0..rows), rng.gen_range(0..columns));
                if current.contains(&new) {
                    continue;
                }

                current.insert(new);
            }
        });
    };

    let max_active = create_memo(cx, move |_| rows.0() * columns.0() - 1);
    let score_text = create_memo(cx, move |_| {
        format!(
            "Score: {} ({:.2}/s) / {} ({:.2}/s)",
            score(),
            (score() * 1000) as f64 / current_record.0().millis() as f64,
            history_best().score(),
            (history_best().score() * 1000) as f64 / history_best().millis() as f64
        )
    });

    view! { cx,
        <div style="display: flex; justify-content: space-evenly;">
            <UsizeInput name="rows" label="Rows: " min=2 max=usize::MAX signal=rows current=current.1 onchange=update_current />
            <UsizeInput name="columns" label="Columns: " min=2 max=usize::MAX signal=columns current=current.1 onchange=update_current />
            <UsizeInput name="active" label="Active: " min=1 max=max_active signal=active current=current.1 onchange=update_current />
            <button on:click=move |_| {
                best_record.1.update(|record| {
                    record.set_millis(0);
                });
                history.1.update(|history| {
                    history.clear();
                    LocalStorage::delete("history");
                });
            }>"Clear History"</button>
        </div>

        <Game current={current} history={history} columns={columns.0} rows={rows.0} active={active.0} current_record={current_record} best_record={best_record} />

        <h3 style="text-align: center;">{score_text}</h3>
        <GameHistory history={history.0} />
    }
}

#[component]
fn UsizeInput<F>(
    cx: Scope,
    name: &'static str,
    label: &'static str,
    min: usize,
    #[prop(into)] max: MaybeSignal<usize>,
    signal: SignalPair<usize>,
    current: WriteSignal<Positions>,
    onchange: F,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! { cx,
        <span>
            <label for=name>{label}</label>
            <input
                name=name
                type="number"
                min=min
                max=max
                value=signal.0
                on:change=move |ev| {
                    signal.1(event_target_value(&ev).parse().unwrap_or_else(|_| signal.0()));
                    current.update(|current| current.clear());
                    let _ = LocalStorage::set(name, signal.0());
                    onchange();
                }
            />
        </span>
    }
}

#[component]
fn GameHistory(cx: Scope, history: ReadSignal<Vec<Record>>) -> impl IntoView {
    view! { cx,
        <table class="GameHistory">
            <tr class="GameHistory">
                <th class="GameHistory">"Position"</th>
                <th class="GameHistory">"Score"</th>
                <th class="GameHistory">"Score/s"</th>
                <th class="GameHistory">"Seconds"</th>
                <th class="GameHistory">"Size"</th>
            </tr>

            <For
                each=history
                key=|record| record.position()
                view=move |cx, record| {
                    view! { cx,
                        <tr class="GameHistory">
                            <td class="GameHistory">{record.position()}</td>
                            <td class="GameHistory">{record.score()}</td>
                            <td class="GameHistory">{format!("{:.2}", (record.score() * 1000) as f64 / record.millis() as f64)}</td>
                            <td class="GameHistory">{format!("{:.2}", record.millis() as f64 / 1000f64)}</td>
                            <td class="GameHistory">{format!("{}×{}", record.rows(), record.columns())}</td>
                        </tr>
                    }
                }
            />
        </table>
    }
}

#[component]
fn Game(
    cx: Scope,
    current: SignalPair<Positions>,
    history: SignalPair<Vec<Record>>,
    columns: ReadSignal<usize>,
    rows: ReadSignal<usize>,
    active: ReadSignal<usize>,
    current_record: SignalPair<Record>,
    best_record: SignalPair<Record>,
) -> impl IntoView {
    let (current, set_current) = current;
    let (history, set_history) = history;
    let (current_record, set_current_record) = current_record;
    let (best_record, set_best_record) = best_record;

    let (start, set_start) = create_signal(cx, Instant::now());
    let (hovered, set_hovered) = create_signal(cx, None);

    let active = move || active().min(rows() * columns() - 1);

    let mut rng = rand::thread_rng();
    set_current.update(|current| {
        current.clear();
        while current.len() < active() {
            let new = (rng.gen_range(0..rows()), rng.gen_range(0..columns()));
            if current.contains(&new) {
                continue;
            }

            current.insert(new);
        }
    });

    let game_over = move || {
        let curr = current_record();
        if curr.score() > 1 {
            set_history.update(|history| {
                history.insert(
                    0,
                    Record::new(
                        history.len() as u64 + 1,
                        curr.score(),
                        curr.millis(),
                        rows(),
                        columns(),
                    ),
                )
            });

            let _ = LocalStorage::set("history", history());
        }
        set_current_record.update(|record| record.set_score(0));
    };

    let on_input = move |row, col| {
        if current().contains(&(row, col)) {
            let now = Instant::now();
            let mut rng = rand::thread_rng();
            set_current.update(|current| {
                let current_record = current_record();
                let best_record = best_record();
                let score = current_record.score();
                let best_score = best_record.score();
                let millis = current_record.millis();
                let best_millis = best_record.millis();
                let rows = rows();
                let columns = columns();

                if score == 0 {
                    set_start(now);
                }

                set_current_record.update(|record| {
                    record.set_millis((now - start()).as_millis());
                    record.set_score(record.score() + 1)
                });

                if score > best_score || (score == best_score && millis < best_millis) {
                    set_best_record.update(|record| {
                        record.set_score(score);
                        record.set_millis(millis);
                    });
                }

                let mut new = (rng.gen_range(0..rows), rng.gen_range(0..columns));
                while current.contains(&new) {
                    new = (rng.gen_range(0..rows), rng.gen_range(0..columns));
                }
                current.remove(&(row, col));
                current.insert(new);
            });
            return;
        }

        game_over();
    };

    let on_trigger = move |ev: Event| {
        if let Some((row, col)) = hovered() {
            on_input(row, col);
            ev.prevent_default();
        } else {
            game_over()
        }
    };

    window_event_listener(ev::keydown, move |ev| on_trigger(ev.into()));
    window_event_listener(ev::touchstart, move |ev| on_trigger(ev.into()));
    window_event_listener(ev::mousedown, move |ev| on_trigger(ev.into()));

    window_event_listener(ev::mouseover, move |ev| {
        use wasm_bindgen::JsCast;

        let Some(target) = ev.target() else {
            set_hovered(None);
            return;
        };

        let element = target.unchecked_into::<web_sys::Element>();

        if element.class_list().contains("cell") {
            let attrs = element.attributes();
            let row = attrs
                .get_named_item("data-row")
                .map(|a| Attr::value(&a).parse());
            let col = attrs
                .get_named_item("data-col")
                .map(|a| Attr::value(&a).parse());

            match (row, col) {
                (Some(Ok(row)), Some(Ok(col))) => set_hovered(Some((row, col))),
                _ => set_hovered(None),
            }
        } else {
            set_hovered(None);
        }
    });

    view! { cx,
        <div class="Game container">
            <div class="Game grid" style=("--columns", columns) style=("--rows", rows)>
                <For
                    each=move || 0..rows()
                    key=|&idx| idx
                    view=move |cx, row| {
                        view! { cx,
                            <div class="Game">
                                <For
                                    each=move || 0..columns()
                                    key=|idx| *idx
                                    view=move |cx, col| {
                                        view! { cx,
                                            <div
                                                class="Game cell"
                                                data-row=row
                                                data-col=col
                                                class:active=move || current().contains(&(row, col))
                                            />
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
