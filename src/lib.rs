#![allow(non_snake_case)]

use std::{
    collections::{HashSet, VecDeque},
    fmt::Write,
    num::NonZeroU32,
};

use leptos::prelude::*;
use leptos::*;
use leptos_router::{components::Router, hooks::query_signal};
use rand::Rng;
use rustc_hash::FxHashMap;
use web_sys::{Attr, Event};
use web_time::Instant;

mod record;

use record::Record;

type SignalPair<T> = (ReadSignal<T>, WriteSignal<T>);
type Position = (u32, u32);
type Positions = HashSet<Position, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Root />
            </main>
        </Router>
    }
}

#[component]
fn Root() -> impl IntoView {
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();

    let rows_query = query_signal::<u32>("r");
    let rows = signal(rows_query.0.get_untracked().unwrap_or_else(|| {
        local_storage
            .get_item("rows")
            .unwrap()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(3)
    }));

    let columns_query = query_signal::<u32>("c");
    let columns = signal(columns_query.0.get_untracked().unwrap_or_else(|| {
        local_storage
            .get_item("columns")
            .unwrap()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(3)
    }));

    let active_query = query_signal::<u32>("a");
    let active = signal(active_query.0.get_untracked().unwrap_or_else(|| {
        local_storage
            .get_item("active")
            .unwrap()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(2)
    }));

    Effect::new(move |_| {
        rows_query.1.set(Some(rows.0()));
    });
    Effect::new(move |_| {
        columns_query.1.set(Some(columns.0()));
    });
    Effect::new(move |_| {
        active_query.1.set(Some(active.0()));
    });

    let current: SignalPair<Positions> = signal(HashSet::with_capacity_and_hasher(
        (active.0.get_untracked() + 1) as usize,
        Default::default(),
    ));

    let history = local_storage
        .get_item("history")
        .unwrap_or_else(|_| Some(String::new()))
        .map(|s| {
            let mut m = FxHashMap::<(u32, u32, u32), VecDeque<Record>>::default();
            let Some((version, mut s)) = s.split_once('\n') else {
                return m;
            };

            match version {
                "1" => {
                    while s.starts_with("\t\t\t") {
                        let Some((rows, rem)) = s.split_once('\n') else {
                            return FxHashMap::default();
                        };
                        s = rem;

                        let Ok(rows) = rows.trim_start_matches("\t\t\t").parse::<u32>() else {
                            return FxHashMap::default();
                        };

                        while s.starts_with("\t\t") {
                            let Some((columns, rem)) = s.split_once('\n') else {
                                return FxHashMap::default();
                            };
                            s = rem;

                            let Ok(columns) = columns.trim_start_matches("\t\t").parse::<u32>()
                            else {
                                return FxHashMap::default();
                            };

                            while s.starts_with("\t") {
                                let Some((active, rem)) = s.split_once('\n') else {
                                    return FxHashMap::default();
                                };
                                s = rem;

                                let Ok(active) = active.trim_start_matches("\t").parse::<u32>()
                                else {
                                    return FxHashMap::default();
                                };

                                while !s.starts_with("\t") {
                                    let Some((record, rem)) = s.split_once('\n') else {
                                        return FxHashMap::default();
                                    };
                                    s = rem;

                                    let Some((pos, score_millis)) = record.split_once(',') else {
                                        return FxHashMap::default();
                                    };

                                    let Some((score, millis)) = score_millis.split_once(',') else {
                                        return FxHashMap::default();
                                    };

                                    let Ok(pos) = pos.parse::<u32>() else {
                                        return FxHashMap::default();
                                    };

                                    let Ok(score) = score.parse::<u32>() else {
                                        return FxHashMap::default();
                                    };

                                    let Ok(millis) = millis.parse::<u128>() else {
                                        return FxHashMap::default();
                                    };

                                    m.entry((rows, columns, active))
                                        .or_insert_with(VecDeque::new)
                                        .push_back(Record::new(
                                            pos, score, millis, rows, columns, active,
                                        ));

                                    if s.is_empty() {
                                        return m;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            return m;
        })
        .unwrap_or_else(FxHashMap::default);

    if history.is_empty() {
        local_storage.delete("history").unwrap();
    }

    let history = signal(history);

    Effect::new(move |_| {
        let mut history_str = String::new();
        writeln!(history_str, "{}", 1).unwrap();
        let mut history_vals: Vec<Record> = Vec::new();
        history.0().values().for_each(|v| history_vals.extend(v));

        if !history_vals.is_empty() {
            history_vals.sort_by_key(|v| v.active);
            history_vals.sort_by_key(|v| v.columns);
            history_vals.sort_by_key(|v| v.rows);

            let mut last_rows: Option<NonZeroU32> = None;
            let mut last_columns: Option<NonZeroU32> = None;
            let mut last_active: Option<NonZeroU32> = None;

            for val in history_vals {
                if last_rows.is_none_or(|v| v.get() != val.rows) {
                    writeln!(history_str, "\t\t\t{}", val.rows).unwrap();
                    last_rows = NonZeroU32::new(val.rows);
                    last_columns = None;
                    last_active = None;
                }
                if last_columns.is_none_or(|v| v.get() != val.columns) {
                    writeln!(history_str, "\t\t{}", val.columns).unwrap();
                    last_columns = NonZeroU32::new(val.columns);
                    last_active = None;
                }

                if last_active.is_none_or(|v| v.get() != val.active) {
                    writeln!(history_str, "\t{}", val.active).unwrap();
                    last_active = NonZeroU32::new(val.active);
                }

                writeln!(history_str, "{},{},{}", val.position, val.score, val.millis).unwrap();
            }
        }

        local_storage.set_item("history", &history_str).unwrap();
    });

    let current_record = signal(Record::new(
        0,
        0,
        0,
        rows.0.get_untracked(),
        columns.0.get_untracked(),
        active.0.get_untracked(),
    ));
    let score = move || current_record.0().score;

    let best_record = Memo::new(move |_| {
        let mut history_obj = history.0();
        let history_obj = history_obj
            .entry((rows.0(), columns.0(), active.0()))
            .or_default();
        std::iter::once(current_record.0())
            .chain(history_obj.iter().copied())
            .max_by(|a, b| {
                use std::cmp::Ordering::*;
                match a.score.cmp(&b.score) {
                    Equal => b.millis.cmp(&a.millis),
                    otherwise => otherwise,
                }
            })
            .unwrap_or_else(|| Record::new(0, 0, 0, rows.0(), columns.0(), active.0()))
    });

    let update_current = move || {
        let active = active.0().min(columns.0() * rows.0() - 1);
        let mut rng = rand::rng();

        current.1.update(|current| {
            current.clear();
            while current.len() < active as usize {
                let new = (
                    rng.random_range(0..rows.0()),
                    rng.random_range(0..columns.0()),
                );
                if current.contains(&new) {
                    continue;
                }

                current.insert(new);
            }
        });
    };

    let max_active = move || rows.0() * columns.0() - 1;
    let score_text = move || {
        format!(
            "Score: {} ({:.2}/s) / {} ({:.2}/s)",
            score(),
            (score() * 1000) as f64 / current_record.0().millis as f64,
            best_record().score,
            (best_record().score * 1000) as f64 / best_record().millis as f64
        )
    };

    view! {
        <div style="display: flex; justify-content: space-evenly;">
            <U32Input name="rows" label="Rows: " min=2 max=|| u32::MAX signal=rows current=current.1 onchange=update_current />
            <U32Input name="columns" label="Columns: " min=2 max=|| u32::MAX signal=columns current=current.1 onchange=update_current />
            <U32Input name="active" label="Active: " min=1 max=max_active signal=active current=current.1 onchange=update_current />
            <button on:click=move |_| {
                history.1.update(|h| {
                    h.insert((rows.0(), columns.0(), active.0()), VecDeque::new());
                });
            }>"Clear History"</button>
        </div>

        <Game current={current} history={history.1} rows={rows.0} columns={columns.0} active={active.0} current_record={current_record} />

        <h3 style="text-align: center;">{score_text}</h3>
        <GameHistory history={history.0} rows={rows.0} columns={columns.0} active={active.0} />
    }
}

#[component]
fn U32Input<M, F>(
    name: &'static str,
    label: &'static str,
    min: u32,
    max: M,
    signal: SignalPair<u32>,
    current: WriteSignal<Positions>,
    onchange: F,
) -> impl IntoView
where
    M: Fn() -> u32 + 'static + Send,
    F: Fn() + 'static,
{
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    view! {
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
                    local_storage.set(name, &signal.0().to_string()).unwrap();
                    onchange();
                }
            />
        </span>
    }
}

#[component]
fn GameHistory(
    history: ReadSignal<FxHashMap<(u32, u32, u32), VecDeque<Record>>>,
    rows: ReadSignal<u32>,
    columns: ReadSignal<u32>,
    active: ReadSignal<u32>,
) -> impl IntoView {
    const EMPTY: VecDeque<Record> = VecDeque::new();
    const EMPTY_REF: &VecDeque<Record> = &EMPTY;

    view! {
        <table class="GameHistory">
            <tr class="GameHistory">
                <th class="GameHistory">"Pos"</th>
                <th class="GameHistory">"Score"</th>
                <th class="GameHistory">"Score/s"</th>
                <th class="GameHistory">"Seconds"</th>
            </tr>

            <For
                each=move || history().get(&(rows(), columns(), active())).unwrap_or_else(|| EMPTY_REF).clone()
                key=|record| record.position
                children=move |record| {
                    view! {
                        <tr class="GameHistory">
                            <td class="GameHistory">{record.position}</td>
                            <td class="GameHistory">{record.score}</td>
                            <td class="GameHistory">{format!("{:.2}", (record.score * 1000) as f64 / record.millis as f64)}</td>
                            <td class="GameHistory">{format!("{:.2}", record.millis as f64 / 1000f64)}</td>
                        </tr>
                    }
                }
            />
        </table>
    }
}

#[component]
fn Game(
    current: SignalPair<Positions>,
    history: WriteSignal<FxHashMap<(u32, u32, u32), VecDeque<Record>>>,
    columns: ReadSignal<u32>,
    rows: ReadSignal<u32>,
    active: ReadSignal<u32>,
    current_record: SignalPair<Record>,
) -> impl IntoView {
    let (current, set_current) = current;
    let (current_record, set_current_record) = current_record;

    let (start, set_start) = signal(Instant::now());
    let (hovered, set_hovered) = signal(None);

    let active = Memo::new(move |_| active().min(rows() * columns() - 1));

    let mut rng = rand::rng();
    set_current.update(|current| {
        current.clear();
        while current.len() < active.get_untracked() as usize {
            let new = (
                rng.random_range(0..rows.get_untracked()),
                rng.random_range(0..columns.get_untracked()),
            );
            if current.contains(&new) {
                continue;
            }

            current.insert(new);
        }
    });

    let game_over = move || {
        if current_record().score > 1 {
            history.update(|history| {
                let entry = history
                    .entry((
                        rows.get_untracked(),
                        columns.get_untracked(),
                        active.get_untracked(),
                    ))
                    .or_insert_with(VecDeque::new);

                entry.push_front(Record::new(
                    entry.len() as u32 + 1,
                    current_record().score,
                    current_record().millis,
                    rows.get_untracked(),
                    columns.get_untracked(),
                    active.get_untracked(),
                ))
            });
        }
        set_current_record.update(|record| record.score = 0);
    };

    let on_input = move |row, col| {
        if current().contains(&(row, col)) {
            let now = Instant::now();
            set_current.update(|current| {
                let mut rng = rand::rng();

                if current_record().score == 0 {
                    set_start(now);
                }

                set_current_record.update(|record| {
                    record.millis = (now - start()).as_millis();
                    record.score += 1;
                });

                let mut new = (rng.random_range(0..rows()), rng.random_range(0..columns()));
                while current.contains(&new) {
                    new = (rng.random_range(0..rows()), rng.random_range(0..columns()));
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

        let Ok(element) = target.dyn_into::<web_sys::Element>() else {
            return;
        };

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

    view! {
        <div class="Game container">
            <div class="Game grid" style=("--columns", move || columns().to_string()) style=("--rows", move || rows().to_string())>
                <For
                    each=move || 0..rows()
                    key=|&idx| idx
                    children=move |row| {
                        view! {
                            <div class="Game">
                                <For
                                    each=move || 0..columns()
                                    key=|idx| *idx
                                    children=move |col| {
                                        view! {
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
